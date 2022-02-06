use http4r_core::handler::Handler;
use http4r_core::headers::Headers;
use http4r_core::http_message::{Request, Response};
use http4r_core::http_message::Body::BodyString;
use http4r_core::uri::Uri;

#[cfg(test)]
mod tests {
    use std::fs;
    use std::fs::File;
    use std::io::{Read, Write};
    use std::str::{from_utf8, Utf8Error};
    use http4r_core::client::{Client};
    use http4r_core::handler::Handler;
    use http4r_core::headers::Headers;
    use http4r_core::http_message::{body_string, Request};
    use http4r_core::http_message::Status::NotFound;
    use http4r_core::server::Server;
    use http4r_core::uri::Uri;
    use http4r_example_app::app::App;

    #[test]
    fn home_page() {
        let mut server = Server::new(0);
        server.test(|| {
            let app = App::new();
            Ok(app) });

        let mut client = Client::new("127.0.0.1", server.port, None);
        client.handle(Request::get(Uri::parse("/"), Headers::empty()), |res| {
            approve(body_string(res.body), "./resources/html/index.html");
        })
    }

    fn approve(actual: String, expected_file: &str) {
        let str = fs::read_to_string(expected_file);
        make_approval_file(&actual, expected_file);
        if str.unwrap() != actual.as_str() {
            panic!("Expected file not the same: {}", expected_file);
        }
    }

    fn make_approval_file(actual: &String, expected_file: &str) {
        let split_on_slash = expected_file.split("/").map(|it| it.to_string()).collect::<Vec<String>>();
        let file_iter = split_on_slash.iter();
        let up_to_file_name = file_iter.clone().take(split_on_slash.len() - 1).map(|it| it.to_string()).collect::<Vec<String>>().join("/");
        let file_name = file_iter.clone().last();
        let name_and_extension = file_name.map(|name| {
            let split = name.to_string().split(".").map(|x| x.to_string()).collect::<Vec<String>>();
            (split.get(0).unwrap().to_owned(), split.get(1).unwrap().to_owned())
        }).unwrap_or(("unknown_file_name".to_string(), "unknown_extension".to_string()));
        let approval_output_file_name = name_and_extension.0 + "_approval" + "." + name_and_extension.1.as_str();
        let create = File::create(up_to_file_name.clone() + "/" + approval_output_file_name.clone().as_str());
        if create.is_err() {
            panic!("{}", format!("Failed to create approval file named: {}", up_to_file_name.clone() + "/" + approval_output_file_name.clone().as_str()));
        }
        create.unwrap().write_all(actual.as_bytes());
    }
}