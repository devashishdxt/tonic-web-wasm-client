use std::{
    pin::Pin,
    task::{Context, Poll},
};

use bytes::Bytes;
use futures_util::{Stream, TryStreamExt, stream::empty};
use http_body::{Body, Frame};
use js_sys::Uint8Array;
use wasm_streams::readable::IntoStream;

use crate::{Error, abort_guard::AbortGuard};

pub struct BodyStream {
    body_stream: Pin<Box<dyn Stream<Item = Result<Bytes, Error>>>>,
    _abort: Option<AbortGuard>,
}

impl BodyStream {
    pub fn new(body_stream: IntoStream<'static>, abort: AbortGuard) -> Self {
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
            _abort: Some(abort),
        }
    }

    pub fn empty() -> Self {
        let body_stream = empty();

        Self {
            body_stream: Box::pin(body_stream),
            _abort: None,
        }
    }
}

impl Body for BodyStream {
    type Data = Bytes;

    type Error = Error;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
        match self.body_stream.as_mut().poll_next(cx) {
            Poll::Ready(maybe) => Poll::Ready(maybe.map(|result| result.map(Frame::data))),
            Poll::Pending => Poll::Pending,
        }
    }
}

unsafe impl Send for BodyStream {}
unsafe impl Sync for BodyStream {}
