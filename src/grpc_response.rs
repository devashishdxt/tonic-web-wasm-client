use std::{
    mem::take,
    pin::Pin,
    task::{Context, Poll},
};

use bytes::{Buf, Bytes, BytesMut};
use http::{header::HeaderName, HeaderMap, HeaderValue};
use http_body::Body;
use httparse::{Status, EMPTY_HEADER};

use crate::ClientError;

/// If 8th MSB of a frame is `0` for data and `1` for trailer
const TRAILER_BIT: u8 = 0b10000000;

pub struct GrpcResponse {
    data: Bytes,
    trailers: HeaderMap<HeaderValue>,
}

impl GrpcResponse {
    pub fn new(mut body: BytesMut, content_type: &str) -> Result<Self, ClientError> {
        if content_type == "application/grpc-web-text+proto" {
            body = BytesMut::from(base64::decode(body)?.as_slice());
        }

        if body.len() < 5 {
            return Ok(Self {
                data: Bytes::new(),
                trailers: Default::default(),
            });
        }

        body.extend(b"\n");

        let body = body.freeze();

        let (data, trailer) = split_data_and_trailer(body);

        let mut trailers_buf = [EMPTY_HEADER; 64];
        let parsed_trailers = match httparse::parse_headers(&trailer, &mut trailers_buf)
            .map_err(|_| ClientError::HeaderParsingError)?
        {
            Status::Complete((_, headers)) => Ok(headers),
            Status::Partial => Err(ClientError::HeaderParsingError),
        }?;

        let mut trailers = HeaderMap::with_capacity(parsed_trailers.len());

        for parsed_trailer in parsed_trailers {
            let header_name = HeaderName::from_bytes(parsed_trailer.name.as_bytes())?;
            let header_value = HeaderValue::from_bytes(parsed_trailer.value)?;
            trailers.insert(header_name, header_value);
        }

        Ok(Self { data, trailers })
    }
}

impl Body for GrpcResponse {
    type Data = Bytes;

    type Error = ClientError;

    fn poll_data(
        mut self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        if self.data.is_empty() {
            Poll::Ready(None)
        } else {
            Poll::Ready(Some(Ok(take(&mut self.data))))
        }
    }

    fn poll_trailers(
        mut self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> Poll<Result<Option<HeaderMap>, Self::Error>> {
        if self.trailers.is_empty() {
            Poll::Ready(Ok(None))
        } else {
            Poll::Ready(Ok(Some(take(&mut self.trailers))))
        }
    }
}

fn split_data_and_trailer(mut body: Bytes) -> (Bytes, Bytes) {
    let mut body_cursor = body.clone();

    let mut data_index = 0;

    let mut compression_flag = body_cursor.get_u8();

    while compression_flag & TRAILER_BIT == 0 {
        let len = body_cursor.get_u32();
        data_index += 5 + (len as usize);

        body_cursor.advance(len as usize);

        compression_flag = body_cursor.get_u8();
    }

    let data = body.split_to(data_index);
    body.advance(5);

    (data, body)
}
