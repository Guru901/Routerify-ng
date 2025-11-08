//! `Routerify` provides a lightweight, idiomatic, composable and modular router implementation with middleware support for the Rust HTTP library [hyper](https://hyper.rs/).
//!
//! Routerify's core features:
//!
//! - 🌀 Design complex routing using [scopes](https://github.com/routerify/routerify/blob/master/examples/scoped_router.rs) and [middlewares](https://github.com/routerify/routerify/blob/master/examples/middleware.rs)
//!
//! - 🚀 Fast route matching using [`RegexSet`](https://docs.rs/regex/1.4.3/regex/struct.RegexSet.html)
//!
//! - 🍺 Route handlers may return any [HttpBody](https://docs.rs/hyper/0.14.4/hyper/body/trait.HttpBody.html)
//!
//! - ❗ Flexible [error handling](https://github.com/routerify/routerify/blob/master/examples/error_handling_with_request_info.rs) strategy
//!
//! - 💁 [`WebSocket` support](https://github.com/routerify/routerify-websocket) out of the box.
//!
//! - 🔥 Route handlers and middleware [may share state](https://github.com/routerify/routerify/blob/master/examples/share_data_and_state.rs)
//!
//! - 🍗 [Extensive documentation](https://docs.rs/routerify/) and [examples](https://github.com/routerify/routerify/tree/master/examples)
//!
//! To generate a quick server app using [Routerify](https://github.com/routerify/routerify) and [hyper](https://hyper.rs/),
//! please check out [hyper-routerify-server-template](https://github.com/routerify/hyper-routerify-server-template).
//!
//!
//! ## Benchmarks
//!
//! | Framework      | Language    | Requests/sec |
//! |----------------|-------------|--------------|
//! | [hyper v0.14](https://github.com/hyperium/hyper) | Rust 1.50.0 | 144,583 |
//! | [routerify v2.0.0](https://github.com/routerify/routerify) with [hyper v0.14](https://github.com/hyperium/hyper) | Rust 1.50.0 | 144,621 |
//! | [actix-web v3](https://github.com/actix/actix-web) | Rust 1.50.0 | 131,292 |
//! | [warp v0.3](https://github.com/seanmonstar/warp) | Rust 1.50.0 | 145,362 |
//! | [go-httprouter, branch master](https://github.com/julienschmidt/httprouter) | Go 1.16 | 130,662 |
//! | [Rocket, branch master](https://github.com/SergioBenitez/Rocket) | Rust 1.50.0 | 130,045 |
//!
//! For more info, please visit [Benchmarks](https://github.com/routerify/routerify-benchmark).
//!
//! ## Basic Example
//!
//! A simple example using `Routerify` with `hyper` would look like the following:
//!
//! ```no_run
//! use http_body_util::Full;
//! use hyper::body::Incoming;
//! use hyper::service::Service;
//! use hyper::{body::Bytes, Request, Response, StatusCode};
//! use hyper_util::rt::{TokioExecutor, TokioIo};
//! use hyper_util::server::conn::auto::Builder;
//! // Import the routerify prelude traits.
//! use routerify_ng::prelude::*;
//! use routerify_ng::{Middleware, RequestInfo, Router, RouterService};
//! use std::sync::Arc;
//! use std::{convert::Infallible, net::SocketAddr};
//! use tokio::net::TcpListener;
//!
//! // Define an app state to share it across the route handlers and middlewares.
//! #[derive(Clone)]
//! struct State(u64);
//!
//! // A handler for "/" page.
//! async fn home_handler(req: Request<Full<Bytes>>) -> Result<Response<Full<Bytes>>, Infallible> {
//!     // Access the app state.
//!     let state = req.data::<State>().unwrap();
//!     println!("State value: {}", state.0);
//!
//!     Ok(Response::new(Full::new(Bytes::from("Home page"))))
//! }
//!
//! // A handler for "/users/:userId" page.
//! async fn user_handler(req: Request<Full<Bytes>>) -> Result<Response<Full<Bytes>>, Infallible> {
//!     let user_id = req.param("userId").unwrap();
//!     Ok(Response::new(Full::new(Bytes::from(format!("Hello {}", user_id)))))
//! }
//!
//! // A middleware which logs an http request.
//! async fn logger(req: Request<Full<Bytes>>) -> Result<Request<Full<Bytes>>, Infallible> {
//!     println!("{} {} {}", req.remote_addr(), req.method(), req.uri().path());
//!     Ok(req)
//! }
//!
//! // Define an error handler function which will accept the `routerify_ng::Error`
//! // and the request information and generates an appropriate response.
//! async fn error_handler(err: routerify_ng::RouteError, _: RequestInfo) -> Response<Full<Bytes>> {
//!     eprintln!("{}", err);
//!     Response::builder()
//!         .status(StatusCode::INTERNAL_SERVER_ERROR)
//!         .body(Full::new(Bytes::from(format!("Something went wrong: {}", err))))
//!         .unwrap()
//! }
//!
//! // Create a `Router<Infallible>` for response body type `Full<hyper::body::Bytes>`
//! // and for handler error type `Infallible`.
//! fn router() -> Router<Infallible> {
//!     // Create a router and specify the logger middleware and the handlers.
//!     // Here, "Middleware::pre" means we're adding a pre middleware which will be executed
//!     // before any route handlers.
//!     Router::builder()
//!         // Specify the state data which will be available to every route handlers,
//!         // error handler and middlewares.
//!         .data(State(100))
//!         .middleware(Middleware::pre(logger))
//!         .get("/", home_handler)
//!         .get("/users/:userId", user_handler)
//!         .err_handler_with_info(error_handler)
//!         .build()
//!         .unwrap()
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
//!     let router = router();
//!
//!     let router_service = Arc::new(RouterService::new(router).unwrap());
//!
//!     // The address on which the server will be listening.
//!     let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
//!
//!     // Create a server by passing the created service to `.serve` method.
//!     let listener = TcpListener::bind(addr).await?;
//!     println!("App is running on: {}", addr);
//!
//!     loop {
//!         let (stream, _) = listener.accept().await?;
//!
//!         let router_service = router_service.clone();
//!
//!         tokio::spawn(async move {
//!             // Get the request service for this connection
//!             let request_service = router_service.call(&stream).await.unwrap();
//!
//!             // Wrap the stream in TokioIo for hyper
//!             let io = TokioIo::new(stream);
//!             let builder = Builder::new(TokioExecutor::new());
//!
//!             // Serve the connection
//!             if let Err(err) = builder.serve_connection(io, request_service).await {
//!                 eprintln!("Error serving connection: {:?}", err);
//!             }
//!         });
//!     }
//! }
//! ```
//!
//! ## Routing
//!
//! ### Route Handlers
//!
//! A handler could be a function or a closure to handle a request at a specified route path.
//!
//! Here is a handler with a function:
//!
//! ```
//! use http_body_util::Full;
//! use hyper::{
//!     body::{Bytes, Incoming},
//!     Request, Response,
//! };
//! use routerify_ng::Router;
//! use std::convert::Infallible;
//!
//! // A handler for "/about" page.
//! async fn about_handler(_: Request<Full<Bytes>>) -> Result<Response<Full<Bytes>>, Infallible> {
//!     Ok(Response::new(Full::new(Bytes::from("About page"))))
//! }
//!
//! # fn run() -> Router<Infallible> {
//! let router = Router::builder().get("/about", about_handler).build().unwrap();
//! # router
//! # }
//! # run();
//! ```
//!
//! Here is a handler with closure function:
//!
//! ```
//! use http_body_util::Full;
//! use hyper::{body::Bytes, Response};
//! use routerify_ng::Router;
//! # use std::convert::Infallible;
//! use hyper::body::Incoming;
//!
//! # fn run() -> Router<Infallible> {
//! let router = Router::builder()
//!     .get("/about", |req| async move {
//!         Ok(Response::new(Full::new(Bytes::from("About page"))))
//!     })
//!     .build()
//!     .unwrap();
//! # router
//! # }
//! # run();
//! ```
//!
//! ### Route Paths
//!
//! Route paths, in combination with a request method, define the endpoints at which requests can be made.
//! Route paths can be strings or strings with glob pattern `*`.
//!
//!
//! Here are some examples:
//!
//! This route path will match with exactly "/about":
//!
//! ```
//! use http_body_util::Full;
//! use hyper::{body::Bytes, Response};
//! use routerify_ng::Router;
//! # use std::convert::Infallible;
//! use hyper::body::Incoming;
//!
//! # fn run() -> Router<Infallible> {
//! let router = Router::builder()
//!     .get("/about", |req| async move {
//!         Ok(Response::new(Full::new(Bytes::from("About page"))))
//!     })
//!     .build()
//!     .unwrap();
//! # router
//! # }
//! # run();
//! ```
//!
//! A route path using the glob `*` pattern:
//!
//! ```
//! use http_body_util::Full;
//! use hyper::{body::Bytes, Response};
//! use routerify_ng::Router;
//! # use std::convert::Infallible;
//! use hyper::body::Incoming;
//!
//! # fn run() -> Router<Infallible> {
//! let router = Router::builder()
//!     .get("/users/*", |req| async move {
//!         Ok(Response::new(Full::new(Bytes::from(
//!             "It will match /users/, /users/any_path",
//!         ))))
//!     })
//!     .build()
//!     .unwrap();
//! # router
//! # }
//! # run();
//! ```
//!
//! #### Handle 404 Pages
//!
//! Here is an example to handle 404 pages.
//!
//! ```
//! use http_body_util::Full;
//! use hyper::{body::Bytes, Response, StatusCode};
//! use routerify_ng::Router;
//! # use std::convert::Infallible;
//! use hyper::body::Incoming;
//!
//! # fn run() -> Router<Infallible> {
//! let router = Router::builder()
//!     .get(
//!         "/users",
//!         |req| async move { Ok(Response::new(Full::new(Bytes::from("User List")))) },
//!     )
//!     // It fallbacks to the following route for any non-existent routes.
//!     .any(|_req| async move {
//!         Ok(Response::builder()
//!             .status(StatusCode::NOT_FOUND)
//!             .body(Full::new(Bytes::from("NOT FOUND")))
//!             .unwrap())
//!     })
//!     .build()
//!     .unwrap();
//! # router
//! # }
//! # run();
//! ```
//!
//! ### Route Parameters
//!
//! Route parameters are named URL segments that are used to capture the values specified at their position in the URL.
//! The captured values can be accessed by `req.params` and `re.param` methods using the name of the route parameter specified in the path.
//!
//! ```txt
//! Route path: /users/:userName/books/:bookName
//! Request URL: http://localhost:3000/users/alice/books/HarryPotter
//! req.params() returns a hashmap: { "userName": "alice", "bookName": "HarryPotter" }
//! ```
//!
//! To define routes with route parameters, simply specify the route parameters in the path of the route as shown below.
//!
//! ```
//! use http_body_util::Full;
//! use routerify_ng::Router;
//! // Add routerify prelude traits.
//! use hyper::{body::Bytes, Response};
//! use routerify_ng::prelude::*;
//! # use std::convert::Infallible;
//! use hyper::body::Incoming;
//!
//! # fn run() -> Router<Infallible> {
//! let router = Router::builder()
//!     .get("/users/:userName/books/:bookName", |req| async move {
//!         let user_name = req.param("userName").unwrap();
//!         let book_name = req.param("bookName").unwrap();
//!
//!         Ok(Response::new(Full::new(Bytes::from(format!(
//!             "Username: {}, Book Name: {}",
//!             user_name, book_name
//!         )))))
//!     })
//!     .build()
//!     .unwrap();
//! # router
//! # }
//! # run();
//! ```
//!
//! ### Scoping/Mounting Router
//!
//! The `routerify_ng::Router` is a modular, lightweight and mountable router component. A router can be scoped in or mount to a
//! different router.
//!
//! Here is a simple example which creates a Router and it mounts that router at `/api` path with `.scope()` method:
//!
//! ```
//! use http_body_util::Full;
//! use hyper::{body::Bytes, Response};
//! use routerify_ng::prelude::*;
//! use routerify_ng::Router;
//! use std::convert::Infallible;
//! use hyper::body::Incoming;
//!
//! fn api_router() -> Router<Infallible> {
//!     Router::builder()
//!         .get("/books", |req| async move {
//!             Ok(Response::new(Full::new(Bytes::from("List of books"))))
//!         })
//!         .get("/books/:bookId", |req| async move {
//!             Ok(Response::new(Full::new(Bytes::from(format!(
//!                 "Show book: {}",
//!                 req.param("bookId").unwrap()
//!             )))))
//!         })
//!         .build()
//!         .unwrap()
//! }
//!
//! # fn run() -> Router<Infallible> {
//! let router = Router::builder()
//!     // Mounts the API router at "/api" path .
//!     .scope("/api", api_router())
//!     .build()
//!     .unwrap();
//! # router
//! # }
//! # run();
//! ```
//! Now, the app can handle requests to `/api/books` as well as to `/api/books/:bookId`.
//!
//! ## Middleware
//!
//! The `Routerify` also supports Middleware functionality. If you are unfamiliar with Middleware, in short, here a middlewar is a function (or could be a closure
//! function) which access the `req` and `res` object and does some changes to them and passes the transformed request and response object to the other middlewares and the actual route handler
//! to process it.
//!
//! A Middleware function can do the following tasks:
//!
//! - Execute any code.
//! - Transform the request and the response object.
//!
//! Here, the `Routerify` categorizes the middlewares into two different types:
//!
//! ### Pre Middleware
//!
//! The pre Middlewares will be executed before any route handlers and it will access the `req` object and it can also do some changes to the request object
//! if required.
//!
//! Here is an example of a pre middleware:
//!
//! ```
//! use hyper::{body::Incoming, Request};
//! use routerify_ng::{Middleware, Router};
//! use std::convert::Infallible;
//! use hyper::body::Bytes;
//! use http_body_util::Full;
//!
//! // The handler for a pre middleware.
//! // It accepts a `req` and it transforms the `req` and passes it to the next middlewares.
//! async fn my_pre_middleware_handler(req: Request<Full<Bytes>>) -> Result<Request<Full<Bytes>>, Infallible> {
//!     // Do some changes if required.
//!     let transformed_req = req;
//!
//!     // Then return the transformed request object to be consumed by the other middlewares
//!     // and the route handlers.
//!     Ok(transformed_req)
//! }
//!
//! # fn run() -> Router<Infallible> {
//! let router = Router::builder()
//!     // Create a pre middleware instance by `Middleware::pre` method
//!     // and attach it.
//!     .middleware(Middleware::pre(my_pre_middleware_handler))
//!     // A middleware can also be attached on a specific path as shown below.
//!     .middleware(Middleware::pre_with_path("/my-path/log", my_pre_middleware_handler).unwrap())
//!     .build()
//!     .unwrap();
//! # router
//! # }
//! # run();
//! ```
//!
//! Here is a pre middleware which logs the incoming requests:
//!
//! ```
//! use hyper::body::Incoming;
//! use hyper::Request;
//! use routerify_ng::prelude::*;
//! use routerify_ng::{Middleware, Router};
//! use std::convert::Infallible;
//! use hyper::body::Bytes;
//! use http_body_util::Full;
//!
//! async fn logger_middleware_handler(req: Request<Full<Bytes>>) -> Result<Request<Full<Bytes>>, Infallible> {
//!     println!("{} {} {}", req.remote_addr(), req.method(), req.uri().path());
//!     Ok(req)
//! }
//!
//! # fn run() -> Router<Infallible> {
//! let router = Router::builder()
//!     .middleware(Middleware::pre(logger_middleware_handler))
//!     .build()
//!     .unwrap();
//! # router
//! # }
//! # run();
//! ```
//!
//! ### Post Middleware
//!
//! The post Middlewares will be executed after all the route handlers process the request and generates a response and it will access that response object and the request info(optional)
//! and it can also do some changes to the response if required.
//!
//! Here is an example of a post middleware:
//!
//! ```
//! use http_body_util::Full;
//! use hyper::{body::Bytes, Response};
//! use routerify_ng::{Middleware, Router};
//! use std::convert::Infallible;
//! use hyper::body::Incoming;
//!
//! // The handler for a post middleware.
//! // It accepts a `res` and it transforms the `res` and passes it to the next middlewares.
//! async fn my_post_middleware_handler(res: Response<Full<Bytes>>) -> Result<Response<Full<Bytes>>, Infallible> {
//!     // Do some changes if required.
//!     let transformed_res = res;
//!
//!     // Then return the transformed response object to be consumed by the other middlewares.
//!     Ok(transformed_res)
//! }
//!
//! # fn run() -> Router<Infallible> {
//! let router = Router::builder()
//!     // Create a post middleware instance by `Middleware::post` method
//!     // and attach it.
//!     .middleware(Middleware::post(my_post_middleware_handler))
//!     // A middleware can also be attached on a specific path as shown below.
//!     .middleware(Middleware::post_with_path("/my-path/log", my_post_middleware_handler).unwrap())
//!     .build()
//!     .unwrap();
//! # router
//! # }
//! # run();
//! ```
//!
//! Here is a post middleware which adds a header to the response object:
//!
//! ```
//! use routerify_ng::{Router, Middleware};
//! use routerify_ng::prelude::*;
//! use hyper::{Response, header::HeaderValue};
//! use std::convert::Infallible;
//! use http_body_util::Full;
//! use hyper::body::Bytes;
//! use hyper::body::Incoming;
//!
//! async fn my_post_middleware_handler(mut res: Response<Full<Bytes>>) -> Result<Response<Full<Bytes>>, Infallible> {
//!     // Add a header to response object.
//!     res.headers_mut().insert("x-my-custom-header", HeaderValue::from_static("my-value"));
//!
//!     Ok(res)
//! }
//!
//! # fn run() -> Router<Infallible> {
//! let router = Router::builder()
//!     .middleware(Middleware::post(my_post_middleware_handler))
//!     .build()
//!     .unwrap();
//! # router
//! # }
//! # run();
//! ```
//!
//! #### Post Middleware with Request Info
//!
//! Sometimes, the post middleware requires the request informations e.g. headers, method, uri etc to generate a new response. As an example, it could be used to manage
//! sessions. To register this kind of post middleware, you have to use [`Middleware::post_with_info`](./enum.Middleware.html#method.post_with_info) method as follows:
//!
//! ```
//! use http_body_util::Full;
//! use hyper::{body::Bytes, Response};
//! use routerify_ng::{Middleware, RequestInfo, Router};
//! use std::convert::Infallible;
//! use hyper::body::Incoming;
//!
//! // The handler for a post middleware which requires request info.
//! // It accepts `res` and `req_info` and it transforms the `res` and passes it to the next middlewares.
//! async fn post_middleware_with_info_handler(
//!     res: Response<Full<Bytes>>,
//!     req_info: RequestInfo,
//! ) -> Result<Response<Full<Bytes>>, Infallible> {
//!     let transformed_res = res;
//!
//!     // Do some response transformation based on the request headers, method etc.
//!     let _headers = req_info.headers();
//!
//!     // Then return the transformed response object to be consumed by the other middlewares.
//!     Ok(transformed_res)
//! }
//!
//! # fn run() -> Router<Infallible> {
//! let router = Router::builder()
//!     // Create a post middleware instance by `Middleware::post_with_info` method
//!     // and attach it.
//!     .middleware(Middleware::post_with_info(post_middleware_with_info_handler))
//!     // This middleware can also be attached on a specific path as shown below.
//!     .middleware(Middleware::post_with_info_with_path("/my-path", post_middleware_with_info_handler).unwrap())
//!     .build()
//!     .unwrap();
//! # router
//! # }
//! # run();
//! ```
//!
//! ### The built-in Middleware
//!
//! Here is a list of some middlewares which are published in different crates:
//!
//! - [routerify-cors](https://github.com/routerify/routerify-cors): A post middleware which enables `CORS` to the routes.
//! - [routerify-query](https://github.com/routerify/routerify-query): A pre middleware which parses the request query string.
//!
//! ## Data and State Sharing
//!
//! `Routerify` also allows you to share data or app state across the route handlers, middlewares and the error handler via the [`RouterBuilder`](./struct.RouterBuilder.html) method
//! [`data`](./struct.RouterBuilder.html#method.data). As it provides composable router API, it also allows to have app state/data per each sub-router.
//!
//! Here is an example to share app state:
//!
//! ```
//! use http_body_util::Full;
//! use hyper::body::{Bytes, Incoming};
//! use hyper::{Request, Response, StatusCode};
//! // Import the routerify prelude traits.
//! use routerify_ng::prelude::*;
//! use routerify_ng::{Middleware, RequestInfo, Router};
//! # use std::convert::Infallible;
//!
//! // Define an app state to share it across the route handlers, middlewares
//! // and the error handler.
//! #[derive(Clone)]
//! struct State(u64);
//!
//! // A handler for "/" page.
//! async fn home_handler(req: Request<Full<Bytes>>) -> Result<Response<Full<Bytes>>, Infallible> {
//!     // Access the app state.
//!     let state = req.data::<State>().unwrap();
//!     println!("State value: {}", state.0);
//!
//!     Ok(Response::new(Full::new(Bytes::from("Home page"))))
//! }
//!
//! // A middleware which logs an http request.
//! async fn logger(req: Request<Full<Bytes>>) -> Result<Request<Full<Bytes>>, Infallible> {
//!     // You can also access the same state from middleware.
//!     let state = req.data::<State>().unwrap();
//!     println!("State value: {}", state.0);
//!
//!     println!("{} {} {}", req.remote_addr(), req.method(), req.uri().path());
//!     Ok(req)
//! }
//!
//! // Define an error handler function which will accept the `routerify_ng::Error`
//! // and the request information and generates an appropriate response.
//! async fn error_handler(err: routerify_ng::RouteError, req_info: RequestInfo) -> Response<Full<Bytes>> {
//!     // You can also access the same state from error handler.
//!     let state = req_info.data::<State>().unwrap();
//!     println!("State value: {}", state.0);
//!
//!     eprintln!("{}", err);
//!     Response::builder()
//!         .status(StatusCode::INTERNAL_SERVER_ERROR)
//!         .body(Full::new(Bytes::from(format!("Something went wrong: {}", err))))
//!         .unwrap()
//! }
//!
//! // Create a `Router<Infallible>` for response body type `hyper::Body`
//! // and for handler error type `Infallible`.
//! fn router() -> Router<Infallible> {
//!     Router::builder()
//!         // Specify the state data which will be available to every route handlers,
//!         // error handler and middlewares.
//!         .data(State(100))
//!         .middleware(Middleware::pre(logger))
//!         .get("/", home_handler)
//!         .err_handler_with_info(error_handler)
//!         .build()
//!         .unwrap()
//! }
//! ```
//! Here is any example on having app state per each sub-router:
//!
//! ```
//! # use hyper::{Request, Response, StatusCode};
//! # // Import the routerify prelude traits.
//! # use routerify_ng::prelude::*;
//! # use routerify_ng::{Middleware, Router, RouterService, RequestInfo};
//! # use std::{convert::Infallible, net::SocketAddr};
//! # use hyper::body::Incoming;
//!
//! mod foo {
//!     # use std::{convert::Infallible, net::SocketAddr};
//!     # use routerify_ng::{Middleware, Router, RouterService, RequestInfo};
//!     # use hyper::{Request, Response, StatusCode};
//!     use hyper::body::Incoming;
//!
//!     pub fn router() -> Router<Infallible> {
//!         Router::builder()
//!             // Specify data for this sub-router only.
//!             .data("Data for foo router")
//!             .build()
//!             .unwrap()
//!     }
//! }
//!
//! mod bar {
//!     # use std::{convert::Infallible, net::SocketAddr};
//!     # use routerify_ng::{Middleware, Router, RouterService, RequestInfo};
//!     # use hyper::{Request, Response, StatusCode};
//!     use hyper::body::Incoming;
//!
//!     pub fn router() -> Router<Infallible> {
//!         Router::builder()
//!             // Specify data for this sub-router only.
//!             .data("Data for bar router")
//!             .build()
//!             .unwrap()
//!     }
//! }
//!
//! fn router() -> Router<Infallible> {
//!     Router::builder()
//!         // This data will be available to all the child sub-routers.
//!         .data(100_u32)
//!         .scope("/foo", foo::router())
//!         .scope("/bar", bar::router())
//!         .build()
//!         .unwrap()
//! }
//! ```
//!
//! You can also share multiple data as follows:
//!
//! ```
//! # use hyper::{Request, Response, StatusCode};
//! # // Import the routerify prelude traits.
//! # use routerify_ng::prelude::*;
//! # use routerify_ng::{Middleware, Router, RouterService, RequestInfo};
//! # use std::{convert::Infallible, net::SocketAddr};
//! # use hyper::body::Incoming;
//! # use std::sync::Mutex;
//! fn router() -> Router<Infallible> {
//!     Router::builder()
//!         // Share multiple data, a single data for each data type.
//!         .data(100_u32)
//!         .data(String::from("Hello world"))
//!         .build()
//!         .unwrap()
//! }
//! ```
//!
//! ### Request context
//!
//! It's possible to share data local to the request across the route handlers and middleware via the
//! [`RequestExt`](./ext/trait.RequestExt.html) methods [`context`](./ext/trait.RequestExt.html#method.context)
//! and [`set_context`](./ext/trait.RequestExt.html#method.set_context). In the error handler it can be accessed
//! via [`RequestInfo`](./struct.RequestInfo.html) method [`context`](./struct.RequestInfo.html#method.context).
//!
//! ## Error Handling
//!
//! Any route or middleware could go wrong and throw an error. `Routerify` tries to add a default error handler in some cases. But, it also
//! allows to attach a custom error handler. The error handler generates a response based on the error and the request info (optional).
//!
//! Routes and middleware may return any error type. The type must be the same for all routes, middleware and a router instance.
//! The error is boxed into [`RouteError`](./type.RouteError.html)
//! and propagated into an error handler. There, the original error is accessible after downcasting.
//! See this [example](https://github.com/routerify/routerify/tree/master/examples/error_handling_with_custom_errors.rs)
//! for handling custom errors.
//!
//! Here is an basic example:
//!
//! ```
//! use routerify_ng::{Router, Middleware};
//! use routerify_ng::prelude::*;
//! use hyper::{Response, StatusCode, body::{Bytes, Incoming}};
//! use http_body_util::Full;
//!
//! // The error handler will accept the thrown error in routerify_ng::Error type and
//! // it will have to generate a response based on the error.
//! async fn error_handler(err: routerify_ng::RouteError) -> Response<Full<Bytes>> {
//!     Response::builder()
//!       .status(StatusCode::INTERNAL_SERVER_ERROR)
//!       .body(Full::new(Bytes::from("Something went wrong")))
//!       .unwrap()
//! }
//!
//! # fn run() -> Router<hyper::Error> {
//! let router = Router::builder()
//!      .get("/users", |req| async move { Ok(Response::new(Full::new(Bytes::from("It might raise an error")))) })
//!      // Here attach the custom error handler defined above.
//!      .err_handler(error_handler)
//!      .build()
//!      .unwrap();
//! # router
//! # }
//! # run();
//! ```
//!
//! ### Error Handling with Request Info
//!
//! Sometimes, it's needed to to generate response on error based on the request headers, method, uri etc. `Routerify` also provides a method [`err_handler_with_info`](./struct.RouterBuilder.html#method.err_handler_with_info)
//! to register this kind of error handler as follows:
//!
//! ```
//! use routerify_ng::{Router, Middleware, RequestInfo};
//! use routerify_ng::prelude::*;
//! use hyper::{Response, StatusCode, body::{Bytes, Incoming}};
//! use http_body_util::Full;
//!
//! // The error handler will accept the thrown error and the request info and
//! // it will generate a response.
//! async fn error_handler(err: routerify_ng::RouteError, req_info: RequestInfo) -> Response<Full<Bytes>> {
//!     // Now generate response based on the `err` and the `req_info`.
//!     Response::builder()
//!       .status(StatusCode::INTERNAL_SERVER_ERROR)
//!       .body(Full::new(Bytes::from("Something went wrong")))
//!       .unwrap()
//! }
//!
//! # fn run() -> Router<hyper::Error> {
//! let router = Router::builder()
//!      .get("/users", |req| async move { Ok(Response::new(Full::new(Bytes::from("It might raise an error")))) })
//!      // Now register this error handler.
//!      .err_handler_with_info(error_handler)
//!      .build()
//!      .unwrap();
//! # router
//! # }
//! # run();
//! ```

pub use self::error::{Error, RouteError};
pub use self::middleware::{Middleware, PostMiddleware, PreMiddleware};
pub use self::route::Route;
pub use self::router::{Router, RouterBuilder};
#[doc(hidden)]
pub use self::service::RequestService;
pub use self::service::RequestServiceBuilder;
pub use self::service::RouterService;
pub use self::types::{RequestInfo, RouteParams};

mod constants;
mod data_map;
mod error;
pub mod ext;
mod helpers;
mod middleware;
pub mod prelude;
mod regex_generator;
mod route;
mod router;
mod service;
mod types;

/// A Result type often returned from methods that can have routerify errors.
pub type Result<T> = std::result::Result<T, RouteError>;
