use crate::Error;
use crate::regex_generator::generate_exact_match_regex;
use hyper::Request;
use hyper::body::Body;
use regex::Regex;
use std::fmt::{self, Debug, Formatter};
use std::future::Future;
use std::pin::Pin;

type Handler<T, E> = Box<dyn Fn(Request<T>) -> HandlerReturn<T, E> + Send + Sync + 'static>;

type HandlerReturn<T, E> = Box<dyn Future<Output = Result<Request<T>, E>> + Send + 'static>;

/// The pre middleware type. Refer to [Pre Middleware](./index.html#pre-middleware) for more info.
///
/// This `PreMiddleware<E>` type accepts a single type parameter: `E`.
///
/// * The `E` represents any error type which will be used by route handlers and the middlewares. This error type must implement the [std::error::Error](https://doc.rust-lang.org/std/error/trait.Error.html).
pub struct PreMiddleware<T, E> {
    pub(crate) path: String,
    pub(crate) regex: Regex,
    // Make it an option so that when a router is used to scope in another router,
    // It can be extracted out by 'opt.take()' without taking the whole router's ownership.
    pub(crate) handler: Option<Handler<T, E>>,
    // Scope depth with regards to the top level router.
    pub(crate) scope_depth: u32,
}

impl<T: Body, E: Into<Box<dyn std::error::Error + Send + Sync>> + 'static> PreMiddleware<T, E> {
    pub(crate) fn new_with_boxed_handler<P: Into<String>>(
        path: P,
        handler: Handler<T, E>,
        scope_depth: u32,
    ) -> crate::Result<PreMiddleware<T, E>> {
        let path = path.into();
        let (re, _) = generate_exact_match_regex(path.as_str()).map_err(|e| {
            Error::new(format!(
                "Could not create an exact match regex for the pre middleware path: {}",
                e
            ))
        })?;

        Ok(PreMiddleware {
            path,
            regex: re,
            handler: Some(handler),
            scope_depth,
        })
    }

    /// Creates a pre middleware with a handler at the specified path.
    ///
    /// # Examples
    ///
    /// ```
    /// use routerify_ng::{Middleware, PreMiddleware, Router};
    /// use std::convert::Infallible;
    /// use hyper::body::Incoming;
    ///
    /// fn run() -> Router<Incoming, Infallible> {
    ///     let router = Router::builder()
    ///         .middleware(Middleware::Pre(
    ///             PreMiddleware::new("/abc", |req| async move {
    ///                 /* Do some operations */
    ///                 Ok(req)
    ///             })
    ///             .unwrap(),
    ///         ))
    ///         .build()
    ///         .unwrap();
    ///     router
    /// }
    /// ```
    pub fn new<P, H, R>(path: P, handler: H) -> crate::Result<PreMiddleware<T, E>>
    where
        P: Into<String>,
        H: Fn(Request<T>) -> R + Send + Sync + 'static,
        R: Future<Output = Result<Request<T>, E>> + Send + 'static,
    {
        let handler: Handler<T, E> = Box::new(move |req| Box::new(handler(req)));
        PreMiddleware::new_with_boxed_handler(path, handler, 1)
    }

    pub(crate) async fn process(&self, req: Request<T>) -> crate::Result<Request<T>> {
        let handler = self
            .handler
            .as_ref()
            .expect("A router can not be used after mounting into another router");

        Pin::from(handler(req)).await.map_err(Into::into)
    }
}

impl<T, E> Debug for PreMiddleware<T, E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{{ path: {:?}, regex: {:?} }}", self.path, self.regex)
    }
}
