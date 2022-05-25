use std::{
    mem::take,
    pin::Pin,
    task::{Context, Poll},
};

use bytes::{Buf, BufMut, Bytes, BytesMut};
use http::{header::HeaderName, HeaderMap, HeaderValue};
use http_body::Body;
use httparse::{Status, EMPTY_HEADER};

use crate::ClientError;

pub struct GrpcResponse {
    data: Bytes,
    trailers: HeaderMap<HeaderValue>,
}

impl GrpcResponse {
    pub fn new(mut body: BytesMut, content_type: &str) -> Result<Self, ClientError> {
        if content_type == "application/grpc-web-text+proto" {
            body = BytesMut::from(base64::decode(body)?.as_slice());
        }

        body.extend(b"\n");

        let compression_flag = body.get_u8();
        let len = body.get_u32();

        let data_bytes = body.split_to(len as usize).freeze();
        body.advance(5);

        let mut trailers_buf = [EMPTY_HEADER; 64];
        let parsed_trailers = match httparse::parse_headers(&body, &mut trailers_buf)
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

        let mut data = BytesMut::with_capacity(data_bytes.len() + 1);
        data.put_u8(compression_flag);
        data.put_u32(len);
        data.extend_from_slice(&data_bytes);

        Ok(Self {
            data: data.freeze(),
            trailers,
        })
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
