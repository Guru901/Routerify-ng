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
use routerify_ng::{Router, RouterService};
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

// A handler for "/" page.
async fn home_handler(_: Request<Incoming>) -> Result<Response<Full<Bytes>>, io::Error> {
    Ok(Response::new(Full::from("Home page")))
}

// Define a handler for "/users/:userName/books/:bookName" page which will have two
// route parameters: `userName` and `bookName`.
async fn user_book_handler(req: Request<Incoming>) -> Result<Response<Full<Bytes>>, io::Error> {
    let user_name = req.param("userName").unwrap();
    let book_name = req.param("bookName").unwrap();

    Ok(Response::new(Full::from(format!(
        "User: {}, Book: {}",
        user_name, book_name
    ))))
}

fn router() -> Router<io::Error> {
    // Create a router and specify the the handlers.
    Router::builder()
        .get("/", home_handler)
        .get("/users/:userName/books/:bookName", user_book_handler)
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
