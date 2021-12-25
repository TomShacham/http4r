use std::io::Read;
use std::net::TcpStream;
use std::str;
use crate::http_message::Body::{BodyStream, BodyString};
use crate::http_message::Method::{DELETE, GET, OPTIONS, PATCH, POST};
use crate::http_message::Status::{NotFound, OK, Unknown, InternalServerError, BadRequest, LengthRequired, MovedPermanently};


type Headers = Vec<Header>;

pub fn header(headers: &Headers, name: &str) -> Option<Header> {
    for header in headers {
        if header.0.to_lowercase() == name.to_lowercase() {
            return Some(header.clone());
        }
    }
    None
}

pub fn headers_to_string(headers: &Headers) -> String {
    headers.iter().map(|h| {
        let clone = h.clone();
        clone.0 + ": " + clone.1.as_str()
    }).collect::<Vec<String>>()
        .join("\r\n")
}

pub fn add_header(headers: &Headers, header: Header) -> Headers {
    let mut new = vec!();
    let mut exists = false;
    for h in headers {
        if h.0 == header.0 {
            new.push((h.clone().0, h.clone().1 + ", " + header.1.as_str()));
            exists = true
        } else {
            new.push(h.clone())
        }
    }
    if !exists {
        new.push(header)
    }
    return new;
}

pub type Header = (String, String);

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
    let mut snip = 0;
    let mut first_line = None;
    let mut headers = None;
    for char in buffer {
        if first_line.is_none() && prev[2] as char == '\r' && prev[3] as char == '\n' {
            first_line = Some(&buffer[..pre_body_index]);
            snip = pre_body_index;
        }
        if !first_line.is_none() && prev.iter().collect::<String>() == "\r\n\r\n" {
            headers = Some(&buffer[snip..pre_body_index - prev.len()]);
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
    let header_string = str::from_utf8(&headers.unwrap()).unwrap();
    let headers = parse_headers(header_string);

    if header(&headers, "Content-Length").is_none() &&
        header(&headers, "Transfer-Encoding").is_none() {
        return Err(MessageError::NoContentLengthOrTransferEncoding("Content-Length or Transfer-Encoding must be provided".to_string()));
    }

    // todo() support trailers

    let body;
    let content_length = content_length_header(&headers);
    match content_length {
        Some(content_length) if content_length + pre_body_index <= buffer.len() => {
            let result = str::from_utf8(&buffer[pre_body_index..(content_length + pre_body_index)]).unwrap().to_string();
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
        _ => body = Body::BodyString("".to_string())
    }

    if part1.starts_with("HTTP") {
        Ok(HttpMessage::Response(Response {
            status: Status::from(part2),
            headers,
            body,
        }))
    } else {
        Ok(HttpMessage::Request(Request {
            method: Method::from(part1.to_string()),
            uri: part2.to_string(),
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

fn parse_headers(header_string: &str) -> Vec<Header> {
    let mut headers = vec!();
    header_string.split("\r\n").for_each(|pair| {
        let pair = pair.split(": ").collect::<Vec<&str>>();
        headers = add_header(&headers, (pair[0].to_string(), pair[1].to_string()));
    });
    headers
}

pub fn content_length_header(headers: &Vec<Header>) -> Option<usize> {
    header(&headers, "Content-Length").map(|x| { x.1.parse().unwrap() })
}

impl<'a> Response<'a> {
    pub fn resource_and_headers(&self) -> String {
        let mut response = String::new();
        let http = format!("HTTP/1.1 {} {}", &self.status.to_string(), &self.status.value());
        response.push_str(&http);
        response.push_str("\r\n");
        for (name, value) in &self.headers {
            let header_string = format!("{}: {}", name, value);
            response.push_str(&header_string);
            response.push_str("\r\n");
        }
        response.push_str("\r\n");
        response
    }
}

pub enum Body<'a> {
    BodyString(String),
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
            if header(&request.headers, "Content-Length").is_none() {
                return HttpMessage::Request(Request {
                    headers: add_header(&request.headers, ("Content-Length".to_string(), body_length(&request.body).to_string())),
                    ..request
                });
            } else {
                HttpMessage::Request(request)
            }
        }
        HttpMessage::Response(response) => {
            if header(&response.headers, "Content-Length").is_none() {
                return HttpMessage::Response(Response {
                    headers: add_header(&response.headers, ("Content-Length".to_string(), body_length(&response.body).to_string())),
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
    pub uri: String,
    pub method: Method,
}

pub struct Response<'a> {
    pub headers: Headers,
    pub body: Body<'a>,
    pub status: Status,
}

pub fn body_string(mut body: Body) -> String {
    match body {
        BodyString(str) => str,
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
    POST,
    OPTIONS,
    DELETE,
    PATCH,
}

impl Method {
    pub fn value(&self) -> String {
        match self {
            GET => String::from("GET"),
            POST => String::from("POST"),
            PATCH => String::from("PATCH"),
            OPTIONS => String::from("OPTIONS"),
            DELETE => String::from("DELETE"),
        }
    }

    pub(crate) fn from(str: String) -> Method {
        match str.as_str() {
            "GET" => GET,
            "POST" => POST,
            "PATCH" => PATCH,
            "OPTIONS" => OPTIONS,
            "DELETE" => DELETE,
            _ => panic!("Unknown method")
        }
    }
}

pub fn ok(headers: Vec<(String, String)>, body: Body) -> Response {
    Response { headers, body, status: OK }
}

pub fn bad_request(headers: Vec<(String, String)>, body: Body) -> Response {
    Response { headers, body, status: BadRequest }
}

pub fn internal_server_error(headers: Vec<(String, String)>, body: Body) -> Response {
    Response { headers, body, status: InternalServerError }
}

pub fn length_required(headers: Vec<(String, String)>, body: Body) -> Response {
    Response { headers, body, status: LengthRequired }
}

pub fn not_found(headers: Vec<(String, String)>, body: Body) -> Response {
    Response { headers, body, status: NotFound }
}

pub fn moved_permanently(headers: Vec<(String, String)>, body: Body) -> Response {
    Response { headers, body, status: MovedPermanently }
}

pub fn get<'a>(uri: String, headers: Vec<(String, String)>) -> Request<'a> {
    Request { method: GET, headers, body: BodyString("".to_string()), uri }
}

pub fn post<'a>(uri: String, headers: Vec<(String, String)>, body: Body<'a>) -> Request<'a> {
    Request { method: POST, headers, body, uri }
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

