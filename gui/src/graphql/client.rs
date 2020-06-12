//! A port of `graphql_client::web` module to use futures 0.3 and wasm-bindgen-futures 0.4
//! 
//! + Original: <https://github.com/graphql-rust/graphql-client/blob/master/graphql_client/src/web.rs>
//! + License: <https://github.com/graphql-rust/graphql-client/blob/master/LICENSE-MIT>

use futures::{future, Future, TryFutureExt};
use graphql_client::{GraphQLQuery, Response};
use log::debug;
use std::collections::HashMap;
use thiserror::Error;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::console;

/// The main interface to the library.
///
/// The workflow is the following:
///
/// - create a client
/// - (optionally) configure it
/// - use it to perform queries with the [call] method
pub struct Client {
    endpoint: String,
    headers: HashMap<String, String>,
}

/// All the ways a request can go wrong.
///
/// not exhaustive
#[derive(Debug, Error, PartialEq)]
pub enum ClientError {
    /// The body couldn't be built
    #[error("Request body is not a valid string")]
    Body,
    /// An error caused by window.fetch
    #[error("Network error")]
    Network(String),
    /// Error in a dynamic JS cast that should have worked
    #[error("JS casting error")]
    Cast,
    /// No window object could be retrieved
    #[error(
        "No Window object available - the client works only in a browser (non-worker) context"
    )]
    NoWindow,
    /// Response shape does not match the generated code
    #[error("Response shape error")]
    ResponseShape,
    /// Response could not be converted to text
    #[error("Response conversion to text failed (Response.text threw)")]
    ResponseText,
    /// Exception thrown when building the request
    #[error("Error building the request")]
    RequestError,
    /// Other JS exception
    #[error("Unexpected JS exception")]
    JsException,
}

impl Client {
    /// Initialize a client. The `endpoint` parameter is the URI of the GraphQL API.
    pub fn new<Endpoint>(endpoint: Endpoint) -> Client
    where
        Endpoint: Into<String>,
    {
        Client {
            endpoint: endpoint.into(),
            headers: HashMap::new(),
        }
    }

    /// Add a header to those sent with the requests. Can be used for things like authorization.
    pub fn add_header(&mut self, name: &str, value: &str) {
        self.headers.insert(name.into(), value.into());
    }

    /// Perform a query.
    ///
    // Lint disabled: We can pass by value because it's always an empty struct.
    #[allow(clippy::needless_pass_by_value)]
    pub fn call<Q: GraphQLQuery + 'static>(
        &self,
        _query: Q,
        variables: Q::Variables,
    ) -> impl Future<Output = Result<Response<Q::ResponseData>, ClientError>> + 'static {
        // this can be removed when we convert to async/await
        let endpoint = self.endpoint.clone();
        let custom_headers = self.headers.clone();

        let future = future::ready(web_sys::window().ok_or_else(|| ClientError::NoWindow));

        let future = future.and_then(move |window: web_sys::Window| {
            let to_string_result: Result<(web_sys::Window, String), ClientError> =
                serde_json::to_string(&Q::build_query(variables))
                    .map_err(|_| ClientError::Body)
                    .map(move |body| (window, body));
            future::ready(to_string_result)
        });

        let future = future.and_then(move |(window, body)| {
            let mut request_init = web_sys::RequestInit::new();
            request_init
                .method("POST")
                .body(Some(&JsValue::from_str(&body)));

            future::ready(
                web_sys::Request::new_with_str_and_init(&endpoint, &request_init)
                    .map_err(|_| ClientError::JsException)
                    .map(|request| (window, request)),
            )
        });

        let future = future.and_then(move |(window, request)| {
            let result_closure = || {
                let headers = request.headers();
                headers
                    .set("Content-Type", "application/json")
                    .map_err(|_| ClientError::RequestError)?;
                headers
                    .set("Accept", "application/json")
                    .map_err(|_| ClientError::RequestError)?;

                for (header_name, header_value) in custom_headers.iter() {
                    headers
                        .set(header_name, header_value)
                        .map_err(|_| ClientError::RequestError)?;
                }

                Ok((window, request))
            };

            future::ready(result_closure())
        });

        let future = future.and_then(move |(window, request)| {
            JsFuture::from(window.fetch_with_request(&request))
                .map_err(|err| ClientError::Network(js_sys::Error::from(err).message().into()))
        });

        let future = future.and_then(move |res| {
            debug!("response: {:?}", res);
            console::log_1(&res);

            future::ready(
                res.dyn_into::<web_sys::Response>()
                    .map_err(|_| ClientError::Cast),
            )
        });

        let future = future.and_then(move |cast_response| {
            future::ready(cast_response.text().map_err(|_| ClientError::ResponseText))
        });

        let future = future.and_then(move |text_promise| {
            JsFuture::from(text_promise).map_err(|_| ClientError::ResponseText)
        });

        let future = future.and_then(|text| {
            let response_text = text.as_string().unwrap_or_default();
            debug!("response text as string: {:?}", response_text);
            future::ready(
                serde_json::from_str(&response_text).map_err(|_| ClientError::ResponseShape),
            )
        });

        future
    }
}
