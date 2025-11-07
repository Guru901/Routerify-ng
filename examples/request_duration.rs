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
use routerify_ng::{Middleware, RequestInfo, Router, RouterService};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

async fn before(req: Request<Incoming>) -> Result<Request<Incoming>, Infallible> {
    req.set_context(tokio::time::Instant::now());
    Ok(req)
}

async fn hello(_: Request<Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    Ok(Response::new(Full::from("Home page")))
}

async fn after(res: Response<Full<Bytes>>, req_info: RequestInfo) -> Result<Response<Full<Bytes>>, Infallible> {
    let started = req_info.context::<tokio::time::Instant>().unwrap();
    let duration = started.elapsed();
    println!("duration {:?}", duration);
    Ok(res)
}

fn router() -> Router<Incoming, Infallible> {
    Router::builder()
        .get("/", hello)
        .middleware(Middleware::pre(before))
        .middleware(Middleware::post_with_info(after))
        .build()
        .unwrap()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let router = router();

    // Create a Service from the router above to handle incoming requests.
    let router_service = Arc::new(RouterService::new(router).unwrap());

    // The address on which the server will be listening.
    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));

    // Create a server by passing the created service to `.serve` method.
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
            let builder = Builder::new(TokioExecutor::new());

            // Serve the connection
            if let Err(err) = builder.serve_connection(io, request_service).await {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}
