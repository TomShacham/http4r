use crate::httpmessage::Method::{DELETE, GET, OPTIONS, PATCH, POST};
use crate::httpmessage::Status::{NotFound, OK, Unknown, InternalServerError};

pub type Header = (String, String);

pub enum HttpMessage {
    Request(Request),
    Response(Response),
}
impl HttpMessage {
    pub fn to_res(self) -> Response {
        match self {
            HttpMessage::Response(res) => res,
            _ => panic!("Not a response")
        }
    }
    pub fn to_req(self) -> Request {
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
            body: body.to_string(),
            uri: uri.to_string(),
        }
    }
}

impl Response {
    pub fn to_string(&self) -> String {
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
        response.push_str(&self.body);
        response.push_str("\r\n");
        response
    }
}

impl From<&str> for Response {
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
        let _version = http[0];
        let status = http[1];

        Response {
            headers,
            body: body.to_string(),
            status: Status::from(status),
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

pub fn get(uri: String, headers: Vec<(String, String)>) -> Request {
    Request { method: GET, headers, body: "".to_string(), uri }
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

