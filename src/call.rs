use http::{
    HeaderMap, HeaderValue, Request, Response, Uri,
    header::{ACCEPT, CONTENT_TYPE},
    response::Builder,
};
use http_body_util::BodyExt;
use js_sys::{Array, Uint8Array};
use tonic::body::Body;
use wasm_bindgen::JsValue;
use web_sys::{Headers, RequestCredentials, RequestInit};

use crate::{Error, ResponseBody, fetch::fetch, options::FetchOptions};

pub async fn call(
    mut base_url: String,
    request: Request<Body>,
    options: FetchOptions,
) -> Result<Response<ResponseBody>, Error> {
    push_path(&mut base_url, request.uri());

    let headers = prepare_headers(request.headers())?;
    let body = prepare_body(request).await?;

    let request = prepare_request(&base_url, headers, body)?;
    let (init, abort) = options.request_init()?;
    let response = fetch(&request, &init).await?;

    let result = Response::builder().status(response.status());
    let (result, content_type) = set_response_headers(result, &response)?;

    let content_type = content_type.ok_or(Error::MissingContentTypeHeader)?;
    let body_stream = response.body().ok_or(Error::MissingResponseBody)?;

    let body = ResponseBody::new(body_stream, &content_type, abort)?;

    result.body(body).map_err(Into::into)
}

/// Appends the request's path to the base URL. Only the path: for a client built `with_origin`,
/// tonic also puts the origin's scheme and authority into the URI, and the base URL already names
/// the server. A trailing `/` on the base URL is dropped, since the path starts with one.
fn push_path(base_url: &mut String, uri: &Uri) {
    let path = uri.path_and_query().map_or("/", |path| path.as_str());
    base_url.truncate(base_url.trim_end_matches('/').len());
    base_url.push_str(path);
}

fn prepare_headers(header_map: &HeaderMap<HeaderValue>) -> Result<Headers, Error> {
    // Construct default headers.
    let headers = Headers::new().map_err(Error::js_error)?;
    headers
        .append(CONTENT_TYPE.as_str(), "application/grpc-web+proto")
        .map_err(Error::js_error)?;
    headers
        .append(ACCEPT.as_str(), "application/grpc-web+proto")
        .map_err(Error::js_error)?;
    headers.append("x-grpc-web", "1").map_err(Error::js_error)?;

    // Apply default headers.
    for (header_name, header_value) in header_map.iter() {
        // Allow default headers to be overridden except for `content-type`.
        if header_name != CONTENT_TYPE {
            headers
                .set(header_name.as_str(), header_value.to_str()?)
                .map_err(Error::js_error)?;
        }
    }

    Ok(headers)
}

async fn prepare_body(request: Request<Body>) -> Result<Option<JsValue>, Error> {
    let body = Some(request.collect().await?.to_bytes());
    Ok(body.map(|bytes| Uint8Array::from(bytes.as_ref()).into()))
}

fn prepare_request(
    url: &str,
    headers: Headers,
    body: Option<JsValue>,
) -> Result<web_sys::Request, Error> {
    let init = RequestInit::new();

    init.set_method("POST");
    init.set_headers(headers.as_ref());
    if let Some(ref body) = body {
        init.set_body(body);
    }
    init.set_credentials(RequestCredentials::SameOrigin);

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

#[cfg(test)]
mod tests {
    use super::*;

    fn url(base_url: &str, uri: &str) -> String {
        let mut url = base_url.to_string();
        push_path(&mut url, &uri.parse().unwrap());
        url
    }

    #[test]
    fn appends_the_path_to_the_base_url() {
        assert_eq!(
            url("http://localhost:50051", "/echo.Echo/Echo"),
            "http://localhost:50051/echo.Echo/Echo"
        );
        assert_eq!(
            url("http://localhost:50051/api", "/echo.Echo/Echo"),
            "http://localhost:50051/api/echo.Echo/Echo"
        );
    }

    #[test]
    fn drops_a_trailing_slash_from_the_base_url() {
        assert_eq!(
            url("http://localhost:50051/", "/echo.Echo/Echo"),
            "http://localhost:50051/echo.Echo/Echo"
        );
        assert_eq!(
            url("http://localhost:50051/api/", "/echo.Echo/Echo"),
            "http://localhost:50051/api/echo.Echo/Echo"
        );
    }

    #[test]
    fn ignores_the_scheme_and_authority_of_an_origin() {
        assert_eq!(
            url(
                "http://localhost:50051",
                "http://example.com/echo.Echo/Echo"
            ),
            "http://localhost:50051/echo.Echo/Echo"
        );
    }
}
