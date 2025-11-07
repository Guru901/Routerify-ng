# Routerify-NG

[![CI](https://github.com/guru901/routerify-ng/actions/workflows/ci.yml/badge.svg)](https://github.com/guru901/routerify-ng/actions)
[![crates.io](https://img.shields.io/crates/v/routerify-ng.svg)](https://crates.io/crates/routerify-ng)
[![Documentation](https://docs.rs/routerify-ng/badge.svg)](https://docs.rs/routerify-ng)
[![License](https://img.shields.io/crates/l/routerify-ng.svg)](./LICENSE)

**Routerify-NG** (Next Generation) is a modern, lightweight, idiomatic, and modular router for the Rust HTTP library [Hyper 1.x](https://hyper.rs/).  
It’s a maintained and upgraded fork of the original [Routerify](https://github.com/routerify/routerify) rewritten for the new Hyper service model.

---

## ✨ Highlights

- 🌀 Build complex routing with [scopes](https://github.com/guru901/routerify-ng/blob/main/examples/scoped_router.rs) and [middlewares](https://github.com/guru901/routerify-ng/blob/main/examples/middleware.rs)
- ⚙️ Fully compatible with **Hyper 1.x** and **Tokio 1.x**
- 🚀 Fast route matching via [`RegexSet`](https://docs.rs/regex/latest/regex/struct.RegexSet.html)
- 🧩 Middleware system with shared state between routes
- 💬 Robust [error handling](https://github.com/guru901/routerify-ng/blob/main/examples/error_handling.rs)
- 🔄 [`WebSocket`](https://github.com/routerify/routerify-websocket) support (compatible with Hyper 1.x)
- 📚 Extensive documentation and examples

---

## ⚡ Benchmarks

| Framework                                         | Language  | Requests/sec |
| ------------------------------------------------- | --------- | ------------ |
| [Hyper 1.7](https://github.com/hyperium/hyper)    | Rust 2024 | 160 000 +    |
| **Routerify-NG (Hyper 1.7)**                      | Rust 2024 | 158 000 +    |
| [Actix-Web 4](https://github.com/actix/actix-web) | Rust 2024 | 150 000 +    |
| [Warp 0.3](https://github.com/seanmonstar/warp)   | Rust 2024 | 145 000 +    |

_(benchmarks vary per system; see [`benchmarks`](https://github.com/guru901/routerify-ng/tree/main/benchmarks) folder)_

---

## Install

Add this to your `Cargo.toml`:

```toml
[dependencies]
routerify_ng = "0.1.0"
hyper = "1.7"
tokio = { version = "1", features = ["full"] }
```

## Example

```rust
use http_body_util::Full;
use hyper::body::Incoming;
use hyper::service::Service;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioExecutor;
use hyper_util::rt::TokioIo;
use hyper_util::server::conn::auto::Builder;
// Import the routerify prelude traits.
use routerify_ng::prelude::*;
use routerify_ng::{Middleware, RequestInfo, Router, RouterService};
use std::sync::Arc;
use std::{convert::Infallible, net::SocketAddr};
use tokio::net::TcpListener;

// Define an app state to share it across the route handlers and middlewares.
struct State(u64);

// A handler for "/" page.
async fn home_handler(req: Request<Incoming>) -> Result<Response<Full<hyper::body::Bytes>>, Infallible> {
    Ok(Response::new(Full::new(hyper::body::Bytes::from("Home page"))))
}

// A handler for "/users/:userId" page.
async fn user_handler(req: Request<Incoming>) -> Result<Response<Full<hyper::body::Bytes>>, Infallible> {
    let user_id = req.param("userId").unwrap();
    Ok(Response::new(Full::new(hyper::body::Bytes::from(format!(
        "Hello {}",
        user_id
    )))))
}

// A middleware which logs an http request.
async fn logger(req: Request<Incoming>) -> Result<Request<Incoming>, Infallible> {
    println!("{} {} {}", req.remote_addr(), req.method(), req.uri().path());
    Ok(req)
}

// Define an error handler function which will accept the `routerify::Error`
// and the request information and generates an appropriate response.
async fn error_handler(err: routerify_ng::RouteError, _: RequestInfo) -> Response<Full<hyper::body::Bytes>> {
    eprintln!("{}", err);
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Full::new(hyper::body::Bytes::from(format!(
            "Something went wrong: {}",
            err
        ))))
        .unwrap()
}

// Create a `Router<Body, Infallible>` for response body type `hyper::Body`
// and for handler error type `Infallible`.
fn router() -> Router<Incoming, Infallible> {
    // Create a router and specify the logger middleware and the handlers.
    // Here, "Middleware::pre" means we're adding a pre middleware which will be executed
    // before any route handlers.
    Router::builder()
        .middleware(Middleware::pre(logger))
        .get("/", home_handler)
        .get("/users/:userId", user_handler)
        .err_handler_with_info(error_handler)
        .build()
        .unwrap()
}

#[tokio::main]
async fn main() {
    let router = router();

    // Create a Service from the router above to handle incoming requests.
    let router_service = Arc::new(RouterService::new(router).unwrap());

    // The address on which the server will be listening.
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let listener = TcpListener::bind(addr).await.unwrap();
    println!("App is running on: {}", addr);

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let router_service = Arc::clone(&router_service);

                tokio::task::spawn(async move {
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
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
}
```

## Documentation

[Docs](https://docs.rs/routerify-ng) for an exhaustive documentation.

## Examples

Find runnable examples in the [examples](/examples/) directory.

## Contributing

PRs, ideas, and suggestions are always welcome!

If you’d like to help maintain Routerify-NG or extend its ecosystem (WebSockets, tower integration, macros, etc.), open an issue or pull request.

## License

Licensed under the MIT License.

> Routerify-NG — keeping Hyper simple, fast, and modern for the next generation of Rust web developers.
