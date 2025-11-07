use crate::Error;
use crate::regex_generator::generate_exact_match_regex;
use crate::types::RequestInfo;
use http_body_util::Full;
use hyper::Response;
use hyper::body::Bytes;
use regex::Regex;
use std::fmt::{self, Debug, Formatter};
use std::future::Future;
use std::pin::Pin;

type HandlerWithoutInfo<E> = Box<dyn Fn(Response<Full<Bytes>>) -> HandlerWithoutInfoReturn<E> + Send + Sync + 'static>;
type HandlerWithoutInfoReturn<E> = Box<dyn Future<Output = Result<Response<Full<Bytes>>, E>> + Send + 'static>;

type HandlerWithInfo<E> =
    Box<dyn Fn(Response<Full<Bytes>>, RequestInfo) -> HandlerWithInfoReturn<E> + Send + Sync + 'static>;
type HandlerWithInfoReturn<E> = Box<dyn Future<Output = Result<Response<Full<Bytes>>, E>> + Send + 'static>;

/// The post middleware type. Refer to [Post Middleware](./index.html#post-middleware) for more info.
///
/// This `PostMiddleware<B, E>` type accepts two type parameters: `B` and `E`.
///
/// * The `B` represents the response body type which will be used by route handlers and the middlewares and this body type must implement
///   the [HttpBody](https://docs.rs/hyper/0.14.4/hyper/body/trait.HttpBody.html) trait. For an instance, `B` could be [hyper::Body](https://docs.rs/hyper/0.14.4/hyper/body/struct.Body.html)
///   type.
/// * The `E` represents any error type which will be used by route handlers and the middlewares. This error type must implement the [std::error::Error](https://doc.rust-lang.org/std/error/trait.Error.html).
pub struct PostMiddleware<E> {
    pub(crate) path: String,
    pub(crate) regex: Regex,
    // Make it an option so that when a router is used to scope in another router,
    // It can be extracted out by 'opt.take()' without taking the whole router's ownership.
    pub(crate) handler: Option<Handler<E>>,
    // Scope depth with regards to the top level router.
    pub(crate) scope_depth: u32,
}

pub(crate) enum Handler<E> {
    WithoutInfo(HandlerWithoutInfo<E>),
    WithInfo(HandlerWithInfo<E>),
}

impl<E: Into<Box<dyn std::error::Error + Send + Sync>> + 'static> PostMiddleware<E> {
    pub(crate) fn new_with_boxed_handler<P: Into<String>>(
        path: P,
        handler: Handler<E>,
        scope_depth: u32,
    ) -> crate::Result<PostMiddleware<E>> {
        let path = path.into();
        let (re, _) = generate_exact_match_regex(path.as_str()).map_err(|e| {
            Error::new(format!(
                "Could not create an exact match regex for the post middleware path: {}",
                e
            ))
        })?;

        Ok(PostMiddleware {
            path,
            regex: re,
            handler: Some(handler),
            scope_depth,
        })
    }

    /// Creates a post middleware with a handler at the specified path.
    ///
    /// # Examples
    ///
    /// ```
    /// use routerify_ng::{Middleware, PostMiddleware, Router};
    /// use std::convert::Infallible;
    /// use hyper::body::Incoming;
    ///
    /// fn run() -> Router<Incoming, Infallible> {
    ///     let router = Router::builder()
    ///         .middleware(Middleware::Post(
    ///             PostMiddleware::new("/abc", |res| async move {
    ///                 /* Do some operations */
    ///                 Ok(res)
    ///             })
    ///             .unwrap(),
    ///         ))
    ///         .build()
    ///         .unwrap();
    ///     router
    /// }
    /// ```
    pub fn new<P, H, R>(path: P, handler: H) -> crate::Result<PostMiddleware<E>>
    where
        P: Into<String>,
        H: Fn(Response<Full<Bytes>>) -> R + Send + Sync + 'static,
        R: Future<Output = Result<Response<Full<Bytes>>, E>> + Send + 'static,
    {
        let handler: HandlerWithoutInfo<E> = Box::new(move |res: Response<Full<Bytes>>| Box::new(handler(res)));
        PostMiddleware::new_with_boxed_handler(path, Handler::WithoutInfo(handler), 1)
    }

    /// Creates a post middleware which can access [request info](./struct.RequestInfo.html) e.g. headers, method, uri etc. It should be used when the post middleware trandforms the response based on
    /// the request information.
    ///
    /// # Examples
    ///
    /// ```
    /// use http_body_util::Full;
    /// use hyper::{
    ///     body::{Bytes, Incoming},
    ///     Response,
    /// };
    /// use routerify_ng::{Middleware, PostMiddleware, RequestInfo, Router};
    /// use std::convert::Infallible;
    ///
    /// async fn post_middleware_with_info_handler(
    ///     res: Response<Full<Bytes>>,
    ///     req_info: RequestInfo,
    /// ) -> Result<Response<Full<Bytes>>, Infallible> {
    ///     let headers = req_info.headers();
    ///
    ///     // Do some response transformation based on the request headers, method etc.
    ///
    ///     Ok(res)
    /// }
    ///
    /// fn run() -> Router<Incoming, Infallible> {
    ///     let router = Router::builder()
    ///         .middleware(Middleware::Post(
    ///             PostMiddleware::new_with_info("/abc", post_middleware_with_info_handler).unwrap(),
    ///         ))
    ///         .build()
    ///         .unwrap();
    ///     router
    /// }
    /// ```
    pub fn new_with_info<P, H, R>(path: P, handler: H) -> crate::Result<PostMiddleware<E>>
    where
        P: Into<String>,
        H: Fn(Response<Full<Bytes>>, RequestInfo) -> R + Send + Sync + 'static,
        R: Future<Output = Result<Response<Full<Bytes>>, E>> + Send + 'static,
    {
        let handler: HandlerWithInfo<E> =
            Box::new(move |res: Response<Full<Bytes>>, req_info: RequestInfo| Box::new(handler(res, req_info)));
        PostMiddleware::new_with_boxed_handler(path, Handler::WithInfo(handler), 1)
    }

    pub(crate) fn should_require_req_meta(&self) -> bool {
        if let Some(ref handler) = self.handler {
            match handler {
                Handler::WithInfo(_) => true,
                Handler::WithoutInfo(_) => false,
            }
        } else {
            false
        }
    }

    pub(crate) async fn process(
        &self,
        res: Response<Full<Bytes>>,
        req_info: Option<RequestInfo>,
    ) -> crate::Result<Response<Full<Bytes>>> {
        let handler = self
            .handler
            .as_ref()
            .expect("A router can not be used after mounting into another router");

        match handler {
            Handler::WithoutInfo(handler) => Pin::from(handler(res)).await.map_err(Into::into),
            Handler::WithInfo(handler) => Pin::from(handler(res, req_info.expect("No RequestInfo is provided")))
                .await
                .map_err(Into::into),
        }
    }
}

impl<E> Debug for PostMiddleware<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{{ path: {:?}, regex: {:?} }}", self.path, self.regex)
    }
}
