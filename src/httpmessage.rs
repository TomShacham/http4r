use std::io::Read;
use std::net::TcpStream;
use std::str;
use crate::httpmessage::Body::{BodyStream, BodyString};
use crate::httpmessage::Method::{DELETE, GET, OPTIONS, PATCH, POST};
use crate::httpmessage::Status::{NotFound, OK, Unknown, InternalServerError, BadRequest, LengthRequired, MovedPermanently};


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
    pub fn as_res(self) -> Response<'a> {
        match self {
            HttpMessage::Response(res) => res,
            _ => panic!("Not a response")
        }
    }
    pub fn as_req(self) -> Request<'a> {
        match self {
            HttpMessage::Request(req) => req,
            _ => panic!("Not a request")
        }
    }
}

pub fn request_from(buffer: &[u8], mut stream: TcpStream, first_read: usize) -> Result<Request, RequestError> {
    let mut prev: Vec<char> = vec!('1', '2', '3', '4');
    let mut pre_body_index = 0;
    let mut snip = 0;
    let mut request_line = None;
    let mut headers = None;
    for char in buffer {
        if request_line.is_none() && prev[2] as char == '\r' && prev[3] as char == '\n' {
            request_line = Some(&buffer[..pre_body_index]);
            snip = pre_body_index;
        }
        if !request_line.is_none() && prev.iter().collect::<String>() == "\r\n\r\n" {
            headers = Some(&buffer[snip..pre_body_index - prev.len()]);
            break;
        }
        prev.remove(0);
        prev.push(*char as char);
        if pre_body_index > buffer.len() {
            return Err(RequestError::HeadersTooBig(format!("Headers must be less than {}", buffer.len())));
        }
        pre_body_index += 1;
    }
    let request_line = str::from_utf8(&request_line.unwrap()).unwrap().split(" ").collect::<Vec<&str>>();

    let (method, uri, _http_version) = (request_line[0], request_line[1], request_line[2]);
    let header_string = str::from_utf8(&headers.unwrap()).unwrap();
    let headers = parse_headers(header_string);

    if header(&headers, "Content-Length").is_none() &&
        header(&headers, "Transfer-Encoding").is_none() {
        return Err(RequestError::NoContentLengthOrTransferEncoding("Content-Length or Transfer-Encoding must be provided".to_string()));
    }

    // todo() support trailers

    let body;
    let content_length: Option<usize> = content_length_header(&headers);
    println!("request from content length {} {} {}, {}", content_length.unwrap(), pre_body_index, buffer.len(), content_length.unwrap() + pre_body_index <= buffer.len());
    match content_length {
        Some(content_length) if content_length + pre_body_index <= buffer.len() => {
            let result = str::from_utf8(&buffer[pre_body_index..(content_length + pre_body_index)]).unwrap().to_string();
            body = Body::BodyString(result)
        }
        Some(content_length) => {
            let body_stream = stream.take(content_length as u64);
            println!("stream.take request {}", content_length);
            body = Body::BodyStream(Box::new(body_stream));
        }
        _ => body = Body::BodyString("".to_string())
    }

    let request = Request {
        method: Method::from(method.to_string()),
        uri: uri.to_string(),
        headers,
        body,
    };
    Ok(request)
}

pub fn response_from(buffer: &[u8], stream: TcpStream, first_read: usize) -> Result<Response, ResponseError> {
    let mut prev: Vec<char> = vec!('1', '2', '3', '4');
    let mut pre_body_index = 0;
    let mut snip = 0;
    let mut status_line = None;
    let mut headers = None;
    for char in buffer {
        if status_line.is_none() && prev[2] as char == '\r' && prev[3] as char == '\n' {
            status_line = Some(&buffer[..pre_body_index]);
            snip = pre_body_index;
        }
        if !status_line.is_none() && prev.iter().collect::<String>() == "\r\n\r\n" {
            headers = Some(&buffer[snip..pre_body_index - prev.len()]);
            break;
        }
        prev.remove(0);
        prev.push(*char as char);
        if pre_body_index > buffer.len() {
            return Err(ResponseError::HeadersTooBig(format!("Headers must be less than {}", buffer.len())));
        }
        pre_body_index += 1;
    }
    let status_line = str::from_utf8(&status_line.unwrap()).unwrap().split(" ").collect::<Vec<&str>>();

    let (_version, status_code, _status_message) = (status_line[0], status_line[1], status_line[2]);
    let header_string = str::from_utf8(&headers.unwrap()).unwrap();
    let headers = parse_headers(header_string);

    if header(&headers, "Content-Length").is_none() &&
        header(&headers, "Transfer-Encoding").is_none() {
        return Err(ResponseError::NoContentLengthOrTransferEncoding("Content-Length or Transfer-Encoding must be provided".to_string()));
    }

    // todo() support trailers

    let body;
    let content_length: Option<usize> = content_length_header(&headers);
    println!("first read {} vs headers {}", first_read, pre_body_index);
    println!("response from content length {} {} {}, {}", content_length.unwrap(), pre_body_index, buffer.len(), content_length.unwrap() + pre_body_index <= buffer.len());
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
                println!("stream.take response {}", content_length);
                body = Body::BodyStream(Box::new(body_stream));
            }
        }
        _ => body = Body::BodyString("".to_string())
    }

    Ok(Response {
        status: Status::from(status_code),
        headers,
        body
    })
}

#[derive(Debug)]
pub enum RequestError {
    NoContentLengthOrTransferEncoding(String),
    HeadersTooBig(String),
}

pub enum ResponseError {
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
        let http = format!("HTTP/1.1 {} {}", &self.status.to_string(), &self.status.to_u32());
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
            println!("turning body stream to string!");
            let big = &mut Vec::new();
            let read_bytes = reader.read_to_end(big).unwrap();
            println!("read {} bytes in body_string!", read_bytes);
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
    pub fn to_u32(&self) -> u32 {
        match self {
            OK => 200,
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

