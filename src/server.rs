use std::net::{TcpListener, TcpStream};
use std::{str, thread};
use std::io::{Read, Write};
use std::sync::{Arc, mpsc, Mutex};
use std::sync::mpsc::Receiver;
use crate::headers::add_header;
use crate::httphandler::{HttpHandler};
use crate::httpmessage::{get, HttpMessage, Request, Response};
use crate::httpmessage::Body::{BodyStream, BodyString};
use crate::server::Message::NewJob;

pub struct Server {}

impl Server {
    pub fn new(http_handler: HttpHandler, port: u32, pool: Option<ThreadPool>) {
        let addr = format!("127.0.0.1:{}", port);
        let listener = TcpListener::bind(addr).unwrap();

        let handle_connection = |mut stream: TcpStream, handler: HttpHandler| {
            let buffer = read_to_buffer(&mut stream);
            let string = str::from_utf8(&buffer).unwrap();
            let request = Request::from(string);
            let response = handler(request);

            let mut returning: String = response.resource_and_headers();
            match response.body {
                BodyString(body_string) => {
                    returning.push_str(&body_string);
                    returning.push_str("\r\n");
                    stream.write(returning.as_bytes()).unwrap();
                },
                BodyStream(mut body_stream) => {
                    stream.write(returning.as_bytes());
                    let buffer_out = read_to_buffer(&mut body_stream);
                    stream.write(&buffer_out);
                }
            }
            stream.flush().unwrap();
        };

        match pool {
            Some(thread_pool) => {
                for stream in listener.incoming() {
                    thread_pool.execute(move || {
                        handle_connection(stream.unwrap(), http_handler)
                    });
                }
            }
            _ => {
                thread::spawn(move || {
                    for stream in listener.incoming() {
                        handle_connection(stream.unwrap(), http_handler)
                    }
                });
            }
        }
    }
}

pub fn read_to_buffer(stream: &mut TcpStream) -> [u8; 4096] {
    let mut buffer: [u8; 4096] = [0; 4096];
    stream.try_clone().unwrap()
        .read(&mut buffer).unwrap();
    buffer
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

