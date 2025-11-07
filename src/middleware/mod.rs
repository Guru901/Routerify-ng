use crate::types::RequestInfo;
use http_body_util::Full;
use hyper::Response;
use hyper::body::{Body, Bytes};
use std::future::Future;

pub use self::post::PostMiddleware;
pub use self::pre::PreMiddleware;

mod post;
mod pre;

/// Enum type for all the middleware types. Please refer to the [Middleware](./index.html#middleware) for more info.
///
/// This `Middleware<B, E>` type accepts two type parameters: `B` and `E`.
///
/// * The `B` represents the response body type which will be used by route handlers and the middlewares and this body type must implement
///   the [HttpBody](https://docs.rs/hyper/0.14.4/hyper/body/trait.HttpBody.html) trait. For an instance, `B` could be [hyper::Body](https://docs.rs/hyper/0.14.4/hyper/body/struct.Body.html)
///   type.
/// * The `E` represents any error type which will be used by route handlers and the middlewares. This error type must implement the [std::error::Error](https://doc.rust-lang.org/std/error/trait.Error.html).
#[derive(Debug)]
pub enum Middleware<T, E> {
    /// Variant for the pre middleware. Refer to [Pre Middleware](./index.html#pre-middleware) for more info.
    Pre(PreMiddleware<T, E>),

    /// Variant for the post middleware. Refer to [Post Middleware](./index.html#post-middleware) for more info.
    Post(PostMiddleware<E>),
}

impl<T: Body, E: Into<Box<dyn std::error::Error + Send + Sync>> + 'static> Middleware<T, E> {
    /// Creates a pre middleware with a handler at the `/*` path.
    ///
    /// # Examples
    ///
    /// ```
    /// use routerify_ng::{Middleware, Router};
    /// use std::convert::Infallible;
    ///
    /// fn run() -> Router<Incoming, Infallible> {
    ///     let router = Router::builder()
    ///         .middleware(Middleware::pre(|req| async move {
    ///             /* Do some operations */
    ///             Ok(req)
    ///         }))
    ///         .build()
    ///         .unwrap();
    ///     router
    /// }
    /// ```
    pub fn pre<H, R>(handler: H) -> Middleware<T, E>
    where
        H: Fn(hyper::Request<T>) -> R + Send + Sync + 'static,
        R: Future<Output = Result<hyper::Request<T>, E>> + Send + 'static,
    {
        Middleware::pre_with_path("/*", handler).unwrap()
    }

    /// Creates a post middleware with a handler at the `/*` path.
    ///
    /// # Examples
    ///
    /// ```
    /// use routerify_ng::{Middleware, Router};
    /// use std::convert::Infallible;
    ///
    /// fn run() -> Router<Incoming, Infallible> {
    ///     let router = Router::builder()
    ///         .middleware(Middleware::post(|res| async move {
    ///             /* Do some operations */
    ///             Ok(res)
    ///         }))
    ///         .build()
    ///         .unwrap();
    ///     router
    /// }
    /// ```
    pub fn post<H, R>(handler: H) -> Middleware<T, E>
    where
        H: Fn(Response<Full<Bytes>>) -> R + Send + Sync + 'static,
        R: Future<Output = Result<Response<Full<Bytes>>, E>> + Send + 'static,
    {
        Middleware::post_with_path("/*", handler).unwrap()
    }

    /// Creates a post middleware which can access [request info](./struct.RequestInfo.html) e.g. headers, method, uri etc. It should be used when the post middleware trandforms the response based on
    /// the request information.
    ///
    /// # Examples
    ///
    /// ```
    /// use http_body_util::Full;
    /// use hyper::{body::Bytes, Response};
    /// use routerify_ng::{Middleware, RequestInfo, Router};
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
    ///         .middleware(Middleware::post_with_info(post_middleware_with_info_handler))
    ///         .build()
    ///         .unwrap();
    ///     router
    /// }
    /// ```
    pub fn post_with_info<H, R>(handler: H) -> Middleware<T, E>
    where
        H: Fn(Response<Full<Bytes>>, RequestInfo) -> R + Send + Sync + 'static,
        R: Future<Output = Result<Response<Full<Bytes>>, E>> + Send + 'static,
    {
        Middleware::post_with_info_with_path("/*", handler).unwrap()
    }

    /// Create a pre middleware with a handler at the specified path.
    ///
    /// # Examples
    ///
    /// ```
    ///    use routerify_ng::{Middleware, Router};
    ///    use std::convert::Infallible;
    ///    
    ///    fn run() -> Router<Incoming, Infallible> {
    ///        let router = Router::builder()
    ///            .middleware(
    ///                Middleware::pre_with_path("/my-path", |req| async move {
    ///                    /* Do some operations */
    ///                    Ok(req)
    ///                })
    ///                .unwrap(),
    ///            )
    ///            .build()
    ///            .unwrap();
    ///        router
    ///    }
    /// ```
    pub fn pre_with_path<P, H, R>(path: P, handler: H) -> crate::Result<Middleware<T, E>>
    where
        P: Into<String>,
        H: Fn(hyper::Request<T>) -> R + Send + Sync + 'static,
        R: Future<Output = Result<hyper::Request<T>, E>> + Send + 'static,
    {
        Ok(Middleware::Pre(PreMiddleware::new(path, handler)?))
    }

    /// Creates a post middleware with a handler at the specified path.
    ///
    /// # Examples
    ///
    /// ```
    /// use routerify_ng::{Middleware, Router};
    /// use std::convert::Infallible;
    ///
    /// fn run() -> Router<Incoming, Infallible> {
    ///     let router = Router::builder()
    ///         .middleware(
    ///             Middleware::post_with_path("/my-path", |res| async move {
    ///                 /* Do some operations */
    ///                 Ok(res)
    ///             })
    ///             .unwrap(),
    ///         )
    ///         .build()
    ///         .unwrap();
    ///     router
    /// }
    /// ```
    pub fn post_with_path<P, H, R>(path: P, handler: H) -> crate::Result<Middleware<T, E>>
    where
        P: Into<String>,
        H: Fn(Response<Full<Bytes>>) -> R + Send + Sync + 'static,
        R: Future<Output = Result<Response<Full<Bytes>>, E>> + Send + 'static,
    {
        Ok(Middleware::Post(PostMiddleware::new(path, handler)?))
    }

    /// Creates a post middleware which can access [request info](./struct.RequestInfo.html) e.g. headers, method, uri etc. It should be used when the post middleware trandforms the response based on
    /// the request information.
    ///
    /// # Examples
    ///
    /// ```
    /// use http_body_util::Full;
    /// use routerify_ng::{Middleware, RequestInfo, Router};
    /// use std::convert::Infallible;
    /// use hyper::{Response, body::Bytes};
    ///
    /// async fn post_middleware_with_info_handler(
    ///     res: Response<Full<Bytes>>,
    ///     req_info: RequestInfo,
    /// ) -> Result<Response<Full<Bytes>>, Infallible> {
    ///     let _headers = req_info.headers();
    ///
    ///     // Do some response transformation based on the request headers, method etc.
    ///
    ///     Ok(res)
    /// }
    ///
    /// fn run() -> Router<Incoming, Infallible> {
    ///     let router = Router::builder()
    ///         .middleware(Middleware::post_with_info_with_path("/abc", post_middleware_with_info_handler).unwrap())
    ///         .build()
    ///         .unwrap();
    ///     router
    /// }
    /// ```
    pub fn post_with_info_with_path<P, H, R>(path: P, handler: H) -> crate::Result<Middleware<T, E>>
    where
        P: Into<String>,
        H: Fn(Response<Full<Bytes>>, RequestInfo) -> R + Send + Sync + 'static,
        R: Future<Output = Result<Response<Full<Bytes>>, E>> + Send + 'static,
    {
        Ok(Middleware::Post(PostMiddleware::new_with_info(path, handler)?))
    }
}
