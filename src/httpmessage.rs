use std::borrow::Borrow;
use std::io::Read;
use std::net::TcpStream;
use std::ops::Index;
use std::str;
use crate::httpmessage::Body::{BodyStream, BodyString};
use crate::httpmessage::Method::{DELETE, GET, OPTIONS, PATCH, POST};
use crate::httpmessage::Status::{NotFound, OK, Unknown, InternalServerError};

pub type Header = (String, String);

pub enum HttpMessage {
    Request(Request),
    Response(Response),
}

impl HttpMessage {
    pub fn as_res(self) -> Response {
        match self {
            HttpMessage::Response(res) => res,
            _ => panic!("Not a response")
        }
    }
    pub fn as_req(self) -> Request {
        match self {
            HttpMessage::Request(req) => req,
            _ => panic!("Not a request")
        }
    }
}

impl From<Request> for HttpMessage {
    fn from(req: Request) -> Self {
        HttpMessage::Request(req)
    }
}

impl From<Response> for HttpMessage {
    fn from(res: Response) -> Self {
        HttpMessage::Response(res)
    }
}

impl From<HttpMessage> for Request {
    fn from(message: HttpMessage) -> Self {
        match message {
            HttpMessage::Request(req) => req,
            _ => panic!("Not possible")
        }
    }
}

impl From<HttpMessage> for Response {
    fn from(message: HttpMessage) -> Self {
        match message {
            HttpMessage::Response(res) => res,
            _ => panic!("Not possible")
        }
    }
}

pub fn request_from(buffer: &[u8], stream: &TcpStream) -> Result<Request, String> {
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
    let result = str::from_utf8(&headers.unwrap());

    let headers = result.unwrap().split("\r\n").map(|pair| {
        let pair = pair.split(": ").collect::<Vec<&str>>();
        (pair[0].to_string(), pair[1].to_string())
    }).collect::<Vec<(String, String)>>();

    Ok(Request {
        method: Method::from(method.to_string()),
        uri: uri.to_string(),
        headers,
        body: Body::BodyString("".to_string()),
    })
}

impl From<&str> for Request {
    fn from(str: &str) -> Self {
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
        let method = http[0];
        let uri = http[1];

        Request {
            method: Method::from(method.to_uppercase()),
            headers,
            body: BodyString(body.to_string()),
            uri: uri.to_string(),
        }
    }
}

impl Response {
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

impl Response {
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

pub enum Body {
    BodyString(String),
    BodyStream(Box<dyn Read>),
}

pub struct Request {
    pub headers: Vec<Header>,
    pub body: Body,
    pub uri: String,
    pub method: Method,
}

pub struct Response {
    pub headers: Vec<Header>,
    pub body: Body,
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

pub fn ok(headers: Vec<(String, String)>, body: Body) -> Response {
    Response { headers, body, status: OK }
}

pub fn not_found(headers: Vec<(String, String)>, body: Body) -> Response {
    Response { headers, body, status: NotFound }
}

pub fn get(uri: String, headers: Vec<(String, String)>) -> Request {
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

