use http_body_util::Full;
use hyper::body::Incoming;
use hyper::{Response, body::Bytes};
use routerify_ng::ext::RequestExt;
use routerify_ng::{RouteParams, Router};
use std::convert::Infallible;

fn run() -> Router<Incoming, hyper::Error> {
    let router = Router::builder()
        .get("/users/:userName/books/:bookName", |req| async move {
            let params: &RouteParams = req.params();
            let user_name = params.get("userName").unwrap();
            let book_name = params.get("bookName").unwrap();

            Ok(Response::new(Full::new(Bytes::from(format!(
                "Username: {}, Book Name: {}",
                user_name, book_name
            )))))
        })
        .build()
        .unwrap();
    router
}

fn main() {}
