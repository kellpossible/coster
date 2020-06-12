use log::{debug, info};
use mime_guess;
use rust_embed::RustEmbed;
use std::{convert::Infallible};
use warp::{
    filters::BoxedFilter,
    http,
    http::header::HeaderValue,
    path::Tail,
    reply,
    Filter, Rejection, Reply, hyper::StatusCode,
};
use async_graphql::{Object, Schema, EmptyMutation, EmptySubscription, QueryBuilder, http::{GraphQLPlaygroundConfig, playground_source}};
use async_graphql_warp::{BadRequest, GQLResponse};

#[derive(RustEmbed)]
#[folder = "public/"]
struct Asset;

struct Query;

#[Object]
impl Query {
    #[field(desc = "Returns the sum of a and b")]
    async fn add(&self, a: i32, b: i32) -> i32 {
        a + b
    }
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let localhost: [u8; 4] = [0, 0, 0, 0];
    let port: u16 = 8000;
    let addr = (localhost, port);

    let routes = api()
        .or(static_files_handler())
        .or(index_static_file_redirect());

    println!("Serving on http://0.0.0.0:8000");
    warp::serve(routes).run(addr).await;
}

pub fn api() -> BoxedFilter<(impl Reply,)> {
    let schema = Schema::build(Query, EmptyMutation, EmptySubscription).finish();

    let graphql_post = async_graphql_warp::graphql(schema).and_then(
        |(schema, builder): (_, QueryBuilder)| async move {
            let resp = builder.execute(&schema).await;
            Ok::<_, Infallible>(GQLResponse::from(resp))
        },
    );

    let graphql_playground = warp::path::end().and(warp::get()).map(|| {
        http::Response::builder()
            .header("content-type", "text/html")
            .body(playground_source(GraphQLPlaygroundConfig::new("/api/")))
    });
    

    // let log = warp::log("coster::api");
    warp::path("api")
        .and(graphql_playground.or(graphql_post).recover(|err: Rejection| async move {
            if let Some(BadRequest(err)) = err.find() {
                return Ok::<_, Infallible>(warp::reply::with_status(
                    err.to_string(),
                    StatusCode::BAD_REQUEST,
                ));
            }

            Ok(warp::reply::with_status(
                "INTERNAL_SERVER_ERROR".to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }))
        .boxed()
}

/// For any path within the path `/dist`, serve the embedded static
/// files.
fn static_files_handler() -> BoxedFilter<(impl Reply,)> {
    // let log = warp::log("coster::dist");
    warp::path("dist")
        .and(warp::get())
        .and(warp::path::tail())
        .and_then(|path: Tail| async move {
            debug!(target: "coster::dist", "Serving a request for static file in dist/{:?}", path);
            serve_impl(path.as_str())
        })
        // .with(log)
        .boxed()
}

/// For any path not already matched, return the index.html, so the app will bootstrap itself
/// regardless of whatever the frontend-specific path is.
fn index_static_file_redirect() -> BoxedFilter<(impl Reply,)> {
    warp::path::tail()
        .and_then(|tail| async move {
            debug!(target: "coster::index-redirect", "Serving index page for path: {:?}", tail);
            serve_impl("index.html")
        })
        .boxed()
}

fn serve_impl(path: &str) -> Result<impl Reply, Rejection> {
    debug!(target: "coster::static-files", "Attempting to serve static file: {}", path);
    let asset = Asset::get(path).ok_or_else(warp::reject::not_found)?;
    debug!(target: "coster::static-files", "Found static file: {}", path);
    let mime = mime_guess::from_path(path).first_or_octet_stream();

    let mut res = reply::Response::new(asset.into());
    res.headers_mut().insert(
        "content-type",
        HeaderValue::from_str(mime.as_ref()).unwrap(),
    );
    Ok(res)
}
