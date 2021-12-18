use std::net::{TcpListener, TcpStream};
use std::{str, thread};
use std::borrow::Borrow;
use std::io::{copy, Read, Write};
use std::sync::{Arc, mpsc, Mutex};
use std::sync::mpsc::Receiver;
use crate::headers::add_header;
use crate::httphandler::{HttpHandler};
use crate::httpmessage::{Body, content_length_header, get, header, HttpMessage, Request, request_from, Response};
use crate::httpmessage::Body::{BodyStream, BodyString};
use crate::server::Message::NewJob;

pub struct Server {}

impl Server {
    pub fn new(http_handler: HttpHandler, port: u32, pool: Option<ThreadPool>) {
        let addr = format!("127.0.0.1:{}", port);
        let listener = TcpListener::bind(addr).unwrap();

        let call_handler = |mut stream: TcpStream, handler: HttpHandler| {
            let mut buffer = [0 as u8; 16384];
            stream.read(&mut buffer).unwrap();
            let mut request = request_from(&buffer, stream.try_clone().unwrap()).unwrap();

            let mut response = handler(request);
            let mut returning: String = response.resource_and_headers();

            match response.body {
                BodyString(mut body_string) => {
                    returning.push_str(&body_string);
                    returning.push_str("\r\n");
                    &stream.write(returning.as_bytes());
                }
                BodyStream(ref mut body_stream) => {
                    &stream.write(returning.as_bytes());
                    copy(body_stream, &mut stream);
                }
                _ => {}
            }

            stream.flush().unwrap();
        };

        match pool {
            Some(thread_pool) => {
                for stream in listener.incoming() {
                    thread_pool.execute(move || {
                        call_handler(stream.unwrap(), http_handler)
                    });
                }
            }
            _ => {
                thread::spawn(move || {
                    for stream in listener.incoming() {
                        call_handler(stream.unwrap(), http_handler)
                    }
                });
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

