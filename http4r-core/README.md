# http4r

### Core

The main contract and functionality of http4r. 

- Handler, Server and Client
- Writing/reading http messages to/from wire
- Immutable Request, Response, Uri, Headers, Query etc.
- Supports simple messages, streams, compression and chunked encoding
- Coming soon: multipart, x-www-form-urlencoded

### Philosophy

- Simplicity:
  - Prefer a simple synchronous abstraction over http
  - Extension by composition not configuration
- Minimalism: 
  - use as few dependencies as possible to reduce surface area, crate size and upgrade complexity
  - do not publish convenience functions etc. rather share recipes in the docs
- Compatibility:
  - try to maintain backwards compatibility
  - but unlike Rust itself, prefer to break it over keeping a hamstring-ing abstraction
    - to reduce the likelihood of this, do not publish convenience functions as mentioned above!

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


### GPL Copyright

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

## Contributing

### I'm new to Rust or I'm a bit rusty

Look at the contributing guidelines at [http4r](https://http4r.com/docs/contributing)
