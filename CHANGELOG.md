# Changelog

## [0.2.0] - 2025-11-08

## Added

- Generic body type support across the entire routing system
- Specialized RequestServiceBuilder implementations for Incoming and Empty<Bytes> body types
- Type constraints T: Body on relevant implementations

## Changed

- Router: Changed from Router<E> to Router<T, E> where T is the request body type

  - Example: Router<Infallible> → Router<Incoming, Infallible>

- Middleware: Changed from Middleware<E> to Middleware<T, E>

  - All middleware constructors (pre, post, post_with_info, etc.) now return Middleware<T, E>
  - Handler signatures updated from Request<Incoming> → Request<T>

- Route: Changed from Route<E> to Route<T, E>

  - Internal handler type now generic over body type
  - RouterBuilder: Changed from RouterBuilder<E> to RouterBuilder<T, E>
  - All builder methods now propagate the body type parameter
  - Route handlers accept Request<T> instead of Request<Incoming>

- Services:
  - RequestService<E> → RequestService<T, E>
  - RouterService<E> → RouterService<Incoming, E> (specialized to Incoming)
  - Service trait implementation updated to accept Request<T>

## Documentation

- Updated README with new type signatures and usage patterns
- All code examples updated to reflect Router<Incoming, E> signatures

## [0.1.0] - 2025-11-06

### Added

- Initial release of `routerify_ng`: a lightweight, idiomatic, composable, and modular router implementation for Hyper 1.7.
- Support for hyper HTTP1/HTTP2 with feature flags.
- `Router`, `RouterService`, and `RequestService` abstractions for routing with middleware support.
- Built-in test coverage for request routing.
- Global OPTIONS and default 404 route management.
- Error handling support.
- Percent-encoded URI path handling and request context/meta extraction.
- Extensible and composable request/response handling.
