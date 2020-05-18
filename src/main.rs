use log::{debug, info};
use mime_guess;
use rust_embed::RustEmbed;
use std::path::PathBuf;
use warp::{
    filters::BoxedFilter,
    fs::File,
    http::header::HeaderValue,
    path::{self, Peek, Tail},
    reply::Response,
    Filter, Rejection, Reply,
};

#[derive(RustEmbed)]
#[folder = "public/"]
struct Asset;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    let localhost: [u8; 4] = [0, 0, 0, 0];
    let port: u16 = 8000;
    let addr = (localhost, port);

    let routes = api()
        .or(static_files_handler())
        .or(index_static_file_redirect());

    warp::serve(routes).run(addr).await;
}

pub fn api() -> BoxedFilter<(impl Reply,)> {
    // let log = warp::log("coster::api");
    warp::path("api")
        .and(warp::path!(String))
        .map(|path| {
            debug!(target: "coster::api", "api call: {:?}", path);
            std::convert::identity(path)
        }) // Echos the string back in the response body
        // .with(log)
        .boxed()
}

/// Expose filters that work with static files
// pub fn old_static_files_handler(assets_dir: PathBuf) -> BoxedFilter<(impl Reply,)> {
//     const INDEX_HTML: &str = "index.html";

//     let files =
//         assets(assets_dir.clone()).or(index_static_file_redirect(assets_dir.join(INDEX_HTML)));

//     warp::any().and(files).boxed()
// }

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

    let mut res = Response::new(asset.into());
    res.headers_mut().insert(
        "content-type",
        HeaderValue::from_str(mime.as_ref()).unwrap(),
    );
    Ok(res)
}
