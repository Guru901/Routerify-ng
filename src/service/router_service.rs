use crate::router::Router;
use crate::service::request_service::{RequestService, RequestServiceBuilder};
use hyper::service::Service;
use std::convert::Infallible;
use std::future::{Ready, ready};
use tokio::net::TcpStream;

/// A [`Service`](https://docs.rs/hyper/0.14.4/hyper/service/trait.Service.html) to process incoming requests.
///
/// This `RouterService<B, E>` type accepts two type parameters: `B` and `E`.
///
/// * The `B` represents the response body type which will be used by route handlers and the middlewares and this body type must implement
///   the [HttpBody](https://docs.rs/hyper/0.14.4/hyper/body/trait.HttpBody.html) trait. For an instance, `B` could be [hyper::Body](https://docs.rs/hyper/0.14.4/hyper/body/struct.Body.html)
///   type.
/// * The `E` represents any error type which will be used by route handlers and the middlewares. This error type must implement the [std::error::Error](https://doc.rust-lang.org/std/error/trait.Error.html).
///
/// # Examples
///
/// ```no_run
/// use http_body_util::Full;
/// use hyper::body::Bytes;
/// use hyper::body::Incoming;
/// use hyper::service::Service;
/// use hyper::{Request, Response};
/// use hyper_util::rt::TokioExecutor;
/// use hyper_util::rt::TokioIo;
/// use hyper_util::server::conn::auto::Builder;
/// use routerify_ng::{Router, RouterService};
/// use std::convert::Infallible;
/// use std::net::SocketAddr;
/// use std::sync::Arc;
/// use tokio::net::TcpListener;
///
/// async fn home(_: Request<Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
///     Ok(Response::new(Full::new(Bytes::from("Home page"))))
/// }
///
/// fn router() -> Router<Infallible> {
///     Router::builder().get("/", home).build().unwrap()
/// }
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
///     let router = router();
///
///     // Create a Service from the router above to handle incoming requests.
///     let service = Arc::new(RouterService::new(router).unwrap());
///
///     let addr: SocketAddr = SocketAddr::from(([127, 0, 0, 1], 3001));
///
///     // Create a server by binding to the address.
///     let listener = TcpListener::bind(addr).await?;
///     println!("App is running on: {}", addr);
///
///     loop {
///         let (stream, _) = listener.accept().await?;
///
///         let router_service = service.clone();
///
///         tokio::spawn(async move {
///             // Get the request service for this connection
///             let request_service = router_service.call(&stream).await.unwrap();
///
///             // Wrap the stream in TokioIo for hyper
///             let io = TokioIo::new(stream);
///
///             // Serve the connection
///             let builder = Builder::new(TokioExecutor::new());
///             if let Err(err) = builder.serve_connection(io, request_service).await {
///                 eprintln!("Error serving connection: {:?}", err);
///             }
///         });
///     }
/// }
/// ```
#[derive(Debug)]
pub struct RouterService<E> {
    builder: RequestServiceBuilder<E>,
}

impl<E: Into<Box<dyn std::error::Error + Send + Sync>> + 'static> RouterService<E> {
    /// Creates a new service with the provided router and it's ready to be used with the hyper [`serve`](https://docs.rs/hyper/0.14.4/hyper/server/struct.Builder.html#method.serve)
    /// method.
    pub fn new(router: Router<E>) -> crate::Result<RouterService<E>> {
        let builder = RequestServiceBuilder::new(router)?;
        Ok(RouterService { builder })
    }
}

impl<E: Into<Box<dyn std::error::Error + Send + Sync>> + 'static> Service<&TcpStream> for RouterService<E> {
    type Response = RequestService<E>;
    type Error = Infallible;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    // fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
    //     Poll::Ready(Ok(()))
    // }

    fn call(&self, conn: &TcpStream) -> Self::Future {
        let addr = match conn.peer_addr() {
            Ok(addr) => addr,
            Err(_) => std::net::SocketAddr::from(([0, 0, 0, 0], 0)),
        };
        let req_service = self.builder.build(addr);

        ready(Ok(req_service))
    }
}
