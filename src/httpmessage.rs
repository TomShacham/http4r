use std::borrow::Borrow;
use std::io::Read;
use std::net::TcpStream;
use std::ops::Index;
use std::str;
use crate::httpmessage::Body::{BodyStream, BodyString};
use crate::httpmessage::Method::{DELETE, GET, OPTIONS, PATCH, POST};
use crate::httpmessage::Status::{NotFound, OK, Unknown, InternalServerError};


type Headers = Vec<Header>;

pub fn header(headers: &Headers, name: &str) -> Option<Header> {
    for header in headers {
        if header.0.to_lowercase() == name.to_lowercase() {
            return Some(header.clone())
        }
    }
    None
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
    return new
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

pub fn request_from(buffer: &[u8], stream1: TcpStream) -> Result<Request, String> {
    let mut prev: Vec<char> = vec!('1', '2', '3', '4');
    let mut index = 0;
    let mut snip = 0;
    let mut head = None;
    let mut headers = None;
    for char in buffer {
        if head.is_none() && prev[2] as char == '\r' && prev[3] as char == '\n' {
            head = Some(&buffer[..index]);
            snip = index;
        }
        if !head.is_none() && prev.iter().collect::<String>() == "\r\n\r\n" {
            headers = Some(&buffer[snip..index - prev.len()]);
        }
        prev.remove(0);
        prev.push(*char as char);
        if index > buffer.len() {
            return Err(format!("Headers must be less than {}", buffer.len()));
        }
        index += 1;
    }
    let request_line = str::from_utf8(&head.unwrap()).unwrap().split(" ").collect::<Vec<&str>>();
    ;
    let (method, uri, http_version) = (request_line[0], request_line[1], request_line[2]);
    let header_string = str::from_utf8(&headers.unwrap()).unwrap();

    let mut headers = vec!();
    header_string.split("\r\n").for_each(|pair| {
        let pair = pair.split(": ").collect::<Vec<&str>>();
        headers = add_header(&headers, (pair[0].to_string(), pair[1].to_string()));
    });

    let mut left_to_read = 0;
    let mut body ;
    let content_length: Option<usize> = content_length_header(&headers);
    match content_length {
        Some(content_length) if content_length + index < buffer.len() => {
            let result = str::from_utf8(&buffer[index..(content_length + index)]).unwrap().to_string();
            body = Body::BodyString(result)
        }
        Some(content_length) => {
            let so_far = &buffer[index..(content_length + index)];
            let rest = stream1.take(content_length as u64 - index as u64);
            body = Body::BodyStream(Box::new(so_far.chain(rest)));
            left_to_read = content_length - index;
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

impl<'a> Response<'a> {
    pub fn from(str: &str) -> Self {
        let mut headers = vec!();
        let split_by_crlf = str.split("\r\n").collect::<Vec<&str>>();
        let http = split_by_crlf.first().unwrap().to_string();
        let rest = &split_by_crlf[1..];
        let mut index = 1;
        for pair in rest {
            // indicates start of body, which is preceded by two consecutive \r\n
            if *pair == "" {
                break;
            }
            index += 1;
            let pair = pair.split(": ").collect::<Vec<&str>>();
            let (name, value) = (pair.first(), pair.last());
            headers.push((name.unwrap().to_string(), value.unwrap().to_string()));
        }
        let body = rest.get(index).unwrap();
        let http = http.split(" ").collect::<Vec<&str>>();
        let _version = http[0];
        let status = http[1];

        Response {
            headers,
            body: BodyString(body.to_string()),
            status: Status::from(status),
        }
    }
}

pub enum Body<'a> {
    BodyString(String),
    BodyStream(Box<dyn Read + 'a>),
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

pub fn body_string(body: Body) -> String {
    match body {
        BodyString(str) => str,
        BodyStream(mut tcp_stream) => {
            let mut buffer: [u8; 4096] = [0; 4096];
            tcp_stream.read(&mut buffer).unwrap();
            str::from_utf8(&buffer).unwrap().to_string()
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
    pub(crate) fn value(&self) -> String {
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

pub fn ok<'a>(headers: Vec<(String, String)>, body: Body<'a>) -> Response<'a> {
    Response { headers, body, status: OK }
}

pub fn not_found<'a>(headers: Vec<(String, String)>, body: Body<'a>) -> Response<'a> {
    Response { headers, body, status: NotFound }
}

pub fn get<'a>(uri: String, headers: Vec<(String, String)>) -> Request<'a> {
    Request { method: GET, headers, body: BodyString("".to_string()), uri }
}


#[derive(PartialEq, Debug)]
#[repr(u32)]
pub enum Status {
    OK = 200,
    NotFound = 404,
    InternalServerError = 500,
    Unknown = 0,
}

impl Status {
    pub fn to_string(&self) -> String {
        match self {
            OK => "OK".to_string(),
            NotFound => "Not Found".to_string(),
            InternalServerError => "Internal Server Error".to_string(),
            _ => "Internal Server Error".to_string()
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

