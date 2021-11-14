use crate::headers::add_header;
use crate::httphandler::{Handler, HttpHandler};
use crate::httpmessage::HttpMessage;

pub trait Filter {
    fn filter(&self, handler: Box<dyn HttpHandler>) -> Box<dyn HttpHandler>;
}

impl Filter for IdentityFilter {
    fn filter(&self, handler: Box<dyn HttpHandler>) -> Box<dyn HttpHandler> {
        Box::new(Handler {
            handler: (move |req| {
                return handler.handle(req);
            })
        })
    }

}

impl Filter for RawHttpFilter {
    fn filter(&self, handler: Box<dyn HttpHandler>) -> Box<dyn HttpHandler> {
        self.next.filter(
            Box::new(Handler {
                handler: (move |request| {
                    let req = add_header(("req-header-1".to_string(), "req-value-1".to_string()), HttpMessage::Request(request)).into();
                    let response = handler.handle(req);
                    add_header(("res-header-1".to_string(), "res-value-1".to_string()), HttpMessage::Response(response)).into()
                })
            })
        )
    }
}

pub struct RawHttpFilter {
    pub next: Box<dyn Filter>,
}

pub struct IdentityFilter {}
