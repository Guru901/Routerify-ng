use bytes::Bytes;
use http_body_util::Full;
use hyper::service::Service;
use hyper::Response;
use hyper_util::rt::TokioExecutor;
use hyper_util::rt::TokioIo;
use hyper_util::server::conn::auto::Builder;
// Import the routerify prelude traits.
use routerify::{Router, RouterService};
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

mod users {
    use http::Request;
    use hyper::body::Incoming;
    use routerify::ext::RequestExt;
    use std::sync::Mutex;

    use super::*;

    #[derive(Clone)]
    struct State {
        count: Arc<Mutex<u8>>,
    }

    async fn list(req: Request<Incoming>) -> Result<Response<Full<Bytes>>, io::Error> {
        let count = req.data::<State>().unwrap().count.lock().unwrap();
        Ok(Response::new(Full::from(format!("Suppliers: {}", count))))
    }

    pub fn router() -> Router<io::Error> {
        let state = State {
            count: Arc::new(Mutex::new(20)),
        };
        Router::builder().data(state).get("/", list).build().unwrap()
    }
}

mod offers {
    use std::sync::Mutex;

    use http::Request;
    use hyper::body::Incoming;
    use routerify::ext::RequestExt;

    use super::*;

    #[derive(Clone)]
    struct State {
        count: Arc<Mutex<u8>>,
    }

    async fn list(req: Request<Incoming>) -> Result<Response<Full<Bytes>>, io::Error> {
        let count = req.data::<State>().unwrap().count.lock().unwrap();

        println!("I can also access parent state: {:?}", req.data::<String>().unwrap());

        Ok(Response::new(Full::from(format!("Suppliers: {}", count))))
    }

    pub fn router() -> Router<io::Error> {
        let state = State {
            count: Arc::new(Mutex::new(100)),
        };
        Router::builder().data(state).get("/abc", list).build().unwrap()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let scopes = Router::builder()
        .data("Parent State data".to_owned())
        .scope("/offers", offers::router())
        .scope("/users", users::router())
        .build()
        .unwrap();

    let router = Router::builder().scope("/v1", scopes).build().unwrap();
    dbg!(&router);

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
