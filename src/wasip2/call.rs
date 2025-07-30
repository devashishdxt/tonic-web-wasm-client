use crate::{body_stream::BodyStream, options::FetchOptions, Error, ResponseBody};
use bytes::Bytes;
use futures_util::stream::Stream;
use http::{
    header::{ACCEPT, CONTENT_TYPE},
    Request, Response,
};
use http_body_util::BodyExt;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tonic::body::Body;
use wstd::{
    http::{body::IncomingBody, IntoBody as _},
    io::AsyncRead as _,
};

/// A stream that reads from a `wstd::http::IncomingBody` and yields chunks of bytes.
///
/// This is necessary to adapt the `wstd` response body, which is an asynchronous reader,
/// into a `Stream` that can be used with `http_body_util::StreamBody` to create a `tonic`-compatible
/// response body.
pub struct WstdBodyStream {
    // None is invalid state.
    state: Option<WstdBodyStreamState>,
}

type ReadFuture =
    Pin<Box<dyn Future<Output = (IncomingBody, Box<[u8; 4096]>, wstd::io::Result<usize>)>>>;

// The state machine of the stream
enum WstdBodyStreamState {
    /// Ready to start a new read. Owns the body and the buffer.
    Idle {
        body: IncomingBody,
        buf: Box<[u8; 4096]>,
    },
    /// Actively reading. The future owns the body and buffer.
    Reading { fut: ReadFuture },
    /// The stream is complete and should not be polled again.
    Finished,
}

impl WstdBodyStream {
    pub fn new(body: IncomingBody) -> Self {
        Self {
            state: Some(WstdBodyStreamState::Idle {
                body,
                buf: Box::new([0; 4096]),
            }),
        }
    }
}

impl Stream for WstdBodyStream {
    type Item = Result<Bytes, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Temporarily replace self.state with None to satisfy the borrow checker.
        // We will always replace it with a valid state before returning.
        let mut state =
            std::mem::take(&mut self.state).expect("this function never leaves state == None");

        loop {
            match state {
                WstdBodyStreamState::Idle { mut body, mut buf } => {
                    // Create a future that takes ownership of the body and buffer.
                    let fut = Box::pin(async move {
                        let result = body.read(buf.as_mut()).await;
                        (body, buf, result)
                    });

                    // Transition to the Reading state and immediately poll the new future.
                    state = WstdBodyStreamState::Reading { fut };
                    // Continue loop to poll the new future right away.
                }

                WstdBodyStreamState::Reading { mut fut } => {
                    match fut.as_mut().poll(cx) {
                        Poll::Pending => {
                            // The read is not complete. Put the state back and return.
                            self.state = Some(WstdBodyStreamState::Reading { fut });
                            return Poll::Pending;
                        }
                        Poll::Ready((body, buf, result)) => {
                            match result {
                                Ok(0) => {
                                    // EOF. Mark stream as finished.
                                    self.state = Some(WstdBodyStreamState::Finished);
                                    return Poll::Ready(None);
                                }
                                Ok(n) => {
                                    // Successfully read n bytes.
                                    let bytes = Bytes::copy_from_slice(&buf[..n]);
                                    // The read finished. Transition back to Idle for the next read.
                                    self.state = Some(WstdBodyStreamState::Idle { body, buf });
                                    return Poll::Ready(Some(Ok(bytes)));
                                }
                                Err(err) => {
                                    // An error occurred. Mark stream as finished.
                                    self.state = Some(WstdBodyStreamState::Finished);
                                    return Poll::Ready(Some(Err(Error::WstdHttp(err.into()))));
                                }
                            }
                        }
                    }
                }
                WstdBodyStreamState::Finished => {
                    return Poll::Ready(None);
                }
            }
        }
    }
}

// The core async function that handles the request/response logic using wstd.
pub async fn call(
    base_url: String,
    request: Request<Body>,
    _options: Option<FetchOptions>,
) -> Result<Response<ResponseBody>, Error> {
    let request = {
        let url = format!("{}{}", base_url, request.uri().path());
        let (parts, body) = request.into_parts();

        // Build the wstd HTTP request
        let request = wstd::http::Request::builder()
            .uri(url)
            .method(wstd::http::Method::POST);

        let mut request = request
            .header(CONTENT_TYPE.as_str(), "application/grpc-web+proto")
            .header(ACCEPT.as_str(), "application/grpc-web+proto")
            .header("x-grpc-web", "1");

        // Copy request headers
        for (key, value) in &parts.headers {
            request = request.header(key.as_str(), value.as_bytes());
        }
        // Aggregate the entire request body. This does not work with request message stream.
        let body_bytes = body.collect().await?.to_bytes();
        request.body(body_bytes.into_body())?
    };

    let wstd_client = wstd::http::Client::new();
    let (parts, body) = wstd_client
        .send(request)
        .await
        .map_err(Error::WstdHttp)?
        .into_parts();

    let content_type = parts
        .headers
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .ok_or(Error::MissingContentTypeHeader)?;

    let body_stream = Box::pin(WstdBodyStream::new(body));
    let response_body = ResponseBody::new(BodyStream::new(body_stream), content_type)?;

    Ok(Response::from_parts(parts, response_body))
}
