use bytes::Bytes;
use http_body_util::Full;
use hyper::body::Incoming;
use hyper::service::Service;
use hyper::{
    header::{self, HeaderValue},
    Request, Response,
};
use hyper_util::rt::TokioExecutor;
use hyper_util::rt::TokioIo;
use hyper_util::server::conn::auto::Builder;
// Import the routerify prelude traits.
use routerify::prelude::*;
use routerify::{Middleware, RequestInfo, Router, RouterService};
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

// A handler for "/" page.
async fn home_handler(_: Request<Incoming>) -> Result<Response<Full<Bytes>>, io::Error> {
    Ok(Response::new(Full::from("Home page")))
}

// A handler for "/about" page.
async fn about_handler(_: Request<Incoming>) -> Result<Response<Full<Bytes>>, io::Error> {
    Ok(Response::new(Full::from("About page")))
}

// Define a pre middleware handler which will be executed on every request and
// logs some meta.
async fn logger_middleware(req: Request<Incoming>) -> Result<Request<Incoming>, io::Error> {
    println!("{} {} {}", req.remote_addr(), req.method(), req.uri().path());
    Ok(req)
}

// Define a post middleware handler which will be executed on every request and
// adds a header to the response.
async fn my_custom_header_adder_middleware(mut res: Response<Full<Bytes>>) -> Result<Response<Full<Bytes>>, io::Error> {
    res.headers_mut()
        .insert("x-custom-header", HeaderValue::from_static("some value"));
    Ok(res)
}

// Define a post middleware handler which will be executed on every request and
// accesses request information and adds the session cookies to manage session.
async fn my_session_middleware(
    mut res: Response<Full<Bytes>>,
    req_info: RequestInfo,
) -> Result<Response<Full<Bytes>>, io::Error> {
    // Access a cookie.
    let cookie = req_info
        .headers()
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    res.headers_mut()
        .insert(header::SET_COOKIE, HeaderValue::from_str(cookie).unwrap());

    Ok(res)
}

fn router() -> Router< io::Error> {
    // Create a router and specify the the handlers.
    Router::builder()
        // Create a pre middleware using `Middleware::pre()` method
        // and attach it to the router.
        .middleware(Middleware::pre(logger_middleware))
        // Create a post middleware using `Middleware::post()` method
        // and attach it to the router.
        .middleware(Middleware::post(my_custom_header_adder_middleware))
        // Create a post middleware which will require request info using `Middleware::post_with_info()` method
        // and attach it to the router.
        .middleware(Middleware::post_with_info(my_session_middleware))
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
