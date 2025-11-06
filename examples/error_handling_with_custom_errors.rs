use bytes::Bytes;
use http_body_util::Full;
use hyper::body::Incoming;
use hyper::service::Service;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioExecutor;
use hyper_util::rt::TokioIo;
use hyper_util::server::conn::auto::Builder;
use routerify_ng::{Router, RouterService};
use std::fmt;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

// Define a custom error enum to model a possible API service error.
#[derive(Debug)]
enum ApiError {
    #[allow(dead_code)]
    Unauthorized,
    Generic(String),
}

impl std::error::Error for ApiError {}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ApiError::Unauthorized => write!(f, "Unauthorized"),
            ApiError::Generic(s) => write!(f, "Generic: {}", s),
        }
    }
}

// Router, handlers and middleware must use the same error type.
// In this case it's `ApiError`.

// A handler for "/" page.
async fn home_handler(_: Request<Incoming>) -> Result<Response<Full<Bytes>>, ApiError> {
    // Simulate failure by returning `ApiError::Generic` variant.
    Err(ApiError::Generic("Something went wrong!".into()))
}

// Define an error handler function which will accept the `routerify_ng::RouteError`
// and generates an appropriate response.
async fn error_handler(err: routerify_ng::RouteError) -> Response<Full<Bytes>> {
    // Because `routerify_ng::RouteError` is a boxed error, it must be
    // downcasted first. Unwrap for simplicity.
    let api_err = err.downcast::<ApiError>().unwrap();

    // Now that we've got the actual error, we can handle it
    // appropriately.
    match api_err.as_ref() {
        ApiError::Unauthorized => Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Full::new(Bytes::new()))
            .unwrap(),
        ApiError::Generic(s) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Full::from(s.to_string()))
            .unwrap(),
    }
}

fn router() -> Router<ApiError> {
    // Create a router and specify the the handlers.
    Router::builder()
        .get("/", home_handler)
        // Specify the error handler to handle any errors caused by
        // a route or any middleware.
        .err_handler(error_handler)
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
