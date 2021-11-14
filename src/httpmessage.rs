use crate::httpmessage::Method::{DELETE, GET, OPTIONS, PATCH, POST};
use crate::httpmessage::Status::{NotFound, OK};

pub type Header = (String, String);

pub enum HttpMessage {
    Request(Request),
    Response(Response),
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

pub struct Request {
    pub headers: Vec<Header>,
    pub body: String,
    pub uri: String,
    pub method: Method,
}


#[derive(PartialEq, Debug)]
pub struct Response {
    pub headers: Vec<Header>,
    pub body: String,
    pub status: Status,
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

pub fn ok(headers: Vec<(String, String)>, body: String) -> Response {
    Response { headers, body, status: OK }
}

pub fn not_found(headers: Vec<(String, String)>, body: String) -> Response {
    Response { headers, body, status: NotFound }
}

pub fn get(uri: String, headers: Vec<(String, String)>, body: String) -> Request {
    Request { method: GET, headers, body, uri }
}


#[derive(PartialEq, Debug)]
pub enum Status {
    OK = 200,
    NotFound = 404,
}
