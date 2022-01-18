use std::fmt::{Display, Formatter};
use std::io::{copy, Read, Write};
use std::net::TcpStream;
use std::str;

use crate::headers::{DISALLOWED_TRAILERS, Headers};
use crate::http_message::Body::{BodyStream, BodyString};
use crate::http_message::Method::{CONNECT, DELETE, GET, HEAD, OPTIONS, PATCH, POST, PUT, TRACE};
use crate::http_message::Status::{BadRequest, InternalServerError, LengthRequired, MovedPermanently, NotFound, OK, Unknown};
use crate::uri::Uri;

pub enum HttpMessage<'a> {
    Request(Request<'a>),
    Response(Response<'a>),
}

impl<'a> HttpMessage<'a> {
    pub fn is_req(&self) -> bool {
        match self {
            HttpMessage::Request(_) => true,
            HttpMessage::Response(_) => false
        }
    }

    pub fn is_res(&self) -> bool {
        match self {
            HttpMessage::Request(_) => false,
            HttpMessage::Response(_) => true
        }
    }

    pub fn to_req(self) -> Request<'a> {
        match self {
            HttpMessage::Request(req) => req,
            _ => panic!("Not a request!")
        }
    }
    pub fn to_res(self) -> Response<'a> {
        match self {
            HttpMessage::Response(res) => res,
            _ => panic!("Not a request!")
        }
    }
}

pub fn message_from<'a>(first_read: &'a [u8], stream: TcpStream, first_read_bytes: usize, chunks_vec: &'a mut Vec<u8>, start_line_and_headers_limit: usize) -> Result<HttpMessage<'a>, MessageError> {
    let result = start_line_and_headers_from(first_read, start_line_and_headers_limit);
    if result.is_err() {
        return Err(result.err().unwrap());
    }
    let (end_of_headers_index, start_line, mut headers) = result.ok().unwrap();
    let (part1, part2, part3) = (start_line[0], start_line[1], start_line[2]);
    let is_response = part1.starts_with("HTTP");
    let method_can_have_body = vec!("POST", "PUT", "PATCH", "DELETE").contains(&part1);

    if let Err(e) = check_valid_content_length_or_transfer_encoding(&headers, is_response, method_can_have_body) {
        return Err(e);
    }
    let transfer_encoding = headers.get("Transfer-Encoding");
    if headers.has("Content-Length") && transfer_encoding.is_some() {
        headers = headers.remove("Content-Length");
    }
    let result = body_from(&first_read, stream.try_clone().unwrap(), first_read_bytes, chunks_vec, start_line_and_headers_limit, end_of_headers_index, &headers, part3, !is_response, method_can_have_body, headers.content_length_header(), transfer_encoding);
    if result.is_err() {
        return Err(result.err().unwrap());
    }
    let (body, headers, trailers) = result.unwrap();

    message(part1, part2, part3, is_response, body, headers, trailers)
}

fn message<'a>(part1: &'a str, part2: &'a str, part3: &'a str, is_response: bool, body: Body<'a>, headers: Headers, trailers: Headers) -> Result<HttpMessage<'a>, MessageError> {
    if is_response {
        let (major, minor) = http_version_from(part1);
        Ok(HttpMessage::Response(Response {
            status: Status::from(part2),
            headers,
            body,
            version: HttpVersion { major, minor },
            trailers,
        }))
    } else {
        let (major, minor) = http_version_from(part3);
        Ok(HttpMessage::Request(Request {
            method: Method::from(part1),
            uri: Uri::parse(part2),
            headers,
            body,
            version: HttpVersion { major, minor },
            trailers,
        }))
    }
}

fn body_from<'a>(
    first_read: &'a [u8],
    mut stream: TcpStream,
    first_read_bytes: usize,
    chunks_vec: &'a mut Vec<u8>,
    start_line_and_headers_limit: usize,
    end_of_headers_index: usize,
    headers: &Headers,
    part3: &'a str,
    is_request: bool,
    method_can_have_body: bool,
    content_length: Option<Result<usize, String>>,
    transfer_encoding: Option<String>) -> Result<(Body<'a>, Headers, Headers), MessageError> {
    let mut body;
    let mut trailers = Headers::empty();
    let mut headers = Headers::from_headers(headers);

    if let Some(_encoding) = transfer_encoding {
        let is_version_1_0 = part3 == "HTTP/1.0";
        let result = read_chunked_body_and_trailers(&first_read, &mut stream, first_read_bytes, chunks_vec, start_line_and_headers_limit, end_of_headers_index, &headers, is_request, is_version_1_0);
        if result.is_err() {
            return Err(result.err().unwrap());
        }
        let (total_bytes_read, new_headers, new_trailers) = result.unwrap();
        trailers = new_trailers;
        headers = new_headers;
        body = BodyStream(Box::new(chunks_vec.take(total_bytes_read as u64)));
    } else {
        match content_length {
            Some(_) if is_request && !method_can_have_body => {
                headers = headers.replace(("Content-Length", "0"));
                body = Body::empty()
            }
            // we have read the whole body in the first read
            Some(Ok(content_length)) if first_read_bytes > end_of_headers_index
                && (first_read_bytes - end_of_headers_index) == content_length => {
                let result = str::from_utf8(&first_read[end_of_headers_index..(content_length + end_of_headers_index)]).unwrap();
                body = Body::BodyString(result)
            }
            Some(Ok(content_length)) => {
                if first_read_bytes > end_of_headers_index {
                    let body_so_far = &first_read[end_of_headers_index..first_read_bytes];
                    let body_so_far_size = first_read_bytes - end_of_headers_index;
                    let rest = stream.take(content_length as u64 - body_so_far_size as u64);
                    body = Body::BodyStream(Box::new(body_so_far.chain(rest)));
                } else {
                    let body_stream = stream.take(content_length as u64);
                    body = Body::BodyStream(Box::new(body_stream));
                }
            }
            Some(Err(error)) => {
                return Err(MessageError::InvalidContentLength(format!("Content Length header couldn't be parsed, got {}", error).to_string()));
            }
            _ => body = Body::empty()
        }
    }
    Ok((body, headers, trailers))
}

fn read_chunked_body_and_trailers(
    first_read: &[u8],
    stream: &mut TcpStream,
    first_read_bytes: usize,
    chunks_vec: &mut Vec<u8>,
    start_line_and_headers_limit: usize,
    end_of_headers_index: usize,
    existing_headers: &Headers,
    is_request: bool,
    is_version_1_0: bool,
) -> Result<(usize, Headers, Headers), MessageError> {
    let expected_trailers = existing_headers.get("Trailer");
    let happy_to_receive_trailers = existing_headers.get("TE").map(|t| t.contains("trailers")).unwrap_or(false);
    let cleansed_trailers = expected_trailers
        .map(|ts| ts.split(", ")
            .filter(|t| !DISALLOWED_TRAILERS.contains(&&*t.to_lowercase()))
            .map(|t| t.to_string())
            .collect::<Vec<String>>())
        .unwrap_or(vec!());
    let body_so_far = &first_read[end_of_headers_index..first_read_bytes];

    let result = read_chunks(body_so_far, chunks_vec, "metadata", 0, 0, &cleansed_trailers, start_line_and_headers_limit);
    if result.is_err() {
        return Err(result.err().unwrap());
    }
    let (mut finished, mut in_mode, mut total_bytes_read, mut read_of_current_chunk, mut chunk_size, trailers_first_time) = result.unwrap();
    let mut trailers = trailers_first_time;
    let mut headers = Headers::from_headers(existing_headers);

    while !finished {
        let mut buffer = [0 as u8; 16384];
        let result = stream.read(&mut buffer);
        if result.is_err() { continue; };

        let result = read_chunks(&buffer, chunks_vec, in_mode.as_str(), read_of_current_chunk, chunk_size, &cleansed_trailers, start_line_and_headers_limit);
        if result.is_err() {
            return Err(result.err().unwrap());
        }

        let (is_finished, current_mode, bytes_read, up_to_in_chunk, current_chunk_size, new_trailers) = result.unwrap();

        in_mode = current_mode;
        total_bytes_read += bytes_read;
        read_of_current_chunk = up_to_in_chunk;
        chunk_size = current_chunk_size;
        trailers = new_trailers;
        finished = is_finished;
    }

    if !happy_to_receive_trailers {
        println!("adding trailers {}", trailers.to_wire_string());
        headers = headers.add_all(trailers);
        trailers = Headers::empty();
    }
    // should only be doing this if we are talking to a user agent that does not accept chunked encoding
    // otherwise keep chunked encoding header
    if is_request && is_version_1_0 {
        headers = headers.add(("Content-Length", total_bytes_read.to_string().as_str()))
            .remove("Transfer-Encoding");
    }
    Ok((total_bytes_read, headers, trailers))
}

// todo() return result and err of boundary is fucked and bubble up to a 400 if to_digit doesnt work
fn read_chunks(reader: &[u8], writer: &mut Vec<u8>, last_mode: &str, read_up_to: usize, this_chunk_size: usize, expected_trailers: &Vec<String>, limit: usize) -> Result<(bool, String, usize, usize, usize, Headers), MessageError> {
    let mut prev = vec!('1', '2', '3', '4', '5');
    let mut mode = last_mode;
    let mut chunk_size: usize = this_chunk_size;
    let mut bytes_of_this_chunk_read = read_up_to;
    let mut finished = false;
    let mut bytes_read_from_current_chunk: usize = 0;
    let mut total_bytes_read: usize = 0;
    let mut start_of_trailers: usize = 0;
    let mut trailers = Headers::empty();

    for (index, octet) in reader.iter().enumerate() {
        prev.remove(0);
        prev.push(*octet as char);

        let on_boundary = *octet == b'\n' || *octet == b'\r';
        if mode == "metadata" && !on_boundary {
            // if we have a digit, multiply last digit by 10 and add this one
            // if first digit we encounter is 0 then we'll multiply 0 by 10 and get 0
            // ... and know that we are at the end
            let option = (*octet as char).to_digit(10);
            if option.is_none() {
                println!("tried to digitise {}", *octet as char)
            }
            chunk_size = (chunk_size * 10) + option.unwrap() as usize;
            if chunk_size == 0 { // we have encountered the 0 chunk
                //todo() if at the end of the buffer but we need to read again to get trailers
                finished = true;
                start_of_trailers = index + 3; // add 5 cos we didn't read the \r\n after the 0
                break;
            }
        } else if mode == "metadata" && on_boundary {
            // if we're on the boundary, continue, or change mode to read once we've seen \n
            if *octet == b'\n' {
                mode = "read";
            }
            continue;
        }
        if mode == "read" && bytes_read_from_current_chunk < (chunk_size - bytes_of_this_chunk_read) {
            writer.push(*octet);
            bytes_read_from_current_chunk += 1;
            let last_iteration = index == reader.len() - 1;
            if last_iteration {
                // if last index, add on the bytes read from current chunk
                bytes_of_this_chunk_read += bytes_read_from_current_chunk;
                total_bytes_read += bytes_read_from_current_chunk;
                break;
            }
        } else if mode == "read" && on_boundary {
            // if we're on the boundary, continue, or change mode to metadata once we've seen \n
            // and reset counters
            if *octet == b'\n' {
                total_bytes_read += bytes_read_from_current_chunk;
                bytes_of_this_chunk_read = 0;
                chunk_size = 0;
                bytes_read_from_current_chunk = 0;
                mode = "metadata";
            }
            continue;
        }
    }

    //todo() what if you need to do another read to get all the trailers?
    // finished should be false until we finish reading trailers
    if finished && !expected_trailers.is_empty() {
        let dummy_request_line_and_trailers = ["GET / HTTP/1.1\r\n".as_bytes(), &reader[start_of_trailers..]].concat();
        if let Ok((_, _, headers)) = start_line_and_headers_from(dummy_request_line_and_trailers.as_slice(), limit) {
            trailers = headers.filter(expected_trailers.iter().map(|s| s as &str).collect())
        }
    }

    Ok((finished, mode.to_string(), total_bytes_read, bytes_of_this_chunk_read, chunk_size, trailers))
}

fn check_valid_content_length_or_transfer_encoding(headers: &Headers, is_response: bool, method_can_have_body: bool) -> Result<(), MessageError> {
    let is_req_and_method_can_have_body = !is_response && method_can_have_body;
    let no_content_length_or_transfer_encoding = !headers.has("Content-Length") &&
        headers.get("Transfer-Encoding").is_none();

    if (is_req_and_method_can_have_body && no_content_length_or_transfer_encoding)
        || is_response && no_content_length_or_transfer_encoding {
        return Err(MessageError::NoContentLengthOrTransferEncoding("Content-Length or Transfer-Encoding must be provided".to_string()));
    }
    Ok(())
}

fn start_line_and_headers_from(buffer: &[u8], limit: usize) -> Result<(usize, Vec<&str>, Headers), MessageError> {
    let mut prev: Vec<char> = vec!('1', '2', '3', '4');
    let mut end_of_headers_index = 0;
    let mut end_of_start_line = 0;
    let mut first_line = None;
    let mut headers = None;
    let mut finished = false;

    for (index, octet) in buffer.iter().enumerate() {
        if first_line.is_none() && prev[3] == '\r' && *octet == b'\n' {
            first_line = Some(&buffer[..index - 1]);
            end_of_start_line = index + 1;
        }
        if !first_line.is_none() && prev[1] == '\r' && prev[2] == '\n' && prev[3] == '\r' && *octet == b'\n' {
            let end_of_headers = index - 3; // end is behind the \r\n\r\n chars (ie back 4)
            if end_of_start_line > end_of_headers {
                headers = None
            } else {
                headers = Some(&buffer[end_of_start_line..end_of_headers]);
            }
            finished = true;
            end_of_headers_index = index + 1;
            break;
        }
        if index > limit {
            // todo() do one more read??? or allow user to set a limit on headers/trailers?
            return Err(MessageError::HeadersTooBig(format!("Headers must be less than {}", buffer.len())));
        }

        prev.remove(0);
        prev.push(*octet as char);
    }
    let header_string = if headers.is_none() { "" } else {
        str::from_utf8(&headers.unwrap()).unwrap()
    };

    let headers = Headers::parse_from(header_string);
    let start_line = str::from_utf8(&first_line.unwrap()).unwrap().split(" ").collect::<Vec<&str>>();

    Ok((end_of_headers_index, start_line, headers))
}

pub fn write_body(mut stream: &mut TcpStream, message: HttpMessage) {
    match message {
        HttpMessage::Request(mut req) => {
            let has_transfer_encoding = req.headers.has("Transfer-Encoding");
            let headers = ensure_content_length_or_transfer_encoding(&req.headers, &req.body, has_transfer_encoding, &req.version)
                .unwrap_or(req.headers);

            let mut request_string = format!("{} {} HTTP/{}.{}\r\n{}\r\n\r\n",
                                             req.method.value(),
                                             req.uri.to_string(),
                                             req.version.major,
                                             req.version.minor,
                                             headers.to_wire_string());

            match req.body {
                BodyString(str) => {
                    if has_transfer_encoding && req.version == one_pt_one() {
                        write_chunked_string(stream, request_string, str, req.trailers);
                    } else {
                        request_string.push_str(str);
                        stream.write(request_string.as_bytes()).unwrap();
                    }
                }
                BodyStream(ref mut reader) => {
                    if has_transfer_encoding && req.version == one_pt_one() {
                        write_chunked_stream(stream, reader, request_string, req.trailers);
                    } else {
                        let mut chain = request_string.as_bytes().chain(reader);
                        let _copy = copy(&mut chain, &mut stream).unwrap();
                    }
                }
            }
        }
        HttpMessage::Response(mut res) => {
            let has_transfer_encoding = res.headers.has("Transfer-Encoding");
            let headers = ensure_content_length_or_transfer_encoding(&res.headers, &res.body, has_transfer_encoding, &res.version)
                .unwrap_or(res.headers);
            let mut status_and_headers: String = Response::status_line_and_headers_wire_string(&headers, &res.status);
            let chunked_encoding_desired = headers.has("Transfer-Encoding");

            match res.body {
                BodyString(body_string) => {
                    if chunked_encoding_desired && res.version == one_pt_one() {
                        write_chunked_string(stream, status_and_headers, body_string, res.trailers);
                    } else {
                        status_and_headers.push_str(body_string);
                        stream.write(status_and_headers.as_bytes()).unwrap();
                        if !res.trailers.is_empty() {
                            stream.write(format!("\r\n{}\r\n\r\n", res.trailers.to_wire_string()).as_bytes()).unwrap();
                        }
                    }
                }
                BodyStream(ref mut reader) => {
                    if chunked_encoding_desired && res.version == one_pt_one() {
                        write_chunked_stream(&mut stream, reader, status_and_headers, res.trailers);
                    } else {
                        let mut chain = status_and_headers.as_bytes().chain(reader);
                        let _copy = copy(&mut chain, &mut stream).unwrap();
                    }
                }
            }
        }
    }
}

pub fn write_chunked_string(stream: &mut TcpStream, mut first_line: String, chunk: &str, trailers: Headers) {
    let length = chunk.len().to_string();
    let end = "0\r\n";

    first_line.push_str(length.as_str());
    first_line.push_str("\r\n");
    first_line.push_str(chunk);
    first_line.push_str("\r\n");
    first_line.push_str(end);

    if !trailers.is_empty() {
        first_line.push_str(format!("{}\r\n\r\n", trailers.to_wire_string()).as_str());
    }

    stream.write(first_line.as_bytes()).unwrap();
}


pub fn write_chunked_stream<'a>(mut stream: &mut TcpStream, reader: &mut Box<dyn Read + 'a>, first_line_and_headers: String, trailers: Headers) {
    let buffer = &mut [0 as u8; 16384];
    let mut bytes_read = reader.read(buffer).unwrap_or(0);
    let mut first_write = true;

    while bytes_read > 0 {
        let mut temp = vec!();
        temp.extend_from_slice(bytes_read.to_string().as_bytes());
        temp.push(b'\r');
        temp.push(b'\n');
        temp.append(&mut buffer[..bytes_read].to_vec());
        temp.push(b'\r');
        temp.push(b'\n');

        // write to wire
        if first_write {
            let first_line_and_headers_and_first_chunk = [first_line_and_headers.as_bytes(), temp.as_slice()].concat();
            let _copy = copy(&mut first_line_and_headers_and_first_chunk.as_slice(), &mut stream).unwrap();
            first_write = false;
        } else {
            let _copy = copy(&mut temp.as_slice(), &mut stream).unwrap();
        }

        bytes_read = reader.read(buffer).unwrap_or(0);
    }
    // write end byte
    let mut end = vec!(b'0', b'\r', b'\n');
    if !trailers.is_empty() {
        end.extend_from_slice(format!("{}\r\n\r\n", trailers.to_wire_string()).as_bytes());
    }
    stream.write(end.as_slice()).unwrap();
}

pub fn ensure_content_length_or_transfer_encoding(headers: &Headers, body: &Body, has_transfer_encoding: bool, version: &HttpVersion) -> Option<Headers> {
    let has_content_length = headers.has("Content-Length");
    if !has_content_length && !has_transfer_encoding && body.is_body_string() {
        Some(headers.add(("Content-Length", body.length().to_string().as_str())))
    } else if !has_content_length && !has_transfer_encoding && body.is_body_stream() && version == &one_pt_one() {
        Some(headers.add(("Transfer-Encoding", "chunked")))
    } else {
        None
    }
}

fn http_version_from(str: &str) -> (u8, u8) {
    let mut version_chars = str.chars();
    let major = version_chars.nth(0);
    let minor = version_chars.nth(2);
    (major.unwrap() as u8, minor.unwrap() as u8)
}

#[derive(Debug)]
pub enum MessageError {
    InvalidContentLength(String),
    NoContentLengthOrTransferEncoding(String),
    HeadersTooBig(String),
}

impl Display for MessageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl<'a> Response<'a> {
    pub fn status_line_and_headers_wire_string(headers: &Headers, status: &Status) -> String {
        let mut response = String::new();
        let http = format!("HTTP/1.1 {} {}", &status.value(), &status.to_string());
        response.push_str(&http);
        response.push_str("\r\n");

        response.push_str(&headers.to_wire_string().as_str());

        response.push_str("\r\n\r\n");
        response
    }
}

pub enum Body<'a> {
    BodyString(&'a str),
    BodyStream(Box<dyn Read + 'a>),
}

impl<'a> Body<'a> {
    pub fn empty() -> Body<'a> {
        BodyString("")
    }

    pub fn is_body_string(&self) -> bool {
        match self {
            BodyString(_) => true,
            BodyStream(_) => false
        }
    }

    pub fn is_body_stream(&self) -> bool {
        match self {
            BodyString(_) => false,
            BodyStream(_) => true
        }
    }

    pub fn length(&self) -> usize {
        match self {
            BodyString(str) => str.len(),
            BodyStream(_) => panic!("Do not know the length of a body stream!")
        }
    }
}

pub fn body_length(body: &Body) -> u32 {
    match body {
        BodyString(str) => str.len() as u32,
        BodyStream(_) => panic!("Cannot find length of BodyStream, please provide Content-Length header")
    }
}

pub fn with_content_length(message: HttpMessage) -> HttpMessage {
    match message {
        HttpMessage::Request(request) => {
            if !request.headers.has("Content-Length") && !request.headers.has("Transfer-Encoding") {
                return HttpMessage::Request(Request {
                    headers: request.headers.add(("Content-Length", body_length(&request.body).to_string().as_str())),
                    ..request
                });
            } else {
                HttpMessage::Request(request)
            }
        }
        HttpMessage::Response(response) => {
            if !response.headers.has("Content-Length") && !response.headers.has("Transfer-Encoding") {
                return HttpMessage::Response(Response {
                    headers: response.headers.add(("Content-Length", body_length(&response.body).to_string().as_str())),
                    ..response
                });
            } else {
                HttpMessage::Response(response)
            }
        }
    }
}

#[derive(PartialEq)]
pub struct HttpVersion {
    pub major: u8,
    pub minor: u8,
}

pub fn one_pt_one() -> HttpVersion {
    HttpVersion { major: 1, minor: 1 }
}

pub fn two_pt_oh() -> HttpVersion {
    HttpVersion { major: 2, minor: 0 }
}

pub fn one_pt_oh() -> HttpVersion {
    HttpVersion { major: 1, minor: 0 }
}

pub struct Request<'a> {
    pub headers: Headers,
    pub body: Body<'a>,
    pub uri: Uri<'a>,
    pub method: Method,
    pub version: HttpVersion,
    pub trailers: Headers,
}

pub struct Response<'a> {
    pub headers: Headers,
    pub body: Body<'a>,
    pub status: Status,
    pub version: HttpVersion,
    pub trailers: Headers,
}


impl<'a> Request<'a> {
    pub fn with_body(self, body: Body<'a>) -> Request<'a> {
        Request {
            body,
            ..self
        }
    }

    pub fn with_header(self, pair: (&str, &str)) -> Request<'a> {
        Request {
            headers: self.headers.add(pair),
            ..self
        }
    }

    pub fn with_trailers(self, trailers: Headers) -> Request<'a> {
        Request {
            trailers,
            ..self
        }
    }
}

pub fn body_string(mut body: Body) -> String {
    match body {
        BodyString(str) => str.to_string(),
        BodyStream(ref mut reader) => {
            let big = &mut Vec::new();
            let _read_bytes = reader.read_to_end(big).unwrap(); //todo() this blows up sometimes! unwrap_or()?
            str::from_utf8(&big).unwrap()
                .trim_end_matches(char::from(0))
                .to_string()
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum Method {
    GET,
    CONNECT,
    HEAD,
    POST,
    OPTIONS,
    DELETE,
    PATCH,
    PUT,
    TRACE,
}

impl Method {
    pub fn value(&self) -> String {
        match self {
            GET => String::from("GET"),
            POST => String::from("POST"),
            PATCH => String::from("PATCH"),
            OPTIONS => String::from("OPTIONS"),
            CONNECT => String::from("CONNECT"),
            HEAD => String::from("HEAD"),
            DELETE => String::from("DELETE"),
            PUT => String::from("PUT"),
            TRACE => String::from("TRACE")
        }
    }

    pub fn from(str: &str) -> Method {
        match str {
            "GET" => GET,
            "POST" => POST,
            "PATCH" => PATCH,
            "OPTIONS" => OPTIONS,
            "DELETE" => DELETE,
            "CONNECT" => CONNECT,
            "TRACE" => TRACE,
            "HEAD" => HEAD,
            _ => panic!("Unknown method")
        }
    }
}

impl<'a> Response<'a> {
    pub fn ok(headers: Headers, body: Body) -> Response {
        Response { headers, body, status: OK, version: HttpVersion { major: 1, minor: 1 }, trailers: Headers::empty() }
    }

    pub fn bad_request(headers: Headers, body: Body) -> Response {
        Response { headers, body, status: BadRequest, version: HttpVersion { major: 1, minor: 1 }, trailers: Headers::empty() }
    }

    pub fn internal_server_error(headers: Headers, body: Body) -> Response {
        Response { headers, body, status: InternalServerError, version: HttpVersion { major: 1, minor: 1 }, trailers: Headers::empty() }
    }

    pub fn length_required(headers: Headers, body: Body) -> Response {
        Response { headers, body, status: LengthRequired, version: HttpVersion { major: 1, minor: 1 }, trailers: Headers::empty() }
    }

    pub fn not_found(headers: Headers, body: Body) -> Response {
        Response { headers, body, status: NotFound, version: HttpVersion { major: 1, minor: 1 }, trailers: Headers::empty() }
    }

    pub fn moved_permanently(headers: Headers, body: Body) -> Response {
        Response { headers, body, status: MovedPermanently, version: HttpVersion { major: 1, minor: 1 }, trailers: Headers::empty() }
    }

    pub fn with_trailers(self, trailers: Headers) -> Response<'a> {
        Response {
            trailers,
            ..self
        }
    }

    pub fn with_headers(self, headers: Headers) -> Response<'a> {
        Response {
            headers: self.headers.add_all(headers),
            ..self
        }
    }
}

impl<'a> Request<'a> {
    pub fn request(method: Method, uri: Uri, headers: Headers) -> Request {
        Request { method, headers, body: Body::empty(), uri, version: HttpVersion { major: 1, minor: 1 }, trailers: Headers::empty() }
    }

    pub fn get(uri: Uri, headers: Headers) -> Request {
        Request { method: GET, headers, body: Body::empty(), uri, version: HttpVersion { major: 1, minor: 1 }, trailers: Headers::empty() }
    }

    pub fn post(uri: Uri<'a>, headers: Headers, body: Body<'a>) -> Request<'a> {
        Request { method: POST, headers, body, uri, version: HttpVersion { major: 1, minor: 1 }, trailers: Headers::empty() }
    }
}


#[derive(PartialEq, Debug)]
#[repr(u32)]
pub enum Status {
    OK = 200,
    MovedPermanently = 301,
    BadRequest = 400,
    LengthRequired = 411,
    NotFound = 404,
    InternalServerError = 500,
    Unknown = 0,
}

impl Status {
    pub fn to_string(&self) -> String {
        match self {
            OK => "OK".to_string(),
            MovedPermanently => "Moved Permanently".to_string(),
            NotFound => "Not Found".to_string(),
            BadRequest => "Bad Request".to_string(),
            InternalServerError => "Internal Server Error".to_string(),
            _ => "Unknown".to_string()
        }
    }
    pub fn value(&self) -> u32 {
        match self {
            OK => 200,
            MovedPermanently => 301,
            BadRequest => 400,
            NotFound => 404,
            InternalServerError => 500,
            _ => 500
        }
    }
    pub fn from(str: &str) -> Self {
        match str.to_lowercase().as_str() {
            "200" => OK,
            "400" => BadRequest,
            "404" => NotFound,
            _ => Unknown
        }
    }
}

