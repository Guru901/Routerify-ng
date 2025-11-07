use crate::Error;
use crate::helpers;
use crate::router::Router;
use crate::types::{RequestContext, RequestInfo, RequestMeta};
use http_body_util::Empty;
use http_body_util::Full;
use hyper::body::Body;
use hyper::body::Bytes;
use hyper::body::Incoming;
use hyper::{Request, Response, service::Service};
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;

pub struct RequestService<T, E> {
    pub(crate) router: Arc<Router<T, E>>,
    pub(crate) remote_addr: SocketAddr,
}

impl<T, E> Service<Request<T>> for RequestService<T, E>
where
    E: Into<Box<dyn std::error::Error + Send + Sync>> + 'static,
    T: Body + Send + 'static,
{
    type Response = Response<Full<Bytes>>;
    type Error = crate::RouteError;
    #[allow(clippy::type_complexity)]
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn call(&self, mut req: Request<T>) -> Self::Future {
        let router = self.router.clone();
        let remote_addr = self.remote_addr;

        let fut = async move {
            helpers::update_req_meta_in_extensions(req.extensions_mut(), RequestMeta::with_remote_addr(remote_addr));

            let mut target_path = helpers::percent_decode_request_path(req.uri().path())
                .map_err(|e| Error::new(format!("Couldn't percent decode request path: {}", e)))?;

            if target_path.is_empty() || target_path.as_bytes()[target_path.len() - 1] != b'/' {
                target_path.push('/');
            }

            let mut req_info = None;
            let should_gen_req_info = router
                .should_gen_req_info
                .expect("The `should_gen_req_info` flag in Router is not initialized");

            let context = RequestContext::new();

            if should_gen_req_info {
                req_info = Some(RequestInfo::new_from_req(&req, context.clone()));
            }

            req.extensions_mut().insert(context);

            router.process(target_path.as_str(), req, req_info.clone()).await
        };

        Box::pin(fut)
    }
}

#[derive(Debug)]
pub struct RequestServiceBuilder<T, E> {
    router: Arc<Router<T, E>>,
}

impl<T: Body + 'static, E: Into<Box<dyn std::error::Error + Send + Sync>> + 'static> RequestServiceBuilder<T, E> {
    pub fn new(mut router: Router<T, E>) -> crate::Result<Self> {
        // router.init_keep_alive_middleware();

        router.init_global_options_route();
        router.init_default_404_route();

        router.init_err_handler();

        router.init_regex_set()?;
        router.init_req_info_gen();
        Ok(Self {
            router: Arc::from(router),
        })
    }

    pub fn build(&self, remote_addr: SocketAddr) -> RequestService<T, E> {
        RequestService {
            router: self.router.clone(),
            remote_addr,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Error, RequestServiceBuilder, RouteError, Router};
    use futures::future::poll_fn;
    use http::Method;
    use http_body_util::{Empty, Full};
    use hyper::service::Service;
    use hyper::{Request, Response, body::Bytes};
    use std::net::SocketAddr;
    use std::str::FromStr;
    use std::task::Poll;

    #[tokio::test]
    async fn should_route_request() {
        const RESPONSE_TEXT: &str = "Hello world!";
        let remote_addr = SocketAddr::from_str("0.0.0.0:8080").unwrap();
        let router: Router<Empty<_>, Error> = Router::builder()
            .get("/", |_: _| async move {
                Ok(Response::new(Full::new(hyper::body::Bytes::from(RESPONSE_TEXT))))
            })
            .build()
            .unwrap();
        let req: Request<Empty<_>> = Request::builder()
            .method(Method::GET)
            .uri("/")
            .body(Empty::<Bytes>::new())
            .unwrap();

        let builder = RequestServiceBuilder::new(router).unwrap();
        let service = builder.build(remote_addr);

        poll_fn(|_| -> Poll<Result<(), RouteError>> { Poll::Ready(Ok(())) })
            .await
            .expect("request service is not ready");

        let resp: Response<Full<hyper::body::Bytes>> = service.call(req).await.unwrap();
        let body = resp.into_body();
        let body_bytes = http_body_util::BodyExt::collect(body).await.unwrap().to_bytes();
        let body = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert_eq!(RESPONSE_TEXT, body)
    }
}
