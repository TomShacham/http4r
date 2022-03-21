use http4r_core::handler::Handler;
use http4r_core::headers::Headers;
use http4r_core::http_message::{Request, Response};
use http4r_core::http_message::Body::BodyString;
use http4r_core::uri::Uri;
use serde_json::Value;
use crate::environment::Environment;
use crate::not_found_handler::NotFoundHandler;
use crate::static_file_handler::StaticFileHandler;

pub struct App<H> where H: Handler {
    handler: H,
    pub env: Environment,
}

impl<H> App<H> where H: Handler {
    pub fn new(handler: H, env: Environment) -> App<H> where H: Handler {
        App {
            handler,
            env,
        }
    }
}

impl<H> Handler for App<H> where H: Handler {
    fn handle<F>(&mut self, req: Request, fun: F) -> () where F: FnOnce(Response) -> () + Sized {
        self.handler.handle(req, |res| {
            fun(res)
        })
    }
}

impl<'a> App<NotFoundHandler<StaticFileHandler<'a>>> {
    pub fn in_memory(env: Environment) -> App<NotFoundHandler<StaticFileHandler<'a>>> {
        let env_name = env.get("ENV").unwrap_or("test".to_string());
        App::new(NotFoundHandler::new(StaticFileHandler::new("/resources/html", env_name)), env)
    }

    pub fn production(env: Environment) -> App<EntryPoint<Api, NotFoundHandler<StaticFileHandler<'a>>>> {
        let env_name = env.get("ENV").unwrap_or("production".to_string());
        let not_found_handler = NotFoundHandler::new(
            StaticFileHandler::new("/resources/html", env_name));
        App::new(EntryPoint::new(Api::new(), not_found_handler), env)
    }
}

pub struct EntryPoint<G, H> where H: Handler, G: Handler {
    api: G,
    next: H,
}

impl<G, H> EntryPoint<G, H> where H: Handler, G: Handler {
    pub fn new(api: G, next: H) -> EntryPoint<G, H> {
        EntryPoint { api, next }
    }
}

impl<G, H> Handler for EntryPoint<G, H> where H: Handler, G: Handler {
    fn handle<F>(&mut self, req: Request, fun: F) -> () where F: FnOnce(Response) -> () + Sized {
        match req {
            Request { .. } if req.uri.path.starts_with("/api/") => {
                self.api.handle(req.with_uri(Uri::parse(req.uri.path.replace("/api/", "/").as_str())), fun)
            }
            _ => self.next.handle(req, fun)
        }
    }
}

pub struct Api;

impl Api {
    pub fn new() -> Api {
        Api {}
    }
}

impl Handler for Api {
    fn handle<F>(&mut self, req: Request, fun: F) -> () where F: FnOnce(Response) -> () + Sized {
        match req {
            Request { .. } if req.uri.path.starts_with("/") => {
                let data = r#"{
                    "name": "John Doe",
                    "age": 43
                }"#;
                let json: Value = serde_json::from_str(data).unwrap();
                fun(Response::ok(Headers::empty(), BodyString(json.as_str().unwrap_or("{}"))))
            }
            _ => fun(Response::not_found(Headers::empty(), BodyString("Not found"))),
        }
    }
}