use std::io::Read;
use std::net::TcpStream;
use std::str;
use crate::headers::{Headers};
use crate::http_message::Body::{BodyStream, BodyString};
use crate::http_message::Method::{CONNECT, DELETE, GET, HEAD, OPTIONS, PATCH, POST, PUT, TRACE};
use crate::http_message::Status::{BadRequest, InternalServerError, LengthRequired, MovedPermanently, NotFound, OK, Unknown};
use crate::uri::Uri;

pub enum HttpMessage<'a> {
    Request(Request<'a>),
    Response(Response<'a>),
}

impl<'a> HttpMessage<'a> {
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

pub fn message_from(buffer: &[u8], stream: TcpStream, first_read: usize) -> Result<HttpMessage, MessageError> {
    let mut prev: Vec<char> = vec!('1', '2', '3', '4');
    let mut pre_body_index = 0;
    let mut end_of_start_line = 0;
    let mut first_line = None;
    let mut headers = None;
    for char in buffer {
        if first_line.is_none() && prev[2] as char == '\r' && prev[3] as char == '\n' {
            first_line = Some(&buffer[..pre_body_index]);
            end_of_start_line = pre_body_index;
        }
        if !first_line.is_none() && prev.iter().collect::<String>() == "\r\n\r\n" {
            let end_of_headers = pre_body_index - prev.len();
            if end_of_start_line > end_of_headers {
                headers = None
            } else {
                headers = Some(&buffer[end_of_start_line..end_of_headers]);
            }
            break;
        }
        prev.remove(0);
        prev.push(*char as char);
        if pre_body_index > buffer.len() {
            return Err(MessageError::HeadersTooBig(format!("Headers must be less than {}", buffer.len())));
        }
        pre_body_index += 1;
    }
    let request_line = str::from_utf8(&first_line.unwrap()).unwrap().split(" ").collect::<Vec<&str>>();

    let (part1, part2, _part3) = (request_line[0], request_line[1], request_line[2]);
    let header_string = if headers.is_none() { "" } else {
        str::from_utf8(&headers.unwrap()).unwrap()
    };
    let mut headers = Headers::parse_from(header_string);

    let is_response = part1.starts_with("HTTP");
    let method_can_have_body = vec!("POST", "PUT", "PATCH", "DELETE").contains(&part1);
    let is_req_and_method_can_have_body = !is_response && method_can_have_body;
    let is_req_and_method_cannot_have_body = !is_response && !method_can_have_body;
    let no_content_length_or_transfer_encoding = headers.content_length_header().is_none() &&
        headers.get("Transfer-Encoding").is_none();

    if (is_req_and_method_can_have_body && no_content_length_or_transfer_encoding)
        || is_response && no_content_length_or_transfer_encoding {
        return Err(MessageError::NoContentLengthOrTransferEncoding("Content-Length or Transfer-Encoding must be provided".to_string()));
    }

    // todo() support trailers

    let body;
    let content_length = headers.content_length_header();
    match content_length {
        Some(_) if is_req_and_method_cannot_have_body => {
            headers = headers.replace_header(("Content-Length", "0"));
            body = Body::BodyString("")
        }
        Some(content_length) if content_length + pre_body_index <= buffer.len() => {
            let result = str::from_utf8(&buffer[pre_body_index..(content_length + pre_body_index)]).unwrap();
            body = Body::BodyString(result)
        }
        Some(content_length) => {
            if first_read > pre_body_index {
                let body_so_far = &buffer[pre_body_index..first_read];
                let body_so_far_size = first_read - pre_body_index;
                let rest = stream.take(content_length as u64 - body_so_far_size as u64);
                body = Body::BodyStream(Box::new(body_so_far.chain(rest)));
            } else {
                let body_stream = stream.take(content_length as u64);
                body = Body::BodyStream(Box::new(body_stream));
            }
        }
        _ => body = Body::BodyString("")
    }

    if is_response {
        Ok(HttpMessage::Response(Response {
            status: Status::from(part2),
            headers,
            body,
        }))
    } else {
        Ok(HttpMessage::Request(Request {
            method: Method::from(part1),
            uri: Uri::parse(part2),
            headers,
            body,
        }))
    }
}

#[derive(Debug)]
pub enum MessageError {
    NoContentLengthOrTransferEncoding(String),
    HeadersTooBig(String),
}

impl<'a> Response<'a> {
    pub fn status_line_and_headers(&self) -> String {
        let mut response = String::new();
        let http = format!("HTTP/1.1 {} {}", &self.status.to_string(), &self.status.value());
        response.push_str(&http);
        response.push_str("\r\n");

        response.push_str(&self.headers.to_wire_string().as_str());

        response.push_str("\r\n\r\n");
        response
    }
}

pub enum Body<'a> {
    BodyString(&'a str),
    BodyStream(Box<dyn Read + 'a>),
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
            if request.headers.content_length_header().is_none() {
                return HttpMessage::Request(Request {
                    headers: request.headers.add_header(("Content-Length", body_length(&request.body).to_string().as_str())),
                    ..request
                });
            } else {
                HttpMessage::Request(request)
            }
        }
        HttpMessage::Response(response) => {
            if response.headers.content_length_header().is_none() {
                return HttpMessage::Response(Response {
                    headers: response.headers.add_header(("Content-Length", body_length(&response.body).to_string().as_str())),
                    ..response
                });
            } else {
                HttpMessage::Response(response)
            }
        }
    }
}

pub struct Request<'a> {
    pub headers: Headers,
    pub body: Body<'a>,
    pub uri: Uri<'a>,
    pub method: Method,
}

pub struct Response<'a> {
    pub headers: Headers,
    pub body: Body<'a>,
    pub status: Status,
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
            headers: self.headers.add_header(pair),
            ..self
        }
    }
}

pub fn body_string(mut body: Body) -> String {
    match body {
        BodyString(str) => str.to_string(),
        BodyStream(ref mut reader) => {
            let big = &mut Vec::new();
            let _read_bytes = reader.read_to_end(big).unwrap();
            str::from_utf8(&big).unwrap().trim_end_matches(char::from(0)).to_string()
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
        Response { headers, body, status: OK }
    }

    pub fn bad_request(headers: Headers, body: Body) -> Response {
        Response { headers, body, status: BadRequest }
    }

    pub fn internal_server_error(headers: Headers, body: Body) -> Response {
        Response { headers, body, status: InternalServerError }
    }

    pub fn length_required(headers: Headers, body: Body) -> Response {
        Response { headers, body, status: LengthRequired }
    }

    pub fn not_found(headers: Headers, body: Body) -> Response {
        Response { headers, body, status: NotFound }
    }

    pub fn moved_permanently(headers: Headers, body: Body) -> Response {
        Response { headers, body, status: MovedPermanently }
    }
}

impl<'a> Request<'a> {
    pub fn request(method: Method, uri: Uri, headers: Headers) -> Request {
        Request { method, headers, body: BodyString(""), uri }
    }

    pub fn get(uri: Uri, headers: Headers) -> Request {
        Request { method: GET, headers, body: BodyString(""), uri }
    }

    pub fn post(uri: Uri<'a>, headers: Headers, body: Body<'a>) -> Request<'a> {
        Request { method: POST, headers, body, uri }
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
            "ok" => OK,
            "not found" => NotFound,
            _ => Unknown
        }
    }
}

