//! This is a copy of the hyperium/hyper hello.rs example

use tokio_compat_02::FutureExt;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use std::convert::Infallible;

// Start a Tokio 0.3 runtime
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // When we wrap all the 0.2 code in a lazy future we can then
    // compat it with tokio_compat_02::FutureExt and allow all the
    // code to run within the tokio 0.2 context as well as the tokio
    // 0.3 context.
    server().compat().await?;

    Ok(())
}

// In here run any tokio 0.2 code, remember `bind` will eagerly attempt
// to fetch the tokio context so we wrap it all in a future.
async fn server() -> Result<(), Box<dyn std::error::Error>> {
    // For every connection, we must make a `Service` to handle all
    // incoming HTTP requests on said connection.
    let make_svc = make_service_fn(|_conn| {
        // This is the `Service` that will handle the connection.
        // `service_fn` is a helper to convert a function that
        // returns a Response into a `Service`.
        async { Ok::<_, Infallible>(service_fn(hello)) }
    });

    let addr = ([127, 0, 0, 1], 3000).into();

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);

    server.await?;

    Ok(())
}

async fn hello(_: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new(Body::from("Hello World!")))
}
