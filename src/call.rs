use http::{
    header::{ACCEPT, CONTENT_TYPE},
    response::Builder,
    HeaderMap, HeaderValue, Request, Response,
};
use http_body::Body;
use js_sys::{Array, Uint8Array};
use tonic::body::BoxBody;
use wasm_bindgen::JsValue;
use web_sys::{AbortSignal, Headers, ReferrerPolicy, RequestCredentials, RequestCache, RequestRedirect, RequestInit};

use crate::{fetch::fetch, Error, ResponseBody};

/// Override the default options for the fetch Web API call. See [MDN](https://developer.mozilla.org/en-US/docs/Web/API/fetch#parameters) for details.
#[derive(Debug, Clone)]
pub struct FetchOptions {
    /// Controls what browsers do with credentials (cookies, HTTP authentication entries, and TLS client certificates)
    /// omit, same-origin, or include. The default is same-origin when FetchOptions is not specified.
    pub credentials: RequestCredentials,

    /// These headers are applied to the request after set_response_headers() is called, allowing overrides.
    pub header_override: Option<HeaderMap<HeaderValue>>,

    /// The HTTP method to use for the request. The default is POST.
    pub method: String,

    /// Indicates how the request will use the browser's HTTP cache. The default is "default". 
    pub cache: RequestCache,

    /// Sets how redirects are handled. The default is "follow".
    pub redirect: RequestRedirect,

    /// The referrer of the request. If None, does not override the default referrer.
    pub referrer: Option<String>,

    /// The referrer policy of the request. If None, does not override the default referrer policy.
    pub referrer_policy: Option<ReferrerPolicy>,
    
    /// The integrity value of the request (i.e.: a hash of the body). If None, is not set.
    pub integrity: Option<String>,

    /// The AbortSignal associated with the request. If None, is not set.
    /// This can be used to abort a long-running request.
    pub abort_signal: Option<AbortSignal>,
}

impl Default for FetchOptions {
    fn default() -> Self {
        Self {
            credentials: RequestCredentials::SameOrigin,
            header_override: None,
            method: "POST".to_string(),
            cache: RequestCache::Default,
            redirect: RequestRedirect::Follow,
            referrer: None,
            referrer_policy: None,
            integrity: None,
            abort_signal: None,
        }
    }
}


pub async fn call(
    mut base_url: String,
    request: Request<BoxBody>,
    options: FetchOptions,
) -> Result<Response<ResponseBody>, Error> {
    base_url.push_str(&request.uri().to_string());

    let headers = prepare_headers(request.headers())?;

    if let Some(header_override) = &options.header_override {
        for (header_name, header_value) in header_override.iter() {
            let exists = headers.has(header_name.as_str()).map_err(Error::js_error)?;

            if exists {
                headers.set(header_name.as_str(), header_value.to_str()?).map_err(Error::js_error)?;
            } else {
                headers.append(header_name.as_str(), header_value.to_str()?).map_err(Error::js_error)?;
            }
        }
    }

    let body = prepare_body(request).await?;

    let request = prepare_request(&base_url, headers, body, options)?;
    let response = fetch(&request).await?;

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
    options: FetchOptions,
) -> Result<web_sys::Request, Error> {
    let mut init = RequestInit::new();

    init.method(options.method.as_str())
        .headers(headers.as_ref())
        .body(body.as_ref())
        .cache(options.cache)
        .redirect(options.redirect)
        .credentials(options.credentials);

    if let Some(referrer) = options.referrer {
        init.referrer(referrer.as_str());
    }

    if let Some(referrer_policy) = options.referrer_policy {
        init.referrer_policy(referrer_policy);
    }

    if let Some(integrity) = options.integrity {
        init.integrity(integrity.as_str());
    }

    if let Some(abort_signal) = options.abort_signal {
        init.signal(Some(&abort_signal));
    }

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
