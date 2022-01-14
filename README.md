## http4r

### I'm new to Rust or I'm a bit rusty

`rustup` to update your rust and cargo version

### About

- http4r is based on [Dan's](https://github.com/bodar/) projects
  [utterlyidle](https://github.com/bodar/utterlyidle)
  and [http-handler.rust](https://github.com/danielbodart/http-handler.rust)
  and [http4t](https://github.com/http4t/http4t) by [Matt](https://github.com/savagematt)
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
  - if version is known to be 1.0 then do not send chunked message
    - stream into memory and forward as content-length 
  - client to use Host header eg Dan's
  - support compression
  - set a body length limit
  - multipart form data
  - todos in http_test eg set-cookie [header special case](https://datatracker.ietf.org/doc/html/rfc6265)
  - should query be separate or part of Uri
  - default response headers, content-type, content-length, date
  - support http/1.0 - ie. read til socket close or EOF
  - support Connection: close
  - refactor to parser combinator
  - bidi routing (can we do this without lenses?)
  - support http/2.0


### Getting started
