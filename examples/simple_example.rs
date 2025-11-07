use bytes::Bytes;
use http_body_util::Full;
use hyper::body::Incoming;
use hyper::service::Service;
use hyper::{Request, Response};
use hyper_util::rt::TokioExecutor;
use hyper_util::rt::TokioIo;
use hyper_util::server::conn::auto::Builder;
// Import the routerify prelude traits.
use routerify_ng::prelude::*;
use routerify_ng::{Middleware, Router, RouterService};
use std::{convert::Infallible, net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;

// A handler for "/" page.
async fn home_handler(_: Request<Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    Ok(Response::new(Full::from("Home page")))
}

// A handler for "/about" page.
async fn about_handler(_: Request<Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    Ok(Response::new(Full::from("About page")))
}

// A middleware which logs an http request.
async fn logger(req: Request<Incoming>) -> Result<Request<Incoming>, Infallible> {
    println!("{} {} {}", req.remote_addr(), req.method(), req.uri().path());
    Ok(req)
}

fn router() -> Router<Incoming, Infallible> {
    // Create a router and specify the logger middleware and the handlers.
    // Here, "Middleware::pre" means we're adding a pre middleware which will be executed
    // before any route handlers.
    Router::builder()
        .middleware(Middleware::pre(logger))
        .get("/", home_handler)
        .get("/about", about_handler)
        .build()
        .unwrap()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let router = router();

    // Create a Service from the router above to handle incoming requests.
    let router_service = Arc::new(RouterService::new(router)?);

    // The address on which the server will be listening.
    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));

    // Create a server by binding to the address.
    let listener = TcpListener::bind(addr).await?;
    println!("App is running on: {}", addr);

    loop {
        let (stream, _) = listener.accept().await?;

        let router_service = router_service.clone();

        tokio::spawn(async move {
            // Get the request service for this connection
            let request_service = router_service.call(&stream).await.unwrap();

            // Wrap the stream in TokioIo for hyper
            let io = TokioIo::new(stream);

            // Serve the connection
            let builder = Builder::new(TokioExecutor::new());
            if let Err(err) = builder.serve_connection(io, request_service).await {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}
