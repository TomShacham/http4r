## http4r

### To do

- Prove out concept of SaaF in the browser using WASM
---
- Proper Uri
- Res status message
- http version on message
- support http/1.0 - ie. read til socket close
- support trailers
- support chunked encoding
- multipart form data
- refactor to parser combinator
- JSON body parser
- proper semantics for methods ie. GET has no body etc.
- bidi routing (can we do this without lenses?)
- default response headers, content-type, content-length, date

### About

- http4r is based on [Dan's](https://github.com/bodar/) utterlyidle and http-handler.rust 
- it is a Server as a Function (SaaF) implementation:
  - composable http handlers are just functions `(Request) -> Response`
  - immutable `Request` and `Response`
  - zero magic or reflection, stupidly simple and zero dependencies
  - easily test over the wire or rather in memory