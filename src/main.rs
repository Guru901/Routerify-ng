use http_body_util::Full;
use hyper::body::Incoming;
use hyper::service::Service;
use hyper::{Request, Response, StatusCode};
// Import the routerify prelude traits.
use routerify_ng::prelude::*;
use routerify_ng::{Middleware, RequestInfo, Router, RouterService};
use std::sync::Arc;
use std::{convert::Infallible, net::SocketAddr};
use tokio::net::TcpListener;

// Define an app state to share it across the route handlers and middlewares.
struct State(u64);

// A handler for "/" page.
async fn home_handler(req: Request<Incoming>) -> Result<Response<Full<hyper::body::Bytes>>, Infallible> {
    // Access the app state.
    let state = req.data::<State>().unwrap();
    println!("State value: {}", state.0);

    Ok(Response::new(Full::new(hyper::body::Bytes::from("Home page"))))
}

// A handler for "/users/:userId" page.
async fn user_handler(req: Request<Incoming>) -> Result<Response<Full<hyper::body::Bytes>>, Infallible> {
    let user_id = req.param("userId").unwrap();
    Ok(Response::new(Full::new(hyper::body::Bytes::from(format!(
        "Hello {}",
        user_id
    )))))
}

// A middleware which logs an http request.
async fn logger(req: Request<Incoming>) -> Result<Request<Incoming>, Infallible> {
    println!("{} {} {}", req.remote_addr(), req.method(), req.uri().path());
    Ok(req)
}

// Define an error handler function which will accept the `routerify::Error`
// and the request information and generates an appropriate response.
async fn error_handler(err: routerify_ng::RouteError, _: RequestInfo) -> Response<Full<hyper::body::Bytes>> {
    eprintln!("{}", err);
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Full::new(hyper::body::Bytes::from(format!(
            "Something went wrong: {}",
            err
        ))))
        .unwrap()
}

// Create a `Router<Body, Infallible>` for response body type `hyper::Body`
// and for handler error type `Infallible`.
fn router() -> Router<Incoming, Infallible> {
    // Create a router and specify the logger middleware and the handlers.
    // Here, "Middleware::pre" means we're adding a pre middleware which will be executed
    // before any route handlers.
    Router::builder()
        .middleware(Middleware::pre(logger))
        .get("/", home_handler)
        .get("/users/:userId", user_handler)
        .err_handler_with_info(error_handler)
        .build()
        .unwrap()
}

#[tokio::main]
async fn main() {
    let router = router();

    // Create a Service from the router above to handle incoming requests.
    let service = Arc::new(RouterService::new(router).unwrap());

    // The address on which the server will be listening.
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let listener = TcpListener::bind(addr).await.unwrap();

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let service = Arc::clone(&service);

                tokio::task::spawn(async move {
                    if let Err(err) = service.call(&stream).await {
                        eprintln!("Error serving connection: {:?}", err);
                    }
                });
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
}
