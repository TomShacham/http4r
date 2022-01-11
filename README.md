## http4r

### I'm new to Rust or I'm a bit rusty

`rustup` to update your rust and cargo version

### About

- http4r is based on [Dan's](https://github.com/bodar/) projects
  [utterlyidle](https://github.com/bodar/utterlyidle)
  and [http-handler.rust](https://github.com/danielbodart/http-handler.rust)
  and is based on [http4k](https://www.http4k.org/) inspired by [Mr Dave](https://github.com/daviddenton) and [Ivan Sanchez](https://github.com/s4nchez)
- it is based on [Server as a Function](https://monkey.org/~marius/funsrv.pdf):
  - composable http handlers implement `(Request) -> Response`
  - immutable `Request`, `Response`, `Headers` etc
  - zero magic or reflection, stupidly simple and zero dependencies
  - easily test over the wire or rather in-memory
  - can test in-browser (not over the wire!) by compiling your app to WASM so
    we can write lightning-fast tests for our front end

### To do

- example app
  - write html file handler with appropriate content-type headers and serve 
  local webpage that loads in wasm app and runs tests
    - routing example using match request { destructuring } which will be badass!
    - even better, do this as puppeteer suite
    - JSON body parser
    - read and write to database
---
- core api
  - test the body fitting in the first read stuff - expose underlying stream read/write in order to test?
  - support chunked encoding
  - support trailers
  - configurable body length limits
  - go through rfc and write test for each bit with comment above each one
  - multipart form data
  - todos in http_test eg set-cookie header special case
  - should query be separate or part of Uri
  - default response headers, content-type, content-length, date
  - support http/1.0 - ie. read til socket close or EOF
  - support http/2.0
  - refactor to parser combinator
  - bidi routing (can we do this without lenses?)


### Getting started
