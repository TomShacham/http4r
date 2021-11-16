use std::net::{TcpListener, TcpStream};
use std::{str, thread};
use std::borrow::Borrow;
use std::io::{Read, Write};
use std::sync::{Arc, mpsc, Mutex, MutexGuard};
use std::sync::mpsc::Receiver;
use crate::httphandler::{HttpHandler};
use crate::httpmessage::{Method, Request, Response};
use crate::httpmessage::Method::GET;
use crate::server::Message::NewJob;

pub struct Server {
    listener: TcpListener,
    handler: HttpHandler,
    port: u32,
}

impl Server {
    pub fn new(handler: HttpHandler, port: u32) {
        let addr = format!("127.0.0.1:{}", port);
        let listener = TcpListener::bind(addr).unwrap();
        let pool = ThreadPool::new(4);

        let handle_connection = |mut stream: TcpStream, h: HttpHandler| {
            let mut buffer = [0; 1024];

            stream.read(&mut buffer).unwrap();

            let string = str::from_utf8(&buffer).unwrap();
            println!("got request in server {}", &string);

            let request = Request::from(string);

            let response = h(request);

            let has_content_length = response.headers.iter().any(|(name, value)| name == "Content-Length");
            println!("has content length {}", has_content_length.to_string());
            if !has_content_length {
                let mut with_content_length = response.headers.clone();
                with_content_length.push(("Content-Length".to_string(), response.body.len().to_string()));
                println!("response body is {}", response.body.to_string());
                let response1 = Response {
                    headers: with_content_length,
                    ..response
                };
                let response_string: String = response1.into();
                stream.write(response_string.as_bytes()).unwrap();
                stream.flush().unwrap();
            } else {
                let string1: String = response.into();
                stream.write(string1.as_bytes()).unwrap();
                stream.flush().unwrap();
            }

        };

        thread::spawn(move || {
            for stream in listener.incoming() {
                handle_connection(stream.unwrap(), handler);
            }
        });
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

