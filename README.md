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

http4r is a web toolkit
Copyright (C) 2021-onwards Tom Shacham

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program (see COPYING).  If not, see <https://www.gnu.org/licenses/>.


### Todos

- example app
  - write html file handler with appropriate content-type headers and serve 
  - local webpage that loads in wasm app and runs tests
    - JSON body parser
    - read and write to database
---
- core api
  - limits on headers and body etc tests 
  - if version is known to be 1.0 then do not send chunked message
    - stream into memory and forward as content-length 
  - multipart form data
  - client to use Host header eg Dan's
  - support compression
  - set a body length limit
  - todos in http_test eg set-cookie [header special case](https://datatracker.ietf.org/doc/html/rfc6265)
  - default response headers, content-type, content-length, date
  - support http/1.0 - ie. read til socket close or EOF
    - support Connection: close
  - refactor to parser combinator??
  - bidi routing (can we do this without lenses?)
  - support http/2.0


### Getting started
