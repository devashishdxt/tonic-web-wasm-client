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
    /// The thread that owns the JS objects above.
    #[cfg(target_feature = "atomics")]
    thread: std::thread::ThreadId,
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
            #[cfg(target_feature = "atomics")]
            thread: std::thread::current().id(),
        }
    }

    pub fn empty() -> Self {
        let body_stream = empty();

        Self {
            body_stream: Box::pin(body_stream),
            _abort: None,
            #[cfg(target_feature = "atomics")]
            thread: std::thread::current().id(),
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
        #[cfg(target_feature = "atomics")]
        assert_eq!(
            std::thread::current().id(),
            self.thread,
            "a ResponseBody must be polled on the thread that created it"
        );

        match self.body_stream.as_mut().poll_next(cx) {
            Poll::Ready(maybe) => Poll::Ready(maybe.map(|result| result.map(Frame::data))),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[cfg(target_feature = "atomics")]
impl Drop for BodyStream {
    fn drop(&mut self) {
        // Dropping the JS objects on another thread would touch that thread's JS heap. Leak them.
        if std::thread::current().id() != self.thread {
            std::mem::forget(std::mem::replace(&mut self.body_stream, Box::pin(empty())));
            std::mem::forget(self._abort.take());
        }
    }
}

// The stream and the abort guard hold JS objects, which belong to the thread that created them.
// Without wasm threads there is only that one thread. With wasm threads (`target_feature =
// "atomics"`), the body panics if it is polled on another thread and leaks the JS objects if it is
// dropped on another thread, so moving it never touches the JS objects from the wrong thread.
unsafe impl Send for BodyStream {}
// `&BodyStream` gives no access to the JS objects.
unsafe impl Sync for BodyStream {}
