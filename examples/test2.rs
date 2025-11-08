use bytes::Bytes;
use http_body_util::Full;
use hyper::body::Incoming;
use hyper::service::Service;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder;
use routerify_ng::prelude::*;
use routerify_ng::{Middleware, RequestInfo, Router, RouterService};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

#[derive(Clone)]
pub struct State(pub i32);

pub async fn pre_middleware(req: Request<Incoming>) -> Result<Request<Incoming>, routerify_ng::Error> {
    let data = req.data::<State>().map(|s| s.0).unwrap_or(0);
    println!("Pre Data: {}", data);
    println!("Pre Data2: {:?}", req.data::<u32>());

    Ok(req)
}

pub async fn post_middleware(
    res: Response<Full<Bytes>>,
    req_info: RequestInfo,
) -> Result<Response<Full<Bytes>>, routerify_ng::Error> {
    let data = req_info.data::<State>().map(|s| s.0).unwrap_or(0);
    println!("Post Data: {}", data);

    Ok(res)
}

pub async fn home_handler(req: Request<Incoming>) -> Result<Response<Full<Bytes>>, routerify_ng::Error> {
    let data = req.data::<State>().map(|s| s.0).unwrap_or(0);
    println!("Route Data: {}", data);
    println!("Route Data2: {:?}", req.data::<u32>());

    Err(routerify_ng::Error::new("Error"))
}

async fn error_handler(err: routerify_ng::RouteError, req_info: RequestInfo) -> Response<Full<Bytes>> {
    let data = req_info.data::<State>().map(|s| s.0).unwrap_or(0);
    println!("Error Data: {}", data);
    println!("Error Data2: {:?}", req_info.data::<u32>());

    eprintln!("{}", err);
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Full::from(format!("Something went wrong: {}", err)))
        .unwrap()
}

fn router2() -> Router<Incoming, routerify_ng::Error> {
    Router::builder()
        .data(111_u32)
        .get("/a", |req: Request<Incoming>| async move {
            println!("Router2 Data: {:?}", req.data::<&str>());
            println!("Router2 Data: {:?}", req.data::<State>().map(|s| s.0));
            println!("Router2 Data: {:?}", req.data::<u32>());
            Ok(Response::new(Full::from("Hello world!")))
        })
        .build()
        .unwrap()
}

fn router3() -> Router<Incoming, routerify_ng::Error> {
    Router::builder()
        .data(555_u32)
        .get("/h/g/j", |req: Request<Incoming>| async move {
            println!("Router3 Data: {:?}", req.data::<&str>());
            println!("Router3 Data: {:?}", req.data::<State>().map(|s| s.0));
            println!("Router3 Data: {:?}", req.data::<u32>());
            Ok(Response::new(Full::from("Hello world!")))
        })
        .build()
        .unwrap()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let router: Router<Incoming, routerify_ng::Error> = Router::builder()
        .data(State(100))
        .scope("/r", router2())
        .scope("/bcd", router3())
        .data("abcd")
        .middleware(Middleware::pre(pre_middleware))
        .middleware(Middleware::post_with_info(post_middleware))
        .get("/", home_handler)
        .err_handler_with_info(error_handler)
        .build()
        .unwrap();

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
