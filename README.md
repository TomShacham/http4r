## http4r

### To do

- write html file handler with appropriate content-type headers and serve 
local webpage that loads in wasm app and runs tests
  - even better, do this as puppeteer suite
  - JSON body parser

---
- Proper Uri
- Split server new and handle
  - make a test server and a threadpool server
- Response status message
- http version on message
- support http/1.0 - ie. read til socket close
- support trailers
- support chunked encoding
- multipart form data
- refactor to parser combinator
- proper semantics for methods ie. GET has no body etc.
- bidi routing (can we do this without lenses?)
- default response headers, content-type, content-length, date

### About

- http4r is based on [Dan's](https://github.com/bodar/) projects [utterlyidle](https://github.com/bodar/utterlyidle) and [http-handler.rust](https://github.com/danielbodart/http-handler.rust) 
- it is based on [Server as a Function](https://monkey.org/~marius/funsrv.pdf):
  - composable http handlers implement `(Request) -> Response`
  - immutable `Request`, `Response`, `Headers` etc
  - zero magic or reflection, stupidly simple and zero dependencies
  - easily test over the wire or rather in-memory
  - can test in-browser (not over the wire!) by compiling your app to WASM so 
  we can write lightning-fast tests for our front end


### Getting started
