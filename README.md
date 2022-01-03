## http4r

### About

- http4r is based on [Dan's](https://github.com/bodar/) projects
  [utterlyidle](https://github.com/bodar/utterlyidle)
  and [http-handler.rust](https://github.com/danielbodart/http-handler.rust)
  and [http4k](https://www.http4k.org/) 
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
  - http version on message
  - should query be separate or part of Uri
  - default response headers, content-type, content-length, date
  - support http/1.0 - ie. read til socket close or EOF
  - support http/2.0
  - support trailers
  - support chunked encoding
  - multipart form data
  - refactor to parser combinator
  - bidi routing (can we do this without lenses?)


### Getting started
