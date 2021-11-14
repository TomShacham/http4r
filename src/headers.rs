use crate::httpmessage::{Header, HttpMessage, Request, Response};

pub fn add_header(header: Header, to: HttpMessage) -> HttpMessage {
    match to {
        HttpMessage::Request(req) => {
            let mut headers = req.headers.clone();
            headers.push(header);
            HttpMessage::Request(Request {
                headers,
                ..req
            })
        }
        HttpMessage::Response(res) => {
            let mut headers = res.headers.clone();
            headers.push(header);
            HttpMessage::Response(Response {
                headers,
                ..res
            })
        }
    }
}
