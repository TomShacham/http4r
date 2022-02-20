use std::env;
use std::fs::{canonicalize, File};
use std::io::{Error, Read};
use std::str::from_utf8;
use http4r_core::handler::Handler;
use http4r_core::headers::{cache_control_header, content_type_header, Headers};
use http4r_core::http_message::{Request, Response};
use http4r_core::http_message::Body::{BodyStream, BodyString};

pub struct StaticFileHandler<'a> {
    root: &'a str,
    pub env_name: String,
}
impl<'a> StaticFileHandler<'a> {
    pub fn new(root: &'a str, env_name: String) -> StaticFileHandler<'a> {
        StaticFileHandler {
            root,
            env_name
        }
    }

    fn ok_or_not_found(path: String, file: Result<File, Error>, mut vec: &mut Vec<u8>, is_img: bool) -> Response {
        if file.is_err() {
            println!("Could not open file.");
            return Response::not_found(Headers::empty(), BodyString("Could not open file."));
        }
        let mut file = file.unwrap();
        let metadata = file.metadata();
        if metadata.is_err() {
            println!("Could not get metadata for file.");
            return Response::not_found(Headers::empty(), BodyString("Could not get metadata for file."))
        }
        if !metadata.unwrap().is_file() {
            println!("Not a file but a directory or symlink.");
            return Response::not_found(Headers::empty(), BodyString("Not a file but a directory or symlink."))
        }
        let read = file.read_to_end(&mut vec);
        if read.is_err() {
            println!("Failed to read file.");
            return Response::not_found(Headers::empty(), BodyString("Failed to read file."))
        }
        let (content_type, cache_control) = if path.ends_with(".html") {
            ("text/html", "private, max-age=10")
        } else if path.ends_with(".js") {
            ("text/javascript", "private, max-age=30")
        } else if path.ends_with(".css") {
            ("text/css", "private, max-age=60")
        } else if path.ends_with(".png") {
            ("image/png", "private, max-age=60")
        } else if path.ends_with(".jpg") {
            ("image/jpg", "private, max-age=60")
        } else if path.ends_with(".jpeg") {
            ("image/jpeg", "private, max-age=60")
        } else if path.ends_with(".bmp") {
            ("image/bmp", "private, max-age=60")
        } else if path.ends_with(".gif") {
            ("image/gif", "private, max-age=60")
        } else if path.ends_with(".webp") {
            ("image/webp", "private, max-age=60")
        } else if path.ends_with(".ico") {
            ("image/x-icon", "private, max-age=60")
        } else {
            ("text/plain", "no-store")
        };
        let headers = Headers::from(vec!(
            content_type_header(content_type),
            cache_control_header(cache_control),
            // content_length_header(vec.len().to_string().as_str())
        ));
        if is_img {
            Response::ok(headers.add(("Content-Encoding", "none")), BodyStream(Box::new(vec.as_slice())))
        } else {
            let str = from_utf8(vec.as_slice());
            if str.is_err() {
                println!("Could not read body into utf-8.");
                return Response::not_found(Headers::empty(), BodyString("Could not read body into utf-8."))
            }
            let body = str.unwrap();
            Response::ok(headers, BodyString(body))
        }
    }
}

impl<'a> Handler for StaticFileHandler<'a> {
    fn handle<F>(&mut self, req: Request, fun: F) -> () where F: FnOnce(Response) -> () + Sized {
        match req {
            Request { .. } => {
                let path = if req.uri.path == "/" {
                    "/index".to_string()
                } else {
                    req.uri.path.to_string()
                };
                let result = env::current_dir();
                if result.is_err() {
                    fun(Response::internal_server_error(Headers::empty(), BodyString("Failed to get current directory, perhaps insufficient permissions.")));
                    return;
                }
                let current_dir_plus_root = if self.env_name != "test" {
                    result.unwrap().to_str().unwrap().to_string() + "/http4r-example-app" + self.root
                } else { result.unwrap().to_str().unwrap().to_string() + self.root };
                let mut full_path = current_dir_plus_root.clone() + path.as_str();
                if !Self::is_sub_resource(&full_path) {
                    full_path = full_path + ".html";
                };
                let result = canonicalize(full_path.clone());
                if result.is_err() {
                    fun(Response::not_found(Headers::empty(), BodyString("File does not exist.")));
                    return;
                }
                let canonical_path = result.unwrap();
                let canonical_path_str = canonical_path.to_str().unwrap();
                if !canonical_path_str.starts_with(&current_dir_plus_root) {
                    let string = "Attempted to access a file outside of root: ".to_string() + canonical_path_str;
                    fun(Response::forbidden(Headers::empty(), BodyString(string.as_str())));
                    return;
                }
                println!("StaticFileHandler trying to open file at {}", canonical_path_str);
                let file = File::open(canonical_path_str);
                let mut vec = Vec::new();
                let res = Self::ok_or_not_found(full_path.clone(), file, &mut vec, Self::is_img(&full_path));
                fun(res);
            }
        }
    }
}

impl<'a> StaticFileHandler<'a> {
    fn is_img(full_path: &String) -> bool {
        full_path.ends_with(".png")
            || full_path.ends_with(".jpg")
            || full_path.ends_with(".jpeg")
            || full_path.ends_with(".ico")
            || full_path.ends_with(".bmp")
            || full_path.ends_with(".gif")
            || full_path.ends_with(".webp")
    }
}

impl<'a> StaticFileHandler<'a> {
    fn is_sub_resource(full_path: &String) -> bool {
        full_path.ends_with(".js")
            || full_path.ends_with(".css")
            || full_path.ends_with(".txt")
            || Self::is_img(full_path)
    }
}