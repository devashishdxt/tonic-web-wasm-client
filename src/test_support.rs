//! Helpers for unit tests that feed a [`ResponseBody`] from memory instead of a
//! fetch body, so the decoder can be tested natively with `cargo test`.

use std::{
    collections::VecDeque,
    pin::Pin,
    task::{Context, Poll, Waker},
};

use bytes::{BufMut, Bytes, BytesMut};
use futures_util::stream;
use http::HeaderMap;
use http_body::{Body, Frame};

use crate::{Error, ResponseBody, body_stream::BodyStream};

pub const GRPC_WEB: &str = "application/grpc-web+proto";

/// A grpc-web frame: the flag byte, the big-endian payload length, the payload.
pub fn frame(flag: u8, payload: &[u8]) -> Bytes {
    let mut frame = BytesMut::with_capacity(5 + payload.len());
    frame.put_u8(flag);
    frame.put_u32(payload.len() as u32);
    frame.put_slice(payload);
    frame.freeze()
}

/// A body fed from `chunks`. The stream returns `Pending` before every chunk,
/// as a fetch body does, so `poll_frame` is re-entered at each chunk boundary.
pub fn body_from_chunks(content_type: &str, chunks: Vec<Bytes>) -> ResponseBody {
    let mut chunks = VecDeque::from(chunks);
    let mut pending = false;
    let body_stream = stream::poll_fn(move |_| {
        pending = !pending;
        if pending {
            Poll::Pending
        } else {
            Poll::Ready(chunks.pop_front().map(Ok))
        }
    });

    ResponseBody::from_body_stream(BodyStream::from_stream(body_stream), content_type).unwrap()
}

/// Polls `body` until it returns a frame, an error or the end.
pub fn next_frame(body: &mut ResponseBody) -> Option<Result<Frame<Bytes>, Error>> {
    let mut cx = Context::from_waker(Waker::noop());
    loop {
        if let Poll::Ready(frame) = Pin::new(&mut *body).poll_frame(&mut cx) {
            return frame;
        }
    }
}

/// Drains `body`, returning the data bytes and the trailers.
pub fn drain(mut body: ResponseBody) -> Result<(BytesMut, Option<HeaderMap>), Error> {
    let (mut data, mut trailers) = (BytesMut::new(), None);

    while let Some(frame) = next_frame(&mut body) {
        match frame?.into_data() {
            Ok(bytes) => data.put(bytes),
            Err(frame) => trailers = frame.into_trailers().ok(),
        }
    }

    Ok((data, trailers))
}
