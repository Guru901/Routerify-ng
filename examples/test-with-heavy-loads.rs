use http_body_util::Full;
use hyper::Response;
use hyper::service::Service;
use hyper_util::rt::TokioExecutor;
use hyper_util::rt::TokioIo;
use hyper_util::server::conn::auto::Builder;
// Import the routerify prelude traits.
use routerify_ng::{Middleware, Router, RouterService};
use std::sync::Arc;
use std::{convert::Infallible, net::SocketAddr};
use tokio::net::TcpListener;

fn router() -> Router<Infallible> {
    let mut builder = Router::builder();

    for i in 0..3000_usize {
        builder = builder.middleware(
            Middleware::pre_with_path(format!("/abc-{}", i), move |req| async move {
                // println!("PreMiddleware: {}", format!("/abc-{}", i));
                Ok(req)
            })
            .unwrap(),
        );

        builder = builder.get(format!("/abc-{}", i), move |_req| async move {
            // println!("Route: {}, params: {:?}", format!("/abc-{}", i), req.params());
            Ok(Response::new(Full::from(format!("/abc-{}", i))))
        });

        builder = builder.middleware(
            Middleware::post_with_path(format!("/abc-{}", i), move |res| async move {
                // println!("PostMiddleware: {}", format!("/abc-{}", i));
                Ok(res)
            })
            .unwrap(),
        );
    }

    builder.build().unwrap()
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
