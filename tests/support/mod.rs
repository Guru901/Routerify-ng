use http_body_util::Full;
use hyper::body::Body;
use hyper::body::Bytes;
use hyper::body::Incoming;
use hyper::service::Service;
use hyper_util::rt::TokioExecutor;
use hyper_util::rt::TokioIo;
use hyper_util::server::conn::auto::Builder;
use routerify_ng::RequestService;
use routerify_ng::{Router, RouterService};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::oneshot::{self, Sender};

pub struct Serve {
    addr: SocketAddr,
    tx: Sender<()>,
}

impl Serve {
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }
    pub fn new_request(&self, method: &str, route: &str) -> http::request::Builder {
        http::request::Request::builder()
            .method(method.to_ascii_uppercase().as_str())
            .uri(format!("http://{}{}", self.addr(), route))
    }
    pub fn shutdown(self) {
        self.tx.send(()).unwrap();
    }
}

pub async fn serve<E>(router: Router<Incoming, E>) -> Serve
where
    E: Into<Box<dyn std::error::Error + Send + Sync>> + 'static,
{
    // Bind a TCP listener to an available port.
    let listener = Arc::new(TcpListener::bind("127.0.0.1:0").await.unwrap());
    let addr = listener.local_addr().unwrap();
    // Build the router service, which must be Arc to clone into spawned tasks.
    let router_service = Arc::new(RouterService::new(router).unwrap());
    let (tx, rx) = oneshot::channel::<()>();
    let listener2 = listener.clone();
    let router_service2 = router_service.clone();

    tokio::spawn(async move {
        loop {
            let (stream, _) = listener2.accept().await.unwrap();
            let router_service = router_service2.clone();
            tokio::spawn(async move {
                let request_service = router_service.call(&stream).await.expect("RouterService failed");
                let io = TokioIo::new(stream);
                let builder = Builder::new(TokioExecutor::new());
                let conn = builder.serve_connection(io, request_service);
            });
        }
    });

    Serve { addr, tx }
}

pub async fn into_text<B>(body: B) -> String
where
    B: hyper::body::Body<Data = Bytes> + Send,
    B::Error: std::error::Error + Send + Sync + 'static,
{
    use http_body_util::BodyExt;
    String::from_utf8_lossy(&body.collect().await.unwrap().to_bytes()).to_string()
}
