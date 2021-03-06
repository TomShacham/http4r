/*
    http4r is a web toolkit
    Copyright (C) 2021 Tom Shacham

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::io::{copy, Read, Write};
use std::net::TcpStream;
use std::str;
use std::str::from_utf8;
use crate::codex::Codex;

use crate::headers::{DISALLOWED_TRAILERS, Headers};
use crate::http_message::Body::{BodyStream, BodyString};
use crate::http_message::CompressionAlgorithm::{BROTLI, DEFLATE, GZIP, NONE};
use crate::http_message::Method::{CONNECT, DELETE, GET, HEAD, OPTIONS, PATCH, POST, PUT, TRACE};
use crate::http_message::Status::{BadRequest, Forbidden, InternalServerError, LengthRequired, MovedPermanently, NotFound, OK, Unknown};
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

#[allow(unused_assignments)]
pub fn read_message_from_wire<'a>(
    mut stream: TcpStream,
    mut reader: &'a mut [u8],
    mut start_line_writer: &'a mut Vec<u8>,
    mut headers_writer: &'a mut Vec<u8>,
    chunks_writer: &'a mut Vec<u8>,
    compress_writer: &'a mut Vec<u8>,
    trailers_writer: &'a mut Vec<u8>,
) -> Result<HttpMessage<'a>, MessageError> {
    let (mut read_bytes_from_stream, mut up_to_in_reader, mut result) =
        read(&mut stream, &mut reader, &mut start_line_writer, 0, 0, None,
             |reader, writer, _| { start_line_(reader, writer) },
        );
    let start_line = str::from_utf8(start_line_writer).unwrap().split(" ").collect::<Vec<&str>>();
    let (part1, part2, part3) = (start_line[0], start_line[1], start_line[2]);
    let is_response = part1.starts_with("HTTP");
    let is_request = !is_response;
    let method_can_have_body = vec!("POST", "PUT", "PATCH", "DELETE").contains(&part1);

    (read_bytes_from_stream, up_to_in_reader, result) =
        read(&mut stream, &mut reader, &mut headers_writer, read_bytes_from_stream, up_to_in_reader, None,
             |reader, writer, _| { headers_(reader, writer) },
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

    let request_options = RequestOptions::from(&headers);
    let compression = request_options.read_compression();
    let content_length = headers.content_length_header();

    let result = if let Some(_encoding) = transfer_encoding {
        chunked_body_and_trailers(reader, stream, up_to_in_reader, read_bytes_from_stream, chunks_writer, compress_writer, trailers_writer, &compression)
    } else {
        simple_body(reader, stream, up_to_in_reader, read_bytes_from_stream, is_request, method_can_have_body, content_length, compression, compress_writer)
    };
    if result.is_err() {
        return Err(result.err().unwrap());
    }
    let (body, trailers, content_length) = result.unwrap();

    if headers.get("Transfer-Encoding").is_none() {
        headers = headers.replace(("Content-Length", content_length.to_string().as_str()));
    }
    // should only be doing this if we are talking to a user agent that does not accept chunked encoding
    // otherwise keep chunked encoding header
    if is_request && is_version_1_0 {
        headers = headers.add(("Content-Length", content_length.to_string().as_str()))
            .remove("Transfer-Encoding");
    }

    message(part1.to_string(), part2, part3.clone().to_string(), is_response, body, headers, trailers)
}

#[derive(Clone)]
enum ReadResult {
    Ok((bool, usize, Option<ReadMetadata>)),
    Err(MessageError),
}

impl ReadResult {
    pub fn err(self) -> MessageError {
        return match self {
            ReadResult::Ok(_) => panic!("Not an error"),
            ReadResult::Err(e) => e,
        };
    }

    pub fn is_err(&self) -> bool {
        return match self {
            ReadResult::Ok(_) => false,
            ReadResult::Err(_) => true
        };
    }

    pub fn unwrap(&self) -> (bool, usize, Option<ReadMetadata>) {
        *match &self {
            ReadResult::Ok(triple) => triple,
            ReadResult::Err(e) => panic!("Called unwrap on error: {}", e.to_string())
        }
    }
}

#[derive(Copy, Clone)]
struct ChunkedMetadata {
    mode: ReadMode,
    chunk_size: usize,
    bytes_of_this_chunk_read: usize,
}

#[derive(Copy, Clone)]
struct MultipartMetadata {}

#[derive(Copy, Clone)]
enum ReadMetadata {
    Chunked(ChunkedMetadata),
    Multipart(MultipartMetadata),
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum ReadMode {
    Metadata,
    Data,
}

impl ReadMetadata {
    pub fn chunked(mode: ReadMode, chunk_size: usize, bytes_of_this_chunk_read: usize) -> ReadMetadata {
        ReadMetadata::Chunked(ChunkedMetadata { mode, chunk_size, bytes_of_this_chunk_read })
    }

    pub fn to_chunked_metadata(self) -> ChunkedMetadata {
        match self {
            ReadMetadata::Chunked(m) => m,
            ReadMetadata::Multipart(_) => panic!("Have multipart metadata but expected chunked metadata")
        }
    }
}

fn read(
    stream: &mut TcpStream,
    mut reader: &mut [u8],
    writer: &mut Vec<u8>,
    mut read_bytes_from_stream: usize,
    mut up_to_in_reader: usize,
    mut metadata: Option<ReadMetadata>,
    fun: fn(&mut [u8], &mut Vec<u8>, Option<ReadMetadata>) -> ReadResult,
) -> (usize, usize, ReadResult) {
    let mut finished = false;
    let mut result = ReadResult::Err(MessageError::HeadersTooBig("".to_string()));
    while !finished {
        if up_to_in_reader > 0 && read_bytes_from_stream > up_to_in_reader {
            result = fun(&mut reader[up_to_in_reader..read_bytes_from_stream], writer, metadata);
            if result.is_err() {
                return (0, 0, ReadResult::Err(result.err()));
            }
            // if up_to_in_reader > 0 then we are continuing on in the same reader from previous call to stream.read()
            // ie we didn't need to call stream.read to finish parsing in fun()
            // so we need to increment the up_to_in_reader as we are not starting from the start of the reader
            let (now_finished, new_up_to_in_reader, new_metadata) = result.unwrap();
            metadata = new_metadata;
            finished = now_finished;
            // if we reached the end of the reader then set to zero, otherwise add on how far through we got
            if new_up_to_in_reader == 0 {
                up_to_in_reader = 0
            } else {
                up_to_in_reader += new_up_to_in_reader
            };
            if finished { break; }
        } else {
            read_bytes_from_stream = stream.read(&mut reader).unwrap();
            result = fun(&mut reader[..read_bytes_from_stream], writer, metadata);
            if result.is_err() {
                return (0, 0, ReadResult::Err(result.err()));
            }
            (finished, up_to_in_reader, metadata) = result.unwrap();
        }
    }
    (read_bytes_from_stream, up_to_in_reader, result)
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

fn simple_body<'a>(
    reader: &'a mut [u8],
    stream: TcpStream,
    up_to_in_reader: usize,
    read_bytes_from_stream: usize,
    is_request: bool,
    method_can_have_body: bool,
    content_length: Option<Result<usize, String>>,
    compression: CompressionAlgorithm,
    mut compress_writer: &'a mut Vec<u8>,
) -> Result<(Body<'a>, Headers, usize), MessageError> {
    let bytes_left_in_reader = read_bytes_from_stream - up_to_in_reader;
    let (body, content_length) = match content_length {
        Some(_) if is_request && !method_can_have_body => {
            (Body::empty(), 0)
        }
        // we have read the whole body in the first read
        Some(Ok(content_length)) if bytes_left_in_reader == content_length => {
            if compression.is_some() {
                decompress(&compression, &mut compress_writer, &mut reader[up_to_in_reader..read_bytes_from_stream].to_vec());
                let result = str::from_utf8(compress_writer.as_slice()).unwrap();
                (Body::BodyString(result), compress_writer.len())
            } else {
                let result = str::from_utf8(&reader[up_to_in_reader..read_bytes_from_stream]).unwrap();
                (Body::BodyString(result), content_length)
            }
        }
        Some(Ok(content_length)) => {
            // we need to read more to get the body
            if compression.is_some() {
                let mut rest = Vec::new();
                let _read = stream.take((content_length - bytes_left_in_reader) as u64).read_to_end(&mut rest).unwrap();
                let mut whole = reader[up_to_in_reader..read_bytes_from_stream].to_vec();
                whole.append(&mut rest);
                decompress(&compression, &mut compress_writer, &mut whole);
                let result = str::from_utf8(compress_writer.as_slice()).unwrap();
                (Body::BodyString(result), compress_writer.len())
            } else {
                let rest = stream.take((content_length - bytes_left_in_reader) as u64);
                (Body::BodyStream(Box::new(reader[up_to_in_reader..read_bytes_from_stream].chain(rest))), content_length)
            }
        }
        Some(Err(error)) => {
            return Err(MessageError::InvalidContentLength(format!("Content Length header couldn't be parsed, got {}", error).to_string()));
        }
        _ => (Body::empty(), 0)
    };
    Ok((body, Headers::empty(), content_length))
}

fn chunked_body_and_trailers<'a>(
    reader: &'a mut [u8],
    mut stream: TcpStream,
    up_to_in_reader: usize,
    read_bytes_from_stream: usize,
    chunks_writer: &'a mut Vec<u8>,
    compress_writer: &'a mut Vec<u8>,
    trailers_writer: &'a mut Vec<u8>,
    compression: &CompressionAlgorithm,
) -> Result<(Body<'a>, Headers, usize), MessageError> {
    let result = read_body_and_trailers(reader, &mut stream, up_to_in_reader, read_bytes_from_stream, chunks_writer, trailers_writer);
    if result.is_err() {
        return Err(result.err().unwrap());
    }
    let chunked_body_bytes_read = result.unwrap();
    let trailer_string = from_utf8(trailers_writer.as_slice()).unwrap();
    let trailers = Headers::parse_from(trailer_string);

    let body = if compression.is_some() {
        decompress(&compression, compress_writer, chunks_writer);
        BodyStream(Box::new(compress_writer.take(compress_writer.len() as u64)))
    } else {
        BodyStream(Box::new(chunks_writer.take(chunks_writer.len() as u64)))
    };
    Ok((body, trailers, chunked_body_bytes_read))
}

#[allow(unused_assignments)]
fn read_body_and_trailers(reader: &mut [u8], mut stream: &mut TcpStream, up_to_in_reader: usize, read_bytes_from_stream: usize, chunks_writer: &mut Vec<u8>, trailers_writer: &mut Vec<u8>) -> Result<usize, MessageError> {
    let metadata = Some(ReadMetadata::chunked(ReadMode::Metadata, 0, 0));
    let (mut read_bytes_from_stream, mut up_to_in_reader, mut result) =
        read(&mut stream, reader, chunks_writer, read_bytes_from_stream, up_to_in_reader, metadata, |reader, writer, metadata| {
            let meta = metadata.unwrap().to_chunked_metadata();
            body_chunks_(reader, writer, meta.mode, meta.bytes_of_this_chunk_read, meta.chunk_size)
        });
    if result.is_err() {
        return Err(result.err());
    }
    let (_finished, chunked_body_bytes_read, _metadata) = result.unwrap();

    let more_bytes_to_read_after_body = up_to_in_reader < read_bytes_from_stream;
    if more_bytes_to_read_after_body {
        (read_bytes_from_stream, up_to_in_reader, result) =
            read(&mut stream, reader, trailers_writer, read_bytes_from_stream, up_to_in_reader, metadata, |reader, writer, _metadata| {
                trailers_(reader, writer)
            });
    }
    if result.is_err() {
        return Err(result.err());
    }
    Ok(chunked_body_bytes_read)
}

fn body_chunks_(reader: &[u8], writer: &mut Vec<u8>, mut mode: ReadMode, read_up_to: usize, this_chunk_size: usize) -> ReadResult {
    let mut prev = vec!('1', '2', '3', '4', '5');
    let mut chunk_size: usize = this_chunk_size;
    let mut chunk_size_hex: String = "".to_string();
    let mut bytes_of_this_chunk_read_previously = read_up_to;
    let mut finished = false;
    let mut bytes_of_chunk_read_this_pass: usize = 0;
    let mut up_to_in_reader: usize = 0;

    for (index, octet) in reader.iter().enumerate() {
        prev.remove(0);
        prev.push(*octet as char);

        let on_boundary = *octet == b'\n' || *octet == b'\r';
        if mode == ReadMode::Metadata && !on_boundary {
            chunk_size_hex.push(*octet as char);
        } else if mode == ReadMode::Metadata && on_boundary {
            // if we're on the boundary, continue, or compute chunk length and change mode to read once we've seen \n
            if *octet == b'\n' {
                let result = usize::from_str_radix(&chunk_size_hex, 16);
                if result.is_err() {
                    return ReadResult::Err(MessageError::InvalidBoundaryDigit(format!("Could not parse boundary character {} in chunked encoding", &chunk_size_hex)));
                }
                // reset chunk_size_hex
                chunk_size_hex = "".to_string();
                chunk_size = result.unwrap();
                if chunk_size == 0 {
                    finished = true;
                    // if more bytes left then assume there are trailers (which means end is 0\r\n otherwise its 0\r\n\r\n)
                    if reader.len() > index + 3 {
                        up_to_in_reader = index + 1
                    } else {
                        up_to_in_reader = index + 3; // add 3 to go past the next \n (weve hit the first one in 0\r\n\r\n)
                    }
                    break;
                }
                mode = ReadMode::Data;
            }
            continue;
        }
        if mode == ReadMode::Data && bytes_of_chunk_read_this_pass < (chunk_size - bytes_of_this_chunk_read_previously) {
            writer.push(*octet);
            bytes_of_chunk_read_this_pass += 1;
            let last_iteration = index == reader.len() - 1;
            if last_iteration {
                // if last index, add on the bytes read from current chunk
                bytes_of_this_chunk_read_previously += bytes_of_chunk_read_this_pass;
                up_to_in_reader = 0;
                break;
            }
        } else if mode == ReadMode::Data && on_boundary {
            // if we're on the boundary, continue, or change mode to metadata once we've seen \n
            // and reset counters
            if *octet == b'\n' {
                bytes_of_this_chunk_read_previously = 0;
                chunk_size = 0;
                bytes_of_chunk_read_this_pass = 0;
                mode = ReadMode::Metadata;
            }
            continue;
        }
    }

    let metadata = ReadMetadata::chunked(mode, chunk_size, bytes_of_this_chunk_read_previously);
    ReadResult::Ok((finished, up_to_in_reader, Some(metadata)))
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

fn start_line_(reader: &[u8], writer: &mut Vec<u8>) -> ReadResult {
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

    ReadResult::Ok((finished, up_to_in_reader, None))
}

fn headers_(reader: &[u8], writer: &mut Vec<u8>) -> ReadResult {
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

    ReadResult::Ok((finished, up_to_in_reader, None))
}

fn trailers_(buffer: &[u8], writer: &mut Vec<u8>) -> ReadResult {
    let mut prev: Vec<char> = vec!('1', '2', '3', '4');
    let mut finished = false;

    for (_index, octet) in buffer.iter().enumerate() {
        if prev[1] == '\r' && prev[2] == '\n' && prev[3] == '\r' && *octet == b'\n' {
            finished = true;
            break;
        }
        if writer.len() == writer.capacity() {
            return ReadResult::Err(MessageError::TrailersTooBig(format!("Trailers must be less than {}", writer.capacity())));
        }
        prev.remove(0);
        prev.push(*octet as char);
        writer.push(*octet);
    }

    if finished {
        writer.pop();
        writer.pop();
        writer.pop(); // get rid of previous \r\n\r\n
    }

    ReadResult::Ok((finished, 0, None))
}

#[derive(Copy, Clone)]
pub enum CompressionAlgorithm {
    GZIP,
    BROTLI,
    DEFLATE,
    NONE,
}

impl CompressionAlgorithm {
    pub fn is_none(&self) -> bool {
        match self {
            CompressionAlgorithm::NONE => true,
            _ => false
        }
    }

    pub fn or(self, next: CompressionAlgorithm) -> CompressionAlgorithm {
        match self {
            GZIP => self,
            BROTLI => self,
            DEFLATE => self,
            NONE => next
        }
    }

    pub fn is_some(&self) -> bool {
        !self.is_none()
    }

    pub fn from(str: String) -> CompressionAlgorithm {
        match str {
            str if str.contains("gzip") => GZIP,
            str if str.contains("deflate") => DEFLATE,
            str if str.contains("brotli") => BROTLI,
            _ => NONE
        }
    }

    pub fn supported_algorithms() -> Vec<String> {
        vec!("gzip".to_string(), "brotli".to_string(), "deflate".to_string())
    }

    pub fn to_string_for_content_encoding(&self) -> String {
        match self {
            GZIP => "gzip".to_string(),
            BROTLI => "br".to_string(),
            DEFLATE => "deflate".to_string(),
            CompressionAlgorithm::NONE => "none".to_string()
        }
    }

    pub fn to_string_for_transfer_encoding(&self) -> String {
        match self {
            GZIP => "gzip".to_string(),
            BROTLI => "brotli".to_string(),
            DEFLATE => "deflate".to_string(),
            CompressionAlgorithm::NONE => "".to_string()
        }
    }
}

#[allow(non_snake_case)]
pub fn write_message_to_wire(mut stream: &mut TcpStream, message: HttpMessage, request_options: RequestOptions) {
    match message {
        HttpMessage::Request(mut req) => {
            let chunked_encoding_desired = req.headers.has("Transfer-Encoding");
            let has_content_length = req.headers.has("Content-Length");
            let headers = ensure_content_length_or_transfer_encoding(req.headers, &req.body, &req.version, chunked_encoding_desired, has_content_length);
            let chunked_encoding_desired = headers.has("Transfer-Encoding");

            let compression = compression_from(headers.get("Content-Encoding").or(headers.get("Transfer-Encoding")));

            let headers = set_connection_header_if_needed_and_not_present(headers, chunked_encoding_desired);

            let start_line = format!("{} {} HTTP/{}.{}\r\n",
                                     req.method.value(),
                                     req.uri.to_string(),
                                     req.version.major,
                                     req.version.minor);

            let start_line_and_headers = format!("{}{}\r\n\r\n", start_line, headers.to_wire_string());

            match req.body {
                BodyString(str) => {
                    let is_version_1_1 = req.version == one_pt_one();
                    if chunked_encoding_desired && is_version_1_1 {
                        write_chunked_string(stream, start_line_and_headers, str.as_bytes(), req.trailers, compression);
                    } else {
                        write_string(stream, &compression, start_line_and_headers, str, headers, start_line)
                    }
                }
                BodyStream(ref mut reader) => {
                    if chunked_encoding_desired && req.version == one_pt_one() {
                        write_chunked_stream(stream, reader, start_line_and_headers, req.trailers, compression);
                    } else {
                        if compression.is_none() {
                            let mut chain = start_line_and_headers.as_bytes().chain(reader);
                            let _copy = copy(&mut chain, &mut stream).unwrap();
                        } else {
                            let mut writer = Vec::new();
                            let mut read = [0; 4096];
                            loop {
                                let bytes_read = reader.read(&mut read).unwrap();
                                if bytes_read == 0 {
                                    break;
                                }
                            }
                            compress(&compression, &mut writer, &read);
                            let headers = headers.replace(("Content-Length", writer.len().to_string().as_str()));
                            let start_line_and_headers = format!("{}{}\r\n\r\n", start_line, headers.to_wire_string());
                            let mut whole = start_line_and_headers.as_bytes().to_vec();
                            whole.append(&mut writer);
                            stream.write(&whole).unwrap();
                        }
                    }
                }
            }
        }
        HttpMessage::Response(mut res) => {
            let has_transfer_encoding = res.headers.has("Transfer-Encoding");
            let has_content_length = res.headers.has("Content-Length");
            let mut headers = ensure_content_length_or_transfer_encoding(res.headers, &res.body, &res.version, has_transfer_encoding, has_content_length);

            let compression = if headers.get("Content-Encoding").map(|ce| ce.to_lowercase() == "none").unwrap_or(false) {
                headers = headers.remove("Content-Encoding");
                CompressionAlgorithm::NONE
            } else {
                let compression_algorithm = request_options.write_response_compression()
                    .or(compression_from(headers.get("Transfer-Encoding")))
                    .or(compression_from(headers.get("Content-Encoding")));

                if compression_algorithm.is_some() && !headers.has("Transfer-Encoding") {
                    headers = headers.replace(("Content-Encoding", compression_algorithm.to_string_for_content_encoding().as_str()));
                } else if compression_algorithm.is_some() && headers.has("Transfer-Encoding") {
                    headers = headers.replace(("Transfer-Encoding", format!("{}, chunked", compression_algorithm.to_string_for_transfer_encoding()).as_str()))
                }
                compression_algorithm
            };

            if headers.has("TE") {
                headers = headers.remove("TE")
            };

            headers = headers.remove("Connection");

            let mut trailers = res.trailers;
            if !request_options.wants_trailers && !trailers.is_empty() {
                let as_str = request_options.expected_trailers.iter()
                    .map(|x| x.as_str()).collect::<Vec<&str>>();
                headers = headers.add_all(trailers.filter(as_str));
                trailers = Headers::empty();
            }

            let start_line = format!("HTTP/1.1 {} {}\r\n", &res.status.value(), &res.status.to_string());
            let status_and_headers = format!("{}{}\r\n\r\n", start_line, headers.to_wire_string());

            let chunked_encoding_desired = headers.has("Transfer-Encoding");

            match res.body {
                BodyString(str) => {
                    if chunked_encoding_desired && (res.version == one_pt_one()) {
                        write_chunked_string(stream, status_and_headers, str.as_bytes(), trailers, compression);
                    } else {
                        write_string(stream, &compression, status_and_headers, str, headers, start_line)
                    }
                }
                BodyStream(ref mut reader) => {
                    if chunked_encoding_desired && res.version == one_pt_one() {
                        write_chunked_stream(&mut stream, reader, status_and_headers, trailers, compression);
                    } else {
                        if compression.is_none() {
                            let mut chain = status_and_headers.as_bytes().chain(reader);
                            let _copy = copy(&mut chain, &mut stream).unwrap();
                        } else {
                            let mut writer = Vec::new();
                            let mut whole = Vec::new();
                            let _bytes_read = reader.read_to_end(&mut whole).unwrap();
                            compress(&compression, &mut writer, whole.as_slice());

                            let headers = headers.replace(("Content-length", writer.len().to_string().as_str()));
                            let headers = headers.replace(("Content-Encoding", compression.to_string_for_content_encoding().as_str()));
                            let status_and_headers = Response::status_line_and_headers_wire_string(&headers, &res.status);
                            let mut chain = status_and_headers.as_bytes().chain(writer.as_slice());
                            let _copy = copy(&mut chain, &mut stream).unwrap();
                        }
                    }
                }
            }
        }
    }
}

fn set_connection_header_if_needed_and_not_present(headers: Headers, chunked_encoding_desired: bool) -> Headers {
    if chunked_encoding_desired && headers.get("Connection").map(|h| !h.contains("TE")).unwrap_or(false) {
        headers.replace(("Connection", headers.get("Connection").map(|mut h| {
            h.push_str(", TE");
            h
        }).unwrap_or("TE".to_string()).as_str()))
    } else {
        headers
    }
}

fn write_string(stream: &mut TcpStream, compression: &CompressionAlgorithm, start_line_and_headers: String, body: &str, headers: Headers, start_line: String) {
    if compression.is_none() {
        let status_headers_and_body = [start_line_and_headers.as_bytes(), body.as_bytes()].concat();
        stream.write(status_headers_and_body.as_slice()).unwrap();
    } else {
        let mut writer = Vec::new();
        compress(&compression, &mut writer, body.as_bytes());
        let headers = headers.replace(("Content-Length", writer.len().to_string().as_str()));
        let mut start_line = start_line;
        start_line.push_str(headers.to_wire_string().as_str());
        start_line.push_str("\r\n\r\n");
        let mut whole = start_line.as_bytes().to_vec();
        whole.append(&mut writer);
        stream.write(&whole).unwrap();
    }
}

fn compression_from(option: Option<String>) -> CompressionAlgorithm {
    match option {
        Some(value) if value.contains("br") => CompressionAlgorithm::BROTLI,
        Some(value) if value.contains("gzip") => CompressionAlgorithm::GZIP,
        Some(value) if value.contains("deflate") => CompressionAlgorithm::DEFLATE,
        _ => CompressionAlgorithm::NONE,
    }
}

fn compress<'a>(compression: &'a CompressionAlgorithm, mut writer: &'a mut Vec<u8>, chunk: &'a [u8]) {
    match compression {
        CompressionAlgorithm::GZIP => { Codex::encode(chunk, &mut writer, GZIP); }
        CompressionAlgorithm::BROTLI => { Codex::encode(chunk, &mut writer, BROTLI); }
        CompressionAlgorithm::DEFLATE => { Codex::encode(chunk, &mut writer, DEFLATE); }
        CompressionAlgorithm::NONE => { writer.write_all(chunk).unwrap(); }
    }
}

fn decompress<'a>(compression: &'a CompressionAlgorithm, writer: &'a mut Vec<u8>, reader: &'a mut Vec<u8>) {
    match compression {
        GZIP | DEFLATE | BROTLI => { Codex::decode(reader, writer, compression); }
        NONE => { writer.write_all(reader).unwrap(); }
    }
}

#[allow(unused_assignments)]
pub fn write_chunked_string(stream: &mut TcpStream, mut first_line: String, chunk: &[u8], trailers: Headers, compression: CompressionAlgorithm) {
    let mut writer = Vec::new();
    let mut request = Vec::new();
    if compression.is_some() {
        compress(&compression, &mut writer, chunk);
        write_chunk_metadata(&mut first_line, writer.len());
        request = [first_line.as_bytes(), writer.as_slice(), "\r\n".as_bytes()].concat();
    } else {
        write_chunk_metadata(&mut first_line, chunk.len());
        request = [first_line.as_bytes(), chunk, "\r\n".as_bytes()].concat();
    }
    if !trailers.is_empty() {
        request.extend_from_slice(format!("0\r\n{}\r\n\r\n", trailers.to_wire_string()).as_bytes());
    } else {
        request.extend_from_slice("0\r\n\r\n".as_bytes());
    }
    stream.write(request.as_slice()).unwrap();
}

fn write_chunk_metadata(first_line: &mut String, length: usize) {
    let length_in_hex = format!("{:X}", length);
    first_line.push_str(length_in_hex.as_str());
    first_line.push_str("\r\n");
}

/*
https://doc.rust-lang.org/std/io/trait.Read.html#tymethod.read
It is not an error if the returned value n is smaller than the buffer size, even when the reader is not at the end of the stream yet.
This may happen for example because fewer bytes are actually available right now (e. g. being close to end-of-file) or because read() was interrupted by a signal.
 */
pub fn write_chunked_stream<'a>(mut stream: &mut TcpStream, reader: &mut Box<dyn Read + 'a>, first_line_and_headers: String, trailers: Headers, compression: CompressionAlgorithm) {
    if compression.is_some() {
        write_compressed_chunks(&mut stream, reader, &first_line_and_headers, &trailers, &compression);
    } else {
        write_simple_chunks(&mut stream, reader, first_line_and_headers, trailers);
    }
}

#[allow(unused_assignments)]
fn write_simple_chunks<'a>(mut stream: &mut TcpStream, reader: &mut Box<dyn Read + 'a>, first_line_and_headers: String, trailers: Headers) {
    let buffer = &mut [0 as u8; 16384];
    let mut bytes_read = reader.read(buffer).unwrap_or(0);
    let mut first_write = true;
    let mut temp = Vec::new();

    while bytes_read > 0 {
        temp = Vec::new();
        let length_in_hex = format!("{:X}", bytes_read);
        temp.extend_from_slice(length_in_hex.as_bytes());
        temp.push(b'\r');
        temp.push(b'\n');
        let chunk = &buffer[..bytes_read];
        temp.extend_from_slice(chunk);
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

    let mut end = vec!(b'0', b'\r', b'\n');
    if !trailers.is_empty() {
        end.extend_from_slice(format!("{}\r\n\r\n", trailers.to_wire_string()).as_bytes());
    } else {
        end.push(b'\r');
        end.push(b'\n');
    }
    stream.write(end.as_slice()).unwrap();
}

fn write_compressed_chunks<'a>(mut stream: &mut TcpStream, reader: &mut Box<dyn Read + 'a>, first_line_and_headers: &String, trailers: &Headers, compression: &CompressionAlgorithm) {
    let buffer = &mut [0 as u8; 16384];
    let mut bytes_read = reader.read(buffer).unwrap_or(0);
    let mut temp = Vec::new();

    while bytes_read > 0 {
        let chunk = &buffer[..bytes_read];
        temp.extend_from_slice(chunk);
        bytes_read = reader.read(buffer).unwrap();
    }

    let mut writer = Vec::new();
    compress(&compression, &mut writer, temp.as_slice());
    let compressed_length_in_hex = format!("{:X}", writer.len());
    let reversed = compressed_length_in_hex.chars().rev().collect::<String>();
    writer.insert(0, b'\n');
    writer.insert(0, b'\r');
    for byte in reversed.as_bytes() {
        writer.insert(0, *byte)
    }
    writer.push(b'\r');
    writer.push(b'\n');

    let mut end = vec!(b'0', b'\r', b'\n');
    if !trailers.is_empty() {
        end.extend_from_slice(format!("{}\r\n\r\n", trailers.to_wire_string()).as_bytes());
    } else {
        end.push(b'\r');
        end.push(b'\n');
    }
    let message = [first_line_and_headers.as_bytes(), writer.as_slice(), end.as_slice()].concat();
    let _copy = copy(&mut message.as_slice(), &mut stream).unwrap();
}


pub fn ensure_content_length_or_transfer_encoding(headers: Headers, body: &Body, version: &HttpVersion, chunked_encoding_desired: bool, has_content_length: bool) -> Headers {
    if chunked_encoding_desired && has_content_length {
        headers.remove("Content-Length")
    } else if !chunked_encoding_desired && !has_content_length {
        if body.is_body_string() {
            headers.add(("Content-Length", body.length().to_string().as_str()))
        } else if body.is_body_stream() && version == &one_pt_one() {
            headers.add(("Transfer-Encoding", "chunked"))
        } else {
            headers
        }
    } else {
        headers
    }
}

// something like gzip;q=0.9, deflate;q=0.8
pub fn most_desired_encoding(str: Option<String>) -> Option<String> {
    str.map(|v| {
        let mut ranked = v.split(", ")
            .map(|p| {
                let pair = p.split(";").map(|it| it.to_string()).collect::<Vec<String>>();
                if pair.len() > 1 {
                    let rank = pair[1].split("=").map(|it| it.to_string()).collect::<Vec<String>>();
                    if rank.len() > 1 {
                        Some((pair[0].clone(), rank[1].clone()))
                    } else { None }
                } else { None }
            })
            .filter(|p| p.is_some())
            .map(|x| x.unwrap())
            .filter(|y| CompressionAlgorithm::supported_algorithms().contains(&y.0))
            .collect::<Vec<(String, String)>>();

        ranked.sort_by(|x, y| x.1.parse::<f32>().unwrap().partial_cmp(&y.1.parse::<f32>().unwrap()).unwrap());
        ranked.reverse();
        ranked.first().map(|x| x.0.clone())
    }).unwrap_or(Some("none".to_string()))
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
    InvalidBoundaryDigit(String),
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
    pub fn request(method: Method, uri: Uri, headers: Headers) -> Request {
        Request { method, headers, body: Body::empty(), uri, version: HttpVersion { major: 1, minor: 1 }, trailers: Headers::empty() }
    }

    pub fn get(uri: Uri, headers: Headers) -> Request {
        Request { method: GET, headers, body: Body::empty(), uri, version: HttpVersion { major: 1, minor: 1 }, trailers: Headers::empty() }
    }

    pub fn post(uri: Uri<'a>, headers: Headers, body: Body<'a>) -> Request<'a> {
        Request { method: POST, headers, body, uri, version: HttpVersion { major: 1, minor: 1 }, trailers: Headers::empty() }
    }

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

    pub fn with_uri(self, uri: Uri<'a>) -> Request<'a> {
        Request {
            uri,
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

#[allow(non_snake_case)]
pub struct RequestOptions {
    pub desired_content_encoding: CompressionAlgorithm,
    pub transfer_encoding: CompressionAlgorithm,
    pub compression_from_TE_header: CompressionAlgorithm,
    pub content_encoding: CompressionAlgorithm,
    pub wants_trailers: bool,
    pub expected_trailers: Vec<String>,
}

#[allow(non_snake_case)]
impl RequestOptions {
    pub fn from(headers: &Headers) -> RequestOptions {
        RequestOptions {
            desired_content_encoding: compression_from(headers.get("Accept-Encoding")),
            transfer_encoding: compression_from(headers.get("Transfer-Encoding")),
            content_encoding: compression_from(headers.get("Content-Encoding")),
            compression_from_TE_header: compression_from(most_desired_encoding(headers.get("TE"))),
            wants_trailers: headers.get("TE").map(|t| t.contains("trailers")).unwrap_or(false),
            expected_trailers: headers.get("Trailer").map(|ts| ts.split(", ")
                .filter(|t| !DISALLOWED_TRAILERS.contains(&&*t.to_lowercase()))
                .map(|t| t.to_string())
                .collect::<Vec<String>>())
                .unwrap_or(vec!()),
        }
    }

    pub fn read_compression(&self) -> CompressionAlgorithm {
        self.content_encoding
            .or(self.transfer_encoding)
    }

    pub fn write_response_compression(&self) -> CompressionAlgorithm {
        self.desired_content_encoding
            .or(self.transfer_encoding)
            .or(self.compression_from_TE_header)
    }

    pub fn default() -> RequestOptions {
        RequestOptions {
            desired_content_encoding: NONE,
            transfer_encoding: NONE,
            content_encoding: NONE,
            compression_from_TE_header: NONE,
            wants_trailers: false,
            expected_trailers: vec!(),
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

    pub fn forbidden(headers: Headers, body: Body) -> Response {
        Response { headers, body, status: Forbidden, version: HttpVersion { major: 1, minor: 1 }, trailers: Headers::empty() }
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

#[derive(PartialEq, Debug)]
#[repr(u32)]
pub enum Status {
    OK = 200,
    MovedPermanently = 301,
    BadRequest = 400,
    LengthRequired = 411,
    NotFound = 404,
    Forbidden = 403,
    InternalServerError = 500,
    Unknown = 0,
}

impl Status {
    pub fn to_string(&self) -> String {
        match self {
            OK => "OK".to_string(),
            MovedPermanently => "Moved Permanently".to_string(),
            NotFound => "Not Found".to_string(),
            Forbidden => "Forbidden".to_string(),
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
            Forbidden => 403,
            NotFound => 404,
            InternalServerError => 500,
            _ => 500
        }
    }
    pub fn from(str: &str) -> Self {
        match str.to_lowercase().as_str() {
            "200" => OK,
            "301" => MovedPermanently,
            "400" => BadRequest,
            "403" => Forbidden,
            "404" => NotFound,
            "500" => InternalServerError,
            _ => Unknown
        }
    }
}

