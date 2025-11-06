use crate::Error;
use crate::helpers;
use crate::regex_generator::generate_exact_match_regex;
use crate::types::{RequestMeta, RouteParams};
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Method, Request, Response};
use regex::Regex;
use std::fmt::{self, Debug, Formatter};
use std::future::Future;
use std::pin::Pin;

type Handler<E> = Box<dyn Fn(Request<hyper::body::Incoming>) -> HandlerReturn<E> + Send + Sync + 'static>;
type HandlerReturn<E> = Box<dyn Future<Output = Result<Response<Full<Bytes>>, E>> + Send + 'static>;

/// Represents a single route.
///
/// A route consists of a path, http method type(s) and a handler. It shouldn't be created directly, use [RouterBuilder](./struct.RouterBuilder.html) methods
/// to create a route.
///
/// This `Route<B, E>` type accepts two type parameters: `B` and `E`.
///
/// * The `B` represents the response body type which will be used by route handlers and the middlewares and this body type must implement
///   the [HttpBody](https://docs.rs/hyper/0.14.4/hyper/body/trait.HttpBody.html) trait. For an instance, `B` could be [hyper::Body](https://docs.rs/hyper/0.14.4/hyper/body/struct.Body.html)
///   type.
/// * The `E` represents any error type which will be used by route handlers and the middlewares. This error type must implement the [std::error::Error](https://doc.rust-lang.org/std/error/trait.Error.html).
///
/// # Examples
///
/// ```
/// use http_body_util::Full;
/// use hyper::{
///     body::{Bytes, Incoming},
///     Request, Response,
/// };
/// use routerify_ng::Router;
///
/// async fn home_handler(req: Request<Incoming>) -> Result<Response<Full<Bytes>>, hyper::Error> {
///     Ok(Response::new(Full::new(Bytes::from("home"))))
/// }
///
/// fn run() -> Router<hyper::Error> {
///     let router = Router::builder().get("/", home_handler).build().unwrap();
///     router
/// }
/// ```
pub struct Route<E> {
    pub(crate) path: String,
    pub(crate) regex: Regex,
    route_params: Vec<String>,
    // Make it an option so that when a router is used to scope in another router,
    // It can be extracted out by 'opt.take()' without taking the whole router's ownership.
    pub(crate) handler: Option<Handler<E>>,
    pub(crate) methods: Vec<Method>,
    // Scope depth with regards to the top level router.
    pub(crate) scope_depth: u32,
}

impl<E: Into<Box<dyn std::error::Error + Send + Sync>> + 'static> Route<E> {
    pub(crate) fn new_with_boxed_handler<P: Into<String>>(
        path: P,
        methods: Vec<Method>,
        handler: Handler<E>,
        scope_depth: u32,
    ) -> crate::Result<Route<E>> {
        let path = path.into();
        let (re, params) = generate_exact_match_regex(path.as_str()).map_err(|e| {
            Error::new(format!(
                "Could not create an exact match regex for the route path: {}",
                e
            ))
        })?;

        Ok(Route {
            path,
            regex: re,
            route_params: params,
            handler: Some(handler),
            methods,
            scope_depth,
        })
    }

    pub(crate) fn new<P, H, R>(path: P, methods: Vec<Method>, handler: H) -> crate::Result<Route<E>>
    where
        P: Into<String>,
        H: Fn(Request<hyper::body::Incoming>) -> R + Send + Sync + 'static,
        R: Future<Output = Result<Response<Full<Bytes>>, E>> + Send + 'static,
    {
        let handler: Handler<E> = Box::new(move |req: Request<hyper::body::Incoming>| Box::new(handler(req)));
        Route::new_with_boxed_handler(path, methods, handler, 1)
    }

    pub(crate) fn is_match_method(&self, method: &Method) -> bool {
        self.methods.contains(method)
    }

    pub(crate) async fn process(
        &self,
        target_path: &str,
        mut req: Request<hyper::body::Incoming>,
    ) -> crate::Result<Response<Full<Bytes>>> {
        self.push_req_meta(target_path, &mut req);

        let handler = self
            .handler
            .as_ref()
            .expect("A router can not be used after mounting into another router");

        Pin::from(handler(req)).await.map_err(Into::into)
    }

    fn push_req_meta(&self, target_path: &str, req: &mut Request<hyper::body::Incoming>) {
        self.update_req_meta(req, self.generate_req_meta(target_path));
    }

    fn update_req_meta(&self, req: &mut Request<hyper::body::Incoming>, req_meta: RequestMeta) {
        helpers::update_req_meta_in_extensions(req.extensions_mut(), req_meta);
    }

    fn generate_req_meta(&self, target_path: &str) -> RequestMeta {
        let route_params_list = &self.route_params;
        let ln = route_params_list.len();

        let mut route_params = RouteParams::with_capacity(ln);

        if ln > 0 {
            if let Some(caps) = self.regex.captures(target_path) {
                let mut iter = caps.iter();
                // Skip the first match because it's the whole path.
                iter.next();
                for param in route_params_list {
                    if let Some(Some(g)) = iter.next() {
                        route_params.set(param.clone(), g.as_str());
                    }
                }
            }
        }

        RequestMeta::with_route_params(route_params)
    }
}

impl<E> Debug for Route<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{ path: {:?}, regex: {:?}, route_params: {:?}, methods: {:?} }}",
            self.path, self.regex, self.route_params, self.methods
        )
    }
}
