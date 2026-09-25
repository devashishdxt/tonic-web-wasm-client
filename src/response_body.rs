use std::{
    ops::{Deref, DerefMut},
    pin::Pin,
    task::{Context, Poll, ready},
};

use base64::{Engine, prelude::BASE64_STANDARD};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use http::{HeaderMap, HeaderValue, header::HeaderName};
use http_body::Body;
use httparse::{EMPTY_HEADER, Status};
use pin_project::pin_project;
use wasm_bindgen::JsCast;
use web_sys::ReadableStream;

use crate::{Error, abort_guard::AbortGuard, body_stream::BodyStream, content_type::Encoding};

/// If 8th MSB of a frame is `0` for data and `1` for trailer
const TRAILER_BIT: u8 = 0b10000000;

pub struct EncodedBytes {
    encoding: Encoding,
    raw_buf: BytesMut,
    buf: BytesMut,
}

impl EncodedBytes {
    pub fn new(content_type: &str) -> Result<Self, Error> {
        Ok(Self {
            encoding: Encoding::from_content_type(content_type)?,
            raw_buf: BytesMut::new(),
            buf: BytesMut::new(),
        })
    }

    // Return the largest prefix that can be decoded in one call. Keep incomplete
    // quartets buffered and stop after the first padded quartet, since gRPC-web
    // can concatenate independently padded base64 segments.
    #[inline]
    fn max_decodable(&self) -> usize {
        let complete_quartets = (self.raw_buf.len() / 4) * 4;
        self.raw_buf[..complete_quartets]
            .iter()
            .position(|&byte| byte == b'=')
            .map_or(complete_quartets, |position| (position / 4 + 1) * 4)
    }

    fn decode_base64_chunk(&mut self) -> Result<(), Error> {
        while self.raw_buf.len() >= 4 {
            let index = self.max_decodable();

            let decoded = BASE64_STANDARD
                .decode(self.raw_buf.split_to(index))
                .map(Bytes::from)?;
            self.buf.put(decoded);
        }

        Ok(())
    }

    fn append(&mut self, bytes: Bytes) -> Result<(), Error> {
        match self.encoding {
            Encoding::None => self.buf.put(bytes),
            Encoding::Base64 => {
                self.raw_buf.put(bytes);
                self.decode_base64_chunk()?;
            }
        }

        Ok(())
    }

    fn take(&mut self, length: usize) -> BytesMut {
        let new_buf = self.buf.split_off(length);
        std::mem::replace(&mut self.buf, new_buf)
    }
}

impl Deref for EncodedBytes {
    type Target = BytesMut;

    fn deref(&self) -> &Self::Target {
        &self.buf
    }
}

impl DerefMut for EncodedBytes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buf
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadState {
    CompressionFlag,
    DataLength,
    Data(u32),
    TrailerLength,
    Trailer(u32),
    Done,
}

impl ReadState {
    /// Whether the whole body, including the trailers frame, has been read.
    ///
    /// `TrailerLength` and `Trailer(_)` must not count: in those states the
    /// trailers frame has started but more bytes are still needed from the
    /// stream, which happens whenever a chunk boundary lands inside it.
    fn is_done(&self) -> bool {
        matches!(self, ReadState::Done)
    }
}

/// Type to handle HTTP response
#[pin_project]
pub struct ResponseBody {
    #[pin]
    body_stream: BodyStream,
    buf: EncodedBytes,
    incomplete_data: BytesMut,
    data: Option<BytesMut>,
    trailer: Option<HeaderMap>,
    state: ReadState,
    finished_stream: bool,
}

impl ResponseBody {
    pub(crate) fn new(
        body_stream: ReadableStream,
        content_type: &str,
        abort: AbortGuard,
    ) -> Result<Self, Error> {
        let body_stream =
            wasm_streams::ReadableStream::from_raw(body_stream.unchecked_into()).into_stream();

        Ok(Self {
            body_stream: BodyStream::new(body_stream, abort),
            buf: EncodedBytes::new(content_type)?,
            incomplete_data: BytesMut::new(),
            data: None,
            trailer: None,
            state: ReadState::CompressionFlag,
            finished_stream: false,
        })
    }

    #[cfg(test)]
    pub(crate) fn from_body_stream(
        body_stream: BodyStream,
        content_type: &str,
    ) -> Result<Self, Error> {
        Ok(Self {
            body_stream,
            buf: EncodedBytes::new(content_type)?,
            incomplete_data: BytesMut::new(),
            data: None,
            trailer: None,
            state: ReadState::CompressionFlag,
            finished_stream: false,
        })
    }

    fn read_stream(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        if self.finished_stream {
            return Poll::Ready(Ok(()));
        }

        let this = self.project();

        match ready!(this.body_stream.poll_frame(cx)) {
            Some(Ok(frame)) => {
                if let Some(data) = frame.data_ref()
                    && let Err(e) = this.buf.append(data.clone())
                {
                    return Poll::Ready(Err(e));
                };

                Poll::Ready(Ok(()))
            }
            Some(Err(e)) => Poll::Ready(Err(e)),
            None => {
                *this.finished_stream = true;
                Poll::Ready(Ok(()))
            }
        }
    }

    fn step(self: Pin<&mut Self>) -> Result<(), Error> {
        let this = self.project();

        loop {
            match this.state {
                ReadState::CompressionFlag => {
                    if this.buf.is_empty() {
                        // Can't read compression flag right now
                        return Ok(());
                    } else {
                        let compression_flag = this.buf.take(1);

                        if compression_flag[0] & TRAILER_BIT == 0 {
                            this.incomplete_data.unsplit(compression_flag);
                            *this.state = ReadState::DataLength;
                        } else {
                            *this.state = ReadState::TrailerLength;
                        }
                    }
                }
                ReadState::DataLength => {
                    if this.buf.len() < 4 {
                        // Can't read data length right now
                        return Ok(());
                    } else {
                        let data_length_bytes = this.buf.take(4);
                        // According to [`Buf::get_u32`] docs returns u32 from big-endian bytes
                        let data_length = (&data_length_bytes[..]).get_u32();

                        this.incomplete_data.unsplit(data_length_bytes);
                        *this.state = ReadState::Data(data_length);
                    }
                }
                ReadState::Data(data_length) => {
                    let data_length = *data_length as usize;

                    if this.buf.len() < data_length {
                        // Can't read data right now
                        return Ok(());
                    } else {
                        this.incomplete_data.unsplit(this.buf.take(data_length));

                        let new_data = this.incomplete_data.split();

                        if let Some(data) = this.data {
                            data.unsplit(new_data);
                        } else {
                            *this.data = Some(new_data);
                        }

                        *this.state = ReadState::CompressionFlag;
                    }
                }
                ReadState::TrailerLength => {
                    if this.buf.len() < 4 {
                        // Can't read data length right now
                        return Ok(());
                    } else {
                        let trailer_length_bytes = this.buf.take(4);
                        // According to [`Buf::get_u32`] docs returns u32 from big-endian bytes
                        let trailer_length = (&trailer_length_bytes[..]).get_u32();
                        *this.state = ReadState::Trailer(trailer_length);
                    }
                }
                ReadState::Trailer(trailer_length) => {
                    let trailer_length = *trailer_length as usize;

                    if this.buf.len() < trailer_length {
                        // Can't read trailer right now
                        return Ok(());
                    } else {
                        // The trailers frame is consumed and is the last one, so the
                        // body is over even if parsing it fails below.
                        *this.state = ReadState::Done;

                        let mut trailer_bytes = this.buf.take(trailer_length);
                        trailer_bytes.put_u8(b'\n');

                        let mut trailers_buf = [EMPTY_HEADER; 64];
                        let parsed_trailers =
                            match httparse::parse_headers(&trailer_bytes, &mut trailers_buf)
                                .map_err(|_| Error::HeaderParsingError)?
                            {
                                Status::Complete((_, headers)) => Ok(headers),
                                Status::Partial => Err(Error::HeaderParsingError),
                            }?;

                        let mut trailers = HeaderMap::with_capacity(parsed_trailers.len());

                        for parsed_trailer in parsed_trailers {
                            let header_name =
                                HeaderName::from_bytes(parsed_trailer.name.as_bytes())?;
                            let header_value = HeaderValue::from_bytes(parsed_trailer.value)?;
                            trailers.insert(header_name, header_value);
                        }

                        *this.trailer = Some(trailers);
                    }
                }
                ReadState::Done => return Ok(()),
            }
        }
    }
}

impl Body for ResponseBody {
    type Data = Bytes;

    type Error = Error;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
        // Check if there's already some data in buffer and return that
        if self.data.is_some() {
            let data = self.data.take().unwrap();

            return Poll::Ready(Some(Ok(http_body::Frame::data(data.freeze()))));
        }

        // If the whole body is read, return trailers (if available) before ending
        if self.state.is_done() {
            if let Some(trailers) = self.trailer.take() {
                return Poll::Ready(Some(Ok(http_body::Frame::trailers(trailers))));
            }
            return Poll::Ready(None);
        }

        loop {
            // Read bytes from stream
            if let Err(e) = ready!(self.as_mut().read_stream(cx)) {
                return Poll::Ready(Some(Err(e)));
            }

            // Step the state machine
            if let Err(e) = self.as_mut().step() {
                return Poll::Ready(Some(Err(e)));
            }

            if self.data.is_some() {
                // If data is available in buffer, return that
                let data = self.data.take().unwrap();
                return Poll::Ready(Some(Ok(http_body::Frame::data(data.freeze()))));
            } else if self.state.is_done() {
                // If we finished reading the body, return trailers before ending
                if let Some(trailers) = self.trailer.take() {
                    return Poll::Ready(Some(Ok(http_body::Frame::trailers(trailers))));
                }
                return Poll::Ready(None);
            } else if self.finished_stream {
                // If stream is finished but data is not finished return error
                return Poll::Ready(Some(Err(Error::MalformedResponse)));
            }
        }
    }
}

impl Default for ResponseBody {
    fn default() -> Self {
        Self {
            body_stream: BodyStream::empty(),
            buf: EncodedBytes {
                encoding: Encoding::None,
                raw_buf: BytesMut::new(),
                buf: BytesMut::new(),
            },
            incomplete_data: BytesMut::new(),
            data: None,
            trailer: None,
            state: ReadState::Done,
            finished_stream: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_base64_response() {
        let mut buf = EncodedBytes::new("application/grpc-web-text+proto").unwrap();

        buf.append(Bytes::from_static(b"AAAAAAA=")).unwrap();

        assert_eq!(&buf[..], &[0, 0, 0, 0, 0]);
        assert!(buf.raw_buf.is_empty());
    }

    #[test]
    fn buffers_incomplete_base64_quartets() {
        let mut buf = EncodedBytes::new("application/grpc-web-text+proto").unwrap();

        buf.append(Bytes::from_static(b"YWJjZA=")).unwrap();
        assert_eq!(&buf[..], b"abc");
        assert_eq!(&buf.raw_buf[..], b"ZA=");

        // Consuming decoded data must not affect the pending encoded bytes.
        assert_eq!(&buf.take(3)[..], b"abc");
        buf.append(Bytes::from_static(b"=")).unwrap();
        assert_eq!(&buf[..], b"d");
        assert!(buf.raw_buf.is_empty());
    }

    #[test]
    fn decodes_concatenated_base64_segments_at_every_chunk_size() {
        // Include two-byte padding, one-byte padding, and an unpadded segment.
        let encoded = b"YQ==YmM=ZGVmZ2hpag==";

        for chunk_size in 1..=encoded.len() {
            let mut buf = EncodedBytes::new("application/grpc-web-text+proto").unwrap();
            for chunk in encoded.chunks(chunk_size) {
                buf.append(Bytes::copy_from_slice(chunk)).unwrap();
            }

            assert_eq!(&buf[..], b"abcdefghij", "chunk size {chunk_size}");
            assert!(buf.raw_buf.is_empty());
        }
    }

    #[test]
    fn rejects_invalid_base64() {
        for encoded in [b"!!!!", b"AA=A", b"A==="] {
            let mut buf = EncodedBytes::new("application/grpc-web-text+proto").unwrap();
            assert!(matches!(
                buf.append(Bytes::copy_from_slice(encoded)),
                Err(Error::Base64DecodeError(_))
            ));
        }
    }

    #[test]
    fn binary_response_is_not_base64_decoded() {
        let mut buf = EncodedBytes::new("application/grpc-web+proto").unwrap();
        let bytes = Bytes::from_static(b"\x00\x80!!!!");

        buf.append(bytes.clone()).unwrap();

        assert_eq!(&buf[..], &bytes[..]);
        assert!(buf.raw_buf.is_empty());
    }

    mod split_trailers {
        use bytes::{BufMut, Bytes, BytesMut};

        use crate::{
            Error,
            response_body::TRAILER_BIT,
            test_support::{GRPC_WEB, body_from_chunks, drain, frame, next_frame},
        };

        const TRAILERS: &[u8] = b"grpc-status:0\r\ngrpc-message:\r\n";

        /// A grpc-web response: one data frame per message length, followed by a
        /// trailers frame. Returns the wire bytes and the length of the data frames.
        fn wire(message_lens: &[usize], trailers: &[u8]) -> (Bytes, usize) {
            let mut wire = BytesMut::new();
            for &message_len in message_lens {
                let message: Vec<u8> = (0..message_len).map(|i| i as u8).collect();
                wire.put(frame(0x00, &message));
            }
            let data_frames_len = wire.len();
            wire.put(frame(TRAILER_BIT, trailers));
            (wire.freeze(), data_frames_len)
        }

        fn assert_complete(chunks: Vec<Bytes>, expected_data: &[u8], case: &str) {
            let (data, trailers) =
                drain(body_from_chunks(GRPC_WEB, chunks)).unwrap_or_else(|e| panic!("{case}: {e}"));

            assert_eq!(data, expected_data, "{case}");
            let trailers = trailers.unwrap_or_else(|| panic!("{case}: no trailers"));
            assert_eq!(trailers.get("grpc-status").unwrap(), "0", "{case}");
        }

        #[test]
        fn trailers_are_returned_wherever_the_chunk_boundary_lands() {
            let (wire, data_frames_len) = wire(&[1024], TRAILERS);

            for split in 0..=wire.len() {
                let chunks = vec![wire.slice(..split), wire.slice(split..)];
                assert_complete(
                    chunks,
                    &wire[..data_frames_len],
                    &format!("split at {split}"),
                );
            }
        }

        #[test]
        fn trailers_are_returned_when_every_byte_is_its_own_chunk() {
            let (wire, data_frames_len) = wire(&[300, 0, 7], TRAILERS);

            let chunks = (0..wire.len()).map(|i| wire.slice(i..i + 1)).collect();
            assert_complete(chunks, &wire[..data_frames_len], "one byte per chunk");
        }

        #[test]
        fn stream_ending_inside_the_trailers_frame_is_malformed() {
            let (wire, data_frames_len) = wire(&[16], TRAILERS);

            for end in data_frames_len + 1..wire.len() {
                let result = drain(body_from_chunks(GRPC_WEB, vec![wire.slice(..end)]));
                assert!(
                    matches!(result, Err(Error::MalformedResponse)),
                    "truncated at {end}"
                );
            }
        }

        #[test]
        fn body_ends_after_a_trailers_frame_that_fails_to_parse() {
            let (wire, data_frames_len) = wire(&[16], b"not a header\r\n");
            // More bytes after the trailers frame must not be read as trailers.
            let mut body = body_from_chunks(GRPC_WEB, vec![wire.clone(), wire.clone()]);

            let mut data = BytesMut::new();
            let mut errors = 0;
            while let Some(frame) = next_frame(&mut body) {
                match frame {
                    Ok(frame) => data.put(frame.into_data().expect("no trailers were parsed")),
                    Err(e) => {
                        assert!(matches!(e, Error::HeaderParsingError), "{e}");
                        errors += 1;
                    }
                }
            }

            assert_eq!(errors, 1);
            assert_eq!(data, &wire[..data_frames_len]);
        }
    }
}
