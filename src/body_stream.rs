use std::{
    pin::Pin,
    task::{Context, Poll},
};

use bytes::Bytes;
use futures_util::{stream::empty, Stream, TryStreamExt};
use http_body::Body;
use js_sys::Uint8Array;
use wasm_streams::readable::IntoStream;

use crate::Error;

pub struct BodyStream {
    body_stream: Pin<Box<dyn Stream<Item = Result<Bytes, Error>>>>,
}

impl BodyStream {
    pub fn new(body_stream: IntoStream<'static>) -> Self {
        let body_stream = body_stream
            .map_ok(|js_value| {
                let buffer = Uint8Array::new(&js_value);

                let mut bytes_vec = vec![0; buffer.length() as usize];
                buffer.copy_to(&mut bytes_vec);

                bytes_vec.into()
            })
            .map_err(Error::js_error);

        Self {
            body_stream: Box::pin(body_stream),
        }
    }

    pub fn empty() -> Self {
        let body_stream = empty();

        Self {
            body_stream: Box::pin(body_stream),
        }
    }
}

impl Body for BodyStream {
    type Data = Bytes;

    type Error = Error;

    fn poll_data(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Data, Self::Error>>> {
        self.body_stream.as_mut().poll_next(cx)
    }

    fn poll_trailers(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> Poll<Result<Option<http::HeaderMap>, Self::Error>> {
        Poll::Ready(Ok(None))
    }
}

unsafe impl Send for BodyStream {}
unsafe impl Sync for BodyStream {}
