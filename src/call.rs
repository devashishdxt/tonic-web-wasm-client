use http::{
    header::{ACCEPT, CONTENT_TYPE},
    response::Builder,
    HeaderMap, HeaderValue, Request, Response,
};
use http_body::Body;
use js_sys::{Array, Uint8Array};
use tonic::body::BoxBody;
use wasm_bindgen::JsValue;
use web_sys::{Headers, RequestCredentials, RequestInit};

use crate::{fetch::fetch, options::FetchOptions, Error, ResponseBody};

pub async fn call(
    mut base_url: String,
    request: Request<BoxBody>,
    options: Option<FetchOptions>,
) -> Result<Response<ResponseBody>, Error> {
    base_url.push_str(&request.uri().to_string());

    let headers = prepare_headers(request.headers())?;
    let body = prepare_body(request).await?;

    let request = prepare_request(&base_url, headers, body)?;
    let response = fetch(&request, options).await?;

    let result = Response::builder().status(response.status());
    let (result, content_type) = set_response_headers(result, &response)?;

    let content_type = content_type.ok_or(Error::MissingContentTypeHeader)?;
    let body_stream = response.body().ok_or(Error::MissingResponseBody)?;

    let body = ResponseBody::new(body_stream, &content_type)?;

    result.body(body).map_err(Into::into)
}

fn prepare_headers(header_map: &HeaderMap<HeaderValue>) -> Result<Headers, Error> {
    let headers = Headers::new().map_err(Error::js_error)?;

    headers
        .append(CONTENT_TYPE.as_str(), "application/grpc-web+proto")
        .map_err(Error::js_error)?;
    headers
        .append(ACCEPT.as_str(), "application/grpc-web+proto")
        .map_err(Error::js_error)?;
    headers.append("x-grpc-web", "1").map_err(Error::js_error)?;

    for (header_name, header_value) in header_map.iter() {
        if header_name != CONTENT_TYPE && header_name != ACCEPT {
            headers
                .append(header_name.as_str(), header_value.to_str()?)
                .map_err(Error::js_error)?;
        }
    }

    Ok(headers)
}

async fn prepare_body(request: Request<BoxBody>) -> Result<Option<JsValue>, Error> {
    let body = request.into_body().data().await.transpose()?;
    Ok(body.map(|bytes| Uint8Array::from(bytes.as_ref()).into()))
}

fn prepare_request(
    url: &str,
    headers: Headers,
    body: Option<JsValue>,
) -> Result<web_sys::Request, Error> {
    let mut init = RequestInit::new();

    init.method("POST")
        .headers(headers.as_ref())
        .body(body.as_ref())
        .credentials(RequestCredentials::SameOrigin);

    web_sys::Request::new_with_str_and_init(url, &init).map_err(Error::js_error)
}

fn set_response_headers(
    mut result: Builder,
    response: &web_sys::Response,
) -> Result<(Builder, Option<String>), Error> {
    let headers = response.headers();

    let header_iter = js_sys::try_iter(headers.as_ref()).map_err(Error::js_error)?;

    let mut content_type = None;

    if let Some(header_iter) = header_iter {
        for header in header_iter {
            let header = header.map_err(Error::js_error)?;
            let pair: Array = header.into();

            let header_name = pair.get(0).as_string();
            let header_value = pair.get(1).as_string();

            match (header_name, header_value) {
                (Some(header_name), Some(header_value)) => {
                    if header_name == CONTENT_TYPE.as_str() {
                        content_type = Some(header_value.clone());
                    }

                    result = result.header(header_name, header_value);
                }
                _ => continue,
            }
        }
    }

    Ok((result, content_type))
}
