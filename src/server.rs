use std::net::{TcpListener, TcpStream};
use std::{str, thread};
use std::borrow::Borrow;
use std::io::{copy, Read, Write};
use std::ops::Deref;
use std::sync::{Arc, mpsc, Mutex};
use std::sync::mpsc::Receiver;
use crate::headers::add_header;
use crate::httphandler::{HttpHandler};
use crate::httpmessage::{bad_request, Body, content_length_header, get, header, HttpMessage, length_required, ok, Request, request_from, RequestError, Response};
use crate::httpmessage::Body::{BodyStream, BodyString};
use crate::server::Message::NewJob;

pub struct Server {}

pub struct ServerOptions {
    pub port: Option<u32>,
    pub pool: Option<ThreadPool>,
}

impl Server {
    pub fn new(http_handler: HttpHandler, mut options: ServerOptions) {
        let addr = format!("127.0.0.1:{}", options.port.get_or_insert(7878));
        let listener = TcpListener::bind(addr).unwrap();

        let call_handler = |mut stream: TcpStream, handler: HttpHandler, buffer: &mut [u8]| {
            stream.read(buffer).unwrap();
            let mut result = request_from(&buffer, stream.try_clone().unwrap());

            match result {
                Err(RequestError::HeadersTooBig(msg)) => {
                    let mut response = bad_request(vec!(), BodyString(msg));
                    Self::write_response_to_wire(&mut stream, response)
                },
                Err(RequestError::NoContentLengthOrTransferEncoding(msg)) => {
                    let mut response = length_required(vec!(), BodyString(msg));
                    Self::write_response_to_wire(&mut stream, response)
                },
                Ok(request) => {
                    let mut response = handler(request);
                    Self::write_response_to_wire(&mut stream, response)
                }
            }

            stream.flush().unwrap();
        };

        match options.pool {
            Some(thread_pool) => {
                for stream in listener.incoming() {
                    thread_pool.execute(move || {
                        let mut buffer = &mut [0 as u8; 16384];
                        call_handler(stream.unwrap(), http_handler, buffer)
                    });
                }
            }
            _ => {
                thread::spawn(move || {
                    for stream in listener.incoming() {
                        let mut buffer = &mut [0 as u8; 16384];
                        call_handler(stream.unwrap(), http_handler, buffer)
                    }
                });
            }
        }
    }

    fn write_response_to_wire(mut stream: &mut TcpStream, mut response: Response) {
        let mut returning: String = response.resource_and_headers();

        match response.body {
            BodyString(body_string) => {
                returning.push_str(&body_string);
                &stream.write(returning.as_bytes());
            }
            BodyStream(ref mut body_stream) => {
                &stream.write(returning.as_bytes());
                copy(body_stream, &mut stream);
            }
        }
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)))
        }

        ThreadPool {
            workers,
            sender,
        }
    }

    pub fn execute<F>(&self, f: F)
        where
            F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers.");

        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        println!("Shutting down all workers.");

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

enum Message {
    NewJob(Job),
    Terminate,
}

pub struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Message>>>) -> Worker {
        Worker {
            id,
            thread: Some(thread::spawn(move || loop {
                let message = receiver.lock().unwrap().recv().unwrap();

                match message {
                    Message::NewJob(job) => {
                        println!("Worker {} got a job; executing.", id);
                        job();
                    }
                    Message::Terminate => {
                        println!("Worker {} was told to terminate.", id);
                        break;
                    }
                }
            })),
        }
    }
}

