use std::io::{copy, Read, Write};
use std::net::TcpStream;
use std::str;
use std::str::from_utf8;

use crate::headers::{DISALLOWED_TRAILERS, Headers};
use crate::http_message::Body::{BodyStream, BodyString};
use crate::http_message::Method::{CONNECT, DELETE, GET, HEAD, OPTIONS, PATCH, POST, PUT, TRACE};
use crate::http_message::Status::{BadRequest, InternalServerError, LengthRequired, MovedPermanently, NotFound, OK, Unknown};
use crate::uri::Uri;

pub enum HttpMessage<'a> {
    Request(Request<'a>),
    Response(Response<'a>),
}

impl<'a> HttpMessage<'a> {
    pub fn is_req(&self) -> bool {
        match self {
            HttpMessage::Request(_) => true,
            HttpMessage::Response(_) => false
        }
    }

    pub fn is_res(&self) -> bool {
        match self {
            HttpMessage::Request(_) => false,
            HttpMessage::Response(_) => true
        }
    }

    pub fn to_req(self) -> Request<'a> {
        match self {
            HttpMessage::Request(req) => req,
            _ => panic!("Not a request!")
        }
    }
    pub fn to_res(self) -> Response<'a> {
        match self {
            HttpMessage::Response(res) => res,
            _ => panic!("Not a request!")
        }
    }
}

pub fn message_from<'a>(
    mut stream: TcpStream,
    mut reader: &'a mut [u8],
    mut start_line_writer: &'a mut Vec<u8>,
    mut headers_writer: &'a mut Vec<u8>,
    mut trailers_writer: &'a mut Vec<u8>,
    chunks_vec: &'a mut Vec<u8>,
) -> Result<HttpMessage<'a>, MessageError> {
    let (mut read_bytes_from_stream, mut up_to_in_reader, mut result) =
        read(&mut stream, &mut reader, &mut start_line_writer, 0, 0,
             |reader, writer| { start_line_(reader, writer) },
        );
    let start_line = str::from_utf8(start_line_writer).unwrap().split(" ").collect::<Vec<&str>>();
    let (part1, part2, part3) = (start_line[0], start_line[1], start_line[2]);
    let is_response = part1.starts_with("HTTP");
    let method_can_have_body = vec!("POST", "PUT", "PATCH", "DELETE").contains(&part1);

    (read_bytes_from_stream, up_to_in_reader, result) =
        read(&mut stream, &mut reader, &mut headers_writer, read_bytes_from_stream, up_to_in_reader,
             |reader, writer| { headers_(reader, writer) },
        );

    if result.is_err() {
        return Err(result.err());
    }
    let header_string = from_utf8(headers_writer.as_slice()).unwrap();
    let mut headers = Headers::parse_from(header_string);

    if let Err(e) = check_valid_content_length_or_transfer_encoding(&headers, is_response, method_can_have_body) {
        return Err(e);
    }
    let transfer_encoding = headers.get("Transfer-Encoding");
    if headers.has("Content-Length") && transfer_encoding.is_some() {
        headers = headers.remove("Content-Length");
    }
    let is_version_1_0 = part3 == "HTTP/1.0";
    let result = body_from(
        &mut reader[up_to_in_reader..read_bytes_from_stream],
        stream.try_clone().unwrap(),
        chunks_vec,
        trailers_writer,
        &headers,
        is_version_1_0,
        !is_response,
        method_can_have_body,
        headers.content_length_header(),
        transfer_encoding,
    );
    if result.is_err() {
        return Err(result.err().unwrap());
    }
    let (body, headers, trailers) = result.unwrap();

    message(part1.to_string(), part2, part3.clone().to_string(), is_response, body, headers, trailers)
}

#[derive(Clone)]
enum ReadResult {
    Ok((bool, usize)),
    Err(MessageError)
}

impl ReadResult {
    pub fn of(pair: (bool, usize)) -> ReadResult {
        ReadResult::Ok(pair)
    }

    pub fn error(message_error: MessageError) -> ReadResult {
        ReadResult::Err(message_error)
    }

    pub fn err(self) -> MessageError {
        return match self {
            ReadResult::Ok(_) => panic!("Not an error"),
            ReadResult::Err(e) => e,
        }
    }

    pub fn is_err(&self) -> bool {
        return match self {
            ReadResult::Ok(_) => false,
            ReadResult::Err(_) => true
        }
    }

    pub fn unwrap(&self) -> (bool, usize) {
        *match &self {
            ReadResult::Ok(pair) => pair,
            ReadResult::Err(e) => panic!("Called unwrap on error: {}", e.to_string())
        }
    }

}


fn read(
    stream: &mut TcpStream,
    mut reader: &mut [u8],
    mut writer: &mut Vec<u8>,
    mut read_bytes_from_stream: usize,
    mut up_to_in_reader: usize,
    fun: fn(&mut [u8], &mut Vec<u8>) -> ReadResult
) -> (usize, usize, ReadResult) {
    let mut finished = false;
    let mut result = ReadResult::Err(MessageError::HeadersTooBig("".to_string()));
    while !finished {
        if up_to_in_reader > 0 {
            result = fun(&mut reader[up_to_in_reader..read_bytes_from_stream], writer);
            if result.is_err() {
                return (0, 0, ReadResult::Err(result.err()));
            }
            // if up_to_in_reader > 0 then we are continuing on in the same reader from previous call to stream.read()
            // ie we didn't need to call stream.read to finish parsing in fun()
            // so we need to increment the up_to_in_reader as we are not starting from the start of the reader
            let (now_finished, bytes_read) = result.unwrap();
            finished = now_finished;
            up_to_in_reader += bytes_read;
            if finished { break; }
        } else {
            read_bytes_from_stream = stream.read(&mut reader).unwrap();
            result = fun(&mut reader[..read_bytes_from_stream], writer);
            if result.is_err() {
                return (0, 0, ReadResult::Err(result.err()));
            }
            (finished, up_to_in_reader) = result.unwrap();
        }
    }
    (read_bytes_from_stream, up_to_in_reader, result)
}

fn do_thing(reader: &mut [u8], mut start_line_writer: &mut Vec<u8>, fun: fn(&mut [u8], &mut Vec<u8>) -> Result<(bool, usize), MessageError>) -> Result<(bool, usize), MessageError> {
    let mut result = fun(reader, start_line_writer);
    if result.is_err() {
        return Err(result.err().unwrap());
    }
    let (mut finished, mut up_to_in_reader) = result.ok().unwrap();
    Ok((finished, up_to_in_reader))
}

fn message<'a>(part1: String, part2: &'a str, part3: String, is_response: bool, body: Body<'a>, headers: Headers, trailers: Headers) -> Result<HttpMessage<'a>, MessageError> {
    if is_response {
        let (major, minor) = http_version_from(part1.as_str());
        Ok(HttpMessage::Response(Response {
            status: Status::from(part2),
            headers,
            body,
            version: HttpVersion { major, minor },
            trailers,
        }))
    } else {
        let (major, minor) = http_version_from(part3.as_str());
        Ok(HttpMessage::Request(Request {
            method: Method::from(part1.as_str()),
            uri: Uri::parse(part2),
            headers,
            body,
            version: HttpVersion { major, minor },
            trailers,
        }))
    }
}

fn body_from<'a>(
    mut buffer: &'a mut [u8],
    mut stream: TcpStream,
    chunks_writer: &'a mut Vec<u8>,
    trailers_writer: &'a mut Vec<u8>,
    existing_headers: &Headers,
    is_version_1_0: bool,
    is_request: bool,
    method_can_have_body: bool,
    content_length: Option<Result<usize, String>>,
    transfer_encoding: Option<String>) -> Result<(Body<'a>, Headers, Headers), MessageError> {
    let body;
    let mut trailers = Headers::empty();
    let mut headers = Headers::from_headers(existing_headers);
    let expected_trailers = headers.get("Trailer");
    let happy_to_receive_trailers = headers.get("TE").map(|t| t.contains("trailers")).unwrap_or(false);
    let cleansed_trailers = expected_trailers
        .map(|ts| ts.split(", ")
            .filter(|t| !DISALLOWED_TRAILERS.contains(&&*t.to_lowercase()))
            .map(|t| t.to_string())
            .collect::<Vec<String>>())
        .unwrap_or(vec!());

    if let Some(_encoding) = transfer_encoding {
        // read(&mut stream, buffer, chunks_writer, 0)
        let result = body_chunks_(buffer, chunks_writer, "metadata", 0, 0);
        if result.is_err() {
            return Err(result.err().unwrap());
        }
        let (mut finished, mut mode, mut chunked_body_bytes_read, mut read_of_current_chunk, mut chunk_size, mut start_of_trailers) = result.unwrap();

        let mut last_read_bytes = buffer.len();
        while !finished {
            let result = stream.read(&mut buffer);
            if result.is_err() { continue; };
            let read = result.unwrap();
            if read == 0 {
                break;
            }
            last_read_bytes = read;

            let result = body_chunks_(&buffer, chunks_writer, mode.to_string().as_str(), read_of_current_chunk, chunk_size);
            if result.is_err() {
                return Err(result.err().unwrap());
            }
            let (is_finished, current_mode, bytes_read, up_to_in_chunk, current_chunk_size, trailers_start_at) = result.unwrap();

            mode = current_mode;
            chunked_body_bytes_read += bytes_read;
            read_of_current_chunk = up_to_in_chunk;
            chunk_size = current_chunk_size;
            start_of_trailers = trailers_start_at;
            finished = is_finished;
        }

        let more_bytes_to_read_after_body = start_of_trailers < last_read_bytes;
        if more_bytes_to_read_after_body {
            let result = trailers_(&buffer[start_of_trailers..], trailers_writer);
            if result.is_err() {
                return Err(result.err().unwrap());
            }
            let (mut finished, mut new_trailers) = result.unwrap();
            trailers = new_trailers;
            while !finished {
                let mut buffer = [0; 4096];
                let result = stream.read(&mut buffer).unwrap();
                let result = trailers_(&buffer, trailers_writer);
                if result.is_err() {
                    return Err(result.err().unwrap());
                }
                let (is_finished, new_trailers) = result.unwrap();
                trailers = new_trailers;
                finished = is_finished;
            }
        }

        if !happy_to_receive_trailers {
            let as_str = cleansed_trailers.iter().map(|x| x.as_str()).collect::<Vec<&str>>();
            headers = headers.add_all(trailers.filter(as_str));
            trailers = Headers::empty();
        }
        // should only be doing this if we are talking to a user agent that does not accept chunked encoding
        // otherwise keep chunked encoding header
        if is_request && is_version_1_0 {
            headers = headers.add(("Content-Length", chunked_body_bytes_read.to_string().as_str()))
                .remove("Transfer-Encoding");
        }
        body = BodyStream(Box::new(chunks_writer.take(chunked_body_bytes_read as u64)));
    } else {
        match content_length {
            Some(_) if is_request && !method_can_have_body => {
                headers = headers.replace(("Content-Length", "0"));
                body = Body::empty()
            }
            // we have read the whole body in the first read
            Some(Ok(content_length)) if buffer.len() == content_length => {
                let result = str::from_utf8(&buffer[..]).unwrap();
                body = Body::BodyString(result)
            }
            Some(Ok(content_length)) => {
                // we need to read more to get the body
                if content_length > buffer.len() {
                    let rest = stream.take(content_length as u64 - buffer.len() as u64);
                    body = Body::BodyStream(Box::new(buffer.chain(rest)));
                } else {
                    let body_stream = stream.take(content_length as u64);
                    body = Body::BodyStream(Box::new(body_stream));
                }
            }
            Some(Err(error)) => {
                return Err(MessageError::InvalidContentLength(format!("Content Length header couldn't be parsed, got {}", error).to_string()));
            }
            _ => body = Body::empty()
        }
    }
    Ok((body, headers, trailers))
}

fn body_chunks_(reader: &[u8], writer: &mut Vec<u8>, mut mode: &str, read_up_to: usize, this_chunk_size: usize) -> Result<(bool, String, usize, usize, usize, usize), MessageError> {
    let mut prev = vec!('1', '2', '3', '4', '5');
    let mut chunk_size: usize = this_chunk_size;
    let mut bytes_of_this_chunk_read = read_up_to;
    let mut finished = false;
    let mut bytes_read_from_current_chunk: usize = 0;
    let mut total_bytes_read: usize = 0;
    let mut start_of_trailers: usize = 0;

    for (index, octet) in reader.iter().enumerate() {
        prev.remove(0);
        prev.push(*octet as char);

        let on_boundary = *octet == b'\n' || *octet == b'\r';
        if mode == "metadata" && !on_boundary {
            // if we have a digit, multiply last digit by 10 and add this one
            // if first digit we encounter is 0 then we'll multiply 0 by 10 and get 0
            // ... and know that we are at the end
            let option = (*octet as char).to_digit(10);
            if option.is_none() {
                return Err(MessageError::InvalidBoundaryDigit(format!("Could not parse boundary character {} in chunked encoding", *octet as char)));
            }
            chunk_size = (chunk_size * 10) + option.unwrap() as usize;
            if chunk_size == 0 { // we have encountered the 0 chunk
                //todo() if at the end of the buffer but we need to read again to get trailers
                finished = true;
                start_of_trailers = index + 3; // add 5 cos we didn't read the \r\n after the 0
                break;
            }
        } else if mode == "metadata" && on_boundary {
            // if we're on the boundary, continue, or change mode to read once we've seen \n
            if *octet == b'\n' {
                mode = "read";
            }
            continue;
        }
        if mode == "read" && bytes_read_from_current_chunk < (chunk_size - bytes_of_this_chunk_read) {
            writer.push(*octet);
            bytes_read_from_current_chunk += 1;
            let last_iteration = index == reader.len() - 1;
            if last_iteration {
                // if last index, add on the bytes read from current chunk
                bytes_of_this_chunk_read += bytes_read_from_current_chunk;
                total_bytes_read += bytes_read_from_current_chunk;
                start_of_trailers = index + 3;
                break;
            }
        } else if mode == "read" && on_boundary {
            // if we're on the boundary, continue, or change mode to metadata once we've seen \n
            // and reset counters
            if *octet == b'\n' {
                total_bytes_read += bytes_read_from_current_chunk;
                bytes_of_this_chunk_read = 0;
                chunk_size = 0;
                bytes_read_from_current_chunk = 0;
                mode = "metadata";
            }
            continue;
        }
    }

    Ok((finished, mode.to_string(), total_bytes_read, bytes_of_this_chunk_read, chunk_size, start_of_trailers))
}

fn check_valid_content_length_or_transfer_encoding(headers: &Headers, is_response: bool, method_can_have_body: bool) -> Result<(), MessageError> {
    let is_req_and_method_can_have_body = !is_response && method_can_have_body;
    let no_content_length_or_transfer_encoding = !headers.has("Content-Length") &&
        headers.get("Transfer-Encoding").is_none();

    if (is_req_and_method_can_have_body && no_content_length_or_transfer_encoding)
        || is_response && no_content_length_or_transfer_encoding {
        return Err(MessageError::NoContentLengthOrTransferEncoding("Content-Length or Transfer-Encoding must be provided".to_string()));
    }
    Ok(())
}

fn start_line_(reader: &[u8], mut writer: &mut Vec<u8>) -> ReadResult {
    let mut prev: Vec<char> = vec!('1', '2', '3', '4');
    let mut up_to_in_reader = if reader.len() > 0 { reader.len() - 1 } else { 0 };
    let mut finished = false;

    for (index, octet) in reader.iter().enumerate() {
        if *octet != b'\r' && *octet != b'\n' {
            writer.push(*octet);
        }
        if prev[3] == '\r' && *octet == b'\n' {
            finished = true;
            up_to_in_reader = index + 1;
            break;
        }
        if writer.len() == writer.capacity() {
            return ReadResult::Err(MessageError::StartLineTooBig(format!("Start line must be less than {}", reader.len())));
        }
        prev.remove(0);
        prev.push(*octet as char);
    }

    ReadResult::Ok((finished, up_to_in_reader))
}

fn headers_(reader: &[u8], mut writer: &mut Vec<u8>) -> ReadResult {
    let mut prev: Vec<char> = vec!('1', '2', '3', '4');
    let mut up_to_in_reader = reader.len() - 1;
    let mut finished = false;

    for (index, octet) in reader.iter().enumerate() {
        writer.push(*octet);
        up_to_in_reader = index + 1;
        if prev[1] == '\r' && prev[2] == '\n' && prev[3] == '\r' && *octet == b'\n' {
            finished = true;
            writer.pop();
            writer.pop();
            writer.pop();
            writer.pop(); // get rid of previous \r\n\r\n
            break;
        }
        if writer.len() == writer.capacity() {
            return ReadResult::Err(MessageError::HeadersTooBig(format!("Headers must be less than {}", writer.capacity())));
        }
        prev.remove(0);
        prev.push(*octet as char);
    }

    ReadResult::Ok((finished, up_to_in_reader))
}

fn trailers_(buffer: &[u8], writer: &mut Vec<u8>) -> Result<(bool, Headers), MessageError> {
    let mut prev: Vec<char> = vec!('1', '2', '3', '4');
    let mut finished = false;

    if buffer.len() == 0 {
        return Ok((true, Headers::empty()));
    }

    for (index, octet) in buffer.iter().enumerate() {
        if prev[1] == '\r' && prev[2] == '\n' && prev[3] == '\r' && *octet == b'\n' {
            finished = true;
            break;
        }
        if writer.len() == writer.capacity() {
            return Err(MessageError::TrailersTooBig(format!("Trailers must be less than {}", writer.capacity())));
        }
        prev.remove(0);
        prev.push(*octet as char);
        writer.push(*octet);
    }

    let mut headers = Headers::empty();
    if finished {
        writer.pop();
        writer.pop();
        writer.pop(); // get rid of previous \r\n\r\n
        let header_string = str::from_utf8(writer.as_slice()).unwrap();
        headers = Headers::parse_from(header_string);
    }
    Ok((finished, headers))
}

pub fn write_body(mut stream: &mut TcpStream, message: HttpMessage) {
    match message {
        HttpMessage::Request(mut req) => {
            let has_transfer_encoding = req.headers.has("Transfer-Encoding");
            let headers = ensure_content_length_or_transfer_encoding(&req.headers, &req.body, has_transfer_encoding, &req.version)
                .unwrap_or(req.headers);

            let mut request_string = format!("{} {} HTTP/{}.{}\r\n{}\r\n\r\n",
                                             req.method.value(),
                                             req.uri.to_string(),
                                             req.version.major,
                                             req.version.minor,
                                             headers.to_wire_string());

            match req.body {
                BodyString(str) => {
                    if has_transfer_encoding && req.version == one_pt_one() {
                        write_chunked_string(stream, request_string, str, req.trailers);
                    } else {
                        request_string.push_str(str);
                        stream.write(request_string.as_bytes()).unwrap();
                    }
                }
                BodyStream(ref mut reader) => {
                    if has_transfer_encoding && req.version == one_pt_one() {
                        write_chunked_stream(stream, reader, request_string, req.trailers);
                    } else {
                        let mut chain = request_string.as_bytes().chain(reader);
                        let _copy = copy(&mut chain, &mut stream).unwrap();
                    }
                }
            }
        }
        HttpMessage::Response(mut res) => {
            let has_transfer_encoding = res.headers.has("Transfer-Encoding");
            let headers = ensure_content_length_or_transfer_encoding(&res.headers, &res.body, has_transfer_encoding, &res.version)
                .unwrap_or(res.headers);
            let mut status_and_headers: String = Response::status_line_and_headers_wire_string(&headers, &res.status);
            let chunked_encoding_desired = headers.has("Transfer-Encoding");

            match res.body {
                BodyString(body_string) => {
                    if chunked_encoding_desired && res.version == one_pt_one() {
                        write_chunked_string(stream, status_and_headers, body_string, res.trailers);
                    } else {
                        status_and_headers.push_str(body_string);
                        stream.write(status_and_headers.as_bytes()).unwrap();
                        if !res.trailers.is_empty() {
                            stream.write(format!("\r\n{}\r\n\r\n", res.trailers.to_wire_string()).as_bytes()).unwrap();
                        }
                    }
                }
                BodyStream(ref mut reader) => {
                    if chunked_encoding_desired && res.version == one_pt_one() {
                        write_chunked_stream(&mut stream, reader, status_and_headers, res.trailers);
                    } else {
                        let mut chain = status_and_headers.as_bytes().chain(reader);
                        let _copy = copy(&mut chain, &mut stream).unwrap();
                    }
                }
            }
        }
    }
}

pub fn write_chunked_string(stream: &mut TcpStream, mut first_line: String, chunk: &str, trailers: Headers) {
    let length = chunk.len().to_string();
    let end = "0\r\n";

    first_line.push_str(length.as_str());
    first_line.push_str("\r\n");
    first_line.push_str(chunk);
    first_line.push_str("\r\n");
    first_line.push_str(end);

    if !trailers.is_empty() {
        first_line.push_str(format!("{}\r\n\r\n", trailers.to_wire_string()).as_str());
    }

    stream.write(first_line.as_bytes()).unwrap();
}

/*
https://doc.rust-lang.org/std/io/trait.Read.html#tymethod.read
It is not an error if the returned value n is smaller than the buffer size, even when the reader is not at the end of the stream yet.
This may happen for example because fewer bytes are actually available right now (e. g. being close to end-of-file) or because read() was interrupted by a signal.
 */
pub fn write_chunked_stream<'a>(mut stream: &mut TcpStream, reader: &mut Box<dyn Read + 'a>, first_line_and_headers: String, trailers: Headers) {
    let mut buffer = &mut [0 as u8; 16384];
    let mut bytes_read = reader.read(buffer).unwrap_or(0);
    let mut first_write = true;

    while bytes_read > 0 {
        let mut temp = vec!();
        temp.extend_from_slice(bytes_read.to_string().as_bytes());
        temp.push(b'\r');
        temp.push(b'\n');
        temp.append(&mut buffer[..bytes_read].to_vec());
        temp.push(b'\r');
        temp.push(b'\n');

        // write to wire
        if first_write {
            let first_line_and_headers_and_first_chunk = [first_line_and_headers.as_bytes(), temp.as_slice()].concat();
            let _copy = copy(&mut first_line_and_headers_and_first_chunk.as_slice(), &mut stream).unwrap();
            first_write = false;
        } else {
            let _copy = copy(&mut temp.as_slice(), &mut stream).unwrap();
        }

        bytes_read = reader.read(buffer).unwrap();
    }
    // write end byte
    let mut end = vec!(b'0', b'\r', b'\n');
    if !trailers.is_empty() {
        end.extend_from_slice(format!("{}\r\n\r\n", trailers.to_wire_string()).as_bytes());
    }
    stream.write(end.as_slice()).unwrap();
}

pub fn ensure_content_length_or_transfer_encoding(headers: &Headers, body: &Body, has_transfer_encoding: bool, version: &HttpVersion) -> Option<Headers> {
    let has_content_length = headers.has("Content-Length");
    if !has_content_length && !has_transfer_encoding && body.is_body_string() {
        Some(headers.add(("Content-Length", body.length().to_string().as_str())))
    } else if !has_content_length && !has_transfer_encoding && body.is_body_stream() && version == &one_pt_one() {
        Some(headers.add(("Transfer-Encoding", "chunked")))
    } else {
        None
    }
}

fn http_version_from(str: &str) -> (u8, u8) {
    let mut version_chars = str.chars();
    let major = version_chars.nth(0);
    let minor = version_chars.nth(2);
    (major.unwrap() as u8, minor.unwrap() as u8)
}

#[derive(Clone, Debug)]
pub enum MessageError {
    InvalidContentLength(String),
    NoContentLengthOrTransferEncoding(String),
    StartLineTooBig(String),
    HeadersTooBig(String),
    TrailersTooBig(String),
    InvalidBoundaryDigit(String)
}

impl MessageError {
    pub fn to_string(&self) -> String {
        match &self {
            MessageError::InvalidContentLength(_) => "Invalid content length".to_string(),
            MessageError::NoContentLengthOrTransferEncoding(_) => "No content length or transfer encoding".to_string(),
            MessageError::StartLineTooBig(_) => "Start line too big".to_string(),
            MessageError::HeadersTooBig(_) => "Headers too big".to_string(),
            MessageError::TrailersTooBig(_) => "Trailers too big".to_string(),
            MessageError::InvalidBoundaryDigit(_) => "Invalid boundary digit in chunked encoding".to_string(),
        }
    }
}

impl<'a> Response<'a> {
    pub fn status_line_and_headers_wire_string(headers: &Headers, status: &Status) -> String {
        let mut response = String::new();
        let http = format!("HTTP/1.1 {} {}", &status.value(), &status.to_string());
        response.push_str(&http);
        response.push_str("\r\n");

        response.push_str(&headers.to_wire_string().as_str());

        response.push_str("\r\n\r\n");
        response
    }
}

pub enum Body<'a> {
    BodyString(&'a str),
    BodyStream(Box<dyn Read + 'a>),
}

impl<'a> Body<'a> {
    pub fn empty() -> Body<'a> {
        BodyString("")
    }

    pub fn is_body_string(&self) -> bool {
        match self {
            BodyString(_) => true,
            BodyStream(_) => false
        }
    }

    pub fn is_body_stream(&self) -> bool {
        match self {
            BodyString(_) => false,
            BodyStream(_) => true
        }
    }

    pub fn length(&self) -> usize {
        match self {
            BodyString(str) => str.len(),
            BodyStream(_) => panic!("Do not know the length of a body stream!")
        }
    }
}

pub fn body_length(body: &Body) -> u32 {
    match body {
        BodyString(str) => str.len() as u32,
        BodyStream(_) => panic!("Cannot find length of BodyStream, please provide Content-Length header")
    }
}

pub fn with_content_length(message: HttpMessage) -> HttpMessage {
    match message {
        HttpMessage::Request(request) => {
            if !request.headers.has("Content-Length") && !request.headers.has("Transfer-Encoding") {
                return HttpMessage::Request(Request {
                    headers: request.headers.add(("Content-Length", body_length(&request.body).to_string().as_str())),
                    ..request
                });
            } else {
                HttpMessage::Request(request)
            }
        }
        HttpMessage::Response(response) => {
            if !response.headers.has("Content-Length") && !response.headers.has("Transfer-Encoding") {
                return HttpMessage::Response(Response {
                    headers: response.headers.add(("Content-Length", body_length(&response.body).to_string().as_str())),
                    ..response
                });
            } else {
                HttpMessage::Response(response)
            }
        }
    }
}

#[derive(PartialEq)]
pub struct HttpVersion {
    pub major: u8,
    pub minor: u8,
}

pub fn one_pt_one() -> HttpVersion {
    HttpVersion { major: 1, minor: 1 }
}

pub fn two_pt_oh() -> HttpVersion {
    HttpVersion { major: 2, minor: 0 }
}

pub fn one_pt_oh() -> HttpVersion {
    HttpVersion { major: 1, minor: 0 }
}

pub struct Request<'a> {
    pub headers: Headers,
    pub body: Body<'a>,
    pub uri: Uri<'a>,
    pub method: Method,
    pub version: HttpVersion,
    pub trailers: Headers,
}

pub struct Response<'a> {
    pub headers: Headers,
    pub body: Body<'a>,
    pub status: Status,
    pub version: HttpVersion,
    pub trailers: Headers,
}


impl<'a> Request<'a> {
    pub fn with_body(self, body: Body<'a>) -> Request<'a> {
        Request {
            body,
            ..self
        }
    }

    pub fn with_header(self, pair: (&str, &str)) -> Request<'a> {
        Request {
            headers: self.headers.add(pair),
            ..self
        }
    }

    pub fn with_trailers(self, trailers: Headers) -> Request<'a> {
        Request {
            trailers,
            ..self
        }
    }
}

pub fn body_string(mut body: Body) -> String {
    match body {
        BodyString(str) => str.to_string(),
        BodyStream(ref mut reader) => {
            let big = &mut Vec::new();
            let _read_bytes = reader.read_to_end(big).unwrap(); //todo() this blows up sometimes! unwrap_or()?
            str::from_utf8(&big).unwrap()
                .trim_end_matches(char::from(0))
                .to_string()
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum Method {
    GET,
    CONNECT,
    HEAD,
    POST,
    OPTIONS,
    DELETE,
    PATCH,
    PUT,
    TRACE,
}

impl Method {
    pub fn value(&self) -> String {
        match self {
            GET => String::from("GET"),
            POST => String::from("POST"),
            PATCH => String::from("PATCH"),
            OPTIONS => String::from("OPTIONS"),
            CONNECT => String::from("CONNECT"),
            HEAD => String::from("HEAD"),
            DELETE => String::from("DELETE"),
            PUT => String::from("PUT"),
            TRACE => String::from("TRACE")
        }
    }

    pub fn from(str: &str) -> Method {
        match str {
            "GET" => GET,
            "POST" => POST,
            "PATCH" => PATCH,
            "OPTIONS" => OPTIONS,
            "DELETE" => DELETE,
            "CONNECT" => CONNECT,
            "TRACE" => TRACE,
            "HEAD" => HEAD,
            _ => panic!("Unknown method")
        }
    }
}

impl<'a> Response<'a> {
    pub fn ok(headers: Headers, body: Body) -> Response {
        Response { headers, body, status: OK, version: HttpVersion { major: 1, minor: 1 }, trailers: Headers::empty() }
    }

    pub fn bad_request(headers: Headers, body: Body) -> Response {
        Response { headers, body, status: BadRequest, version: HttpVersion { major: 1, minor: 1 }, trailers: Headers::empty() }
    }

    pub fn internal_server_error(headers: Headers, body: Body) -> Response {
        Response { headers, body, status: InternalServerError, version: HttpVersion { major: 1, minor: 1 }, trailers: Headers::empty() }
    }

    pub fn length_required(headers: Headers, body: Body) -> Response {
        Response { headers, body, status: LengthRequired, version: HttpVersion { major: 1, minor: 1 }, trailers: Headers::empty() }
    }

    pub fn not_found(headers: Headers, body: Body) -> Response {
        Response { headers, body, status: NotFound, version: HttpVersion { major: 1, minor: 1 }, trailers: Headers::empty() }
    }

    pub fn moved_permanently(headers: Headers, body: Body) -> Response {
        Response { headers, body, status: MovedPermanently, version: HttpVersion { major: 1, minor: 1 }, trailers: Headers::empty() }
    }

    pub fn with_trailers(self, trailers: Headers) -> Response<'a> {
        Response {
            trailers,
            ..self
        }
    }

    pub fn with_headers(self, headers: Headers) -> Response<'a> {
        Response {
            headers: self.headers.add_all(headers),
            ..self
        }
    }
}

impl<'a> Request<'a> {
    pub fn request(method: Method, uri: Uri, headers: Headers) -> Request {
        Request { method, headers, body: Body::empty(), uri, version: HttpVersion { major: 1, minor: 1 }, trailers: Headers::empty() }
    }

    pub fn get(uri: Uri, headers: Headers) -> Request {
        Request { method: GET, headers, body: Body::empty(), uri, version: HttpVersion { major: 1, minor: 1 }, trailers: Headers::empty() }
    }

    pub fn post(uri: Uri<'a>, headers: Headers, body: Body<'a>) -> Request<'a> {
        Request { method: POST, headers, body, uri, version: HttpVersion { major: 1, minor: 1 }, trailers: Headers::empty() }
    }
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
    pub fn value(&self) -> u32 {
        match self {
            OK => 200,
            MovedPermanently => 301,
            BadRequest => 400,
            NotFound => 404,
            InternalServerError => 500,
            _ => 500
        }
    }
    pub fn from(str: &str) -> Self {
        match str.to_lowercase().as_str() {
            "200" => OK,
            "400" => BadRequest,
            "404" => NotFound,
            _ => Unknown
        }
    }
}

