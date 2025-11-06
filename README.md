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
use hyper::{Body, Request, Response, Server, StatusCode};
use routerify_ng::prelude::*;
use routerify_ng::{Middleware, Router, RouterService, RequestInfo};
use std::{convert::Infallible, net::SocketAddr};

struct State(u64);

async fn home_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let state = req.data::<State>().unwrap();
    println!("State value: {}", state.0);
    Ok(Response::new(Body::from("Home page")))
}

async fn user_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let user_id = req.param("userId").unwrap();
    Ok(Response::new(Body::from(format!("Hello {}", user_id))))
}

async fn logger(req: Request<Body>) -> Result<Request<Body>, Infallible> {
    println!("{} {} {}", req.remote_addr(), req.method(), req.uri().path());
    Ok(req)
}

async fn error_handler(err: routerify_ng::RouteError, _: RequestInfo) -> Response<Body> {
    eprintln!("{}", err);
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::from(format!("Something went wrong: {}", err)))
        .unwrap()
}

fn router() -> Router<Body, Infallible> {
    Router::builder()
        .data(State(100))
        .middleware(Middleware::pre(logger))
        .get("/", home_handler)
        .get("/users/:userId", user_handler)
        .err_handler_with_info(error_handler)
        .build()
        .unwrap()
}

#[tokio::main]
async fn main() {
    let service = RouterService::new(router()).unwrap();
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    println!("App running on: {}", addr);

    if let Err(err) = Server::bind(&addr).serve(service).await {
        eprintln!("Server error: {}", err);
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
