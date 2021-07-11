


/*


------------------------------------------------------------------------------------------
GPS emulator for sending packet and testing device microservice socket server scalability
------------------------------------------------------------------------------------------



https://blog.softwaremill.com/multithreading-in-rust-with-mpsc-multi-producer-single-consumer-channels-db0fc91ae3fa
https://danielkeep.github.io/tlborm/book/
https://cetra3.github.io/blog/implementing-a-jobq/
https://cetra3.github.io/blog/implementing-a-jobq-with-tokio/
https://docs.rs/tokio/1.7.1/tokio/sync/index.html
https://blog.logrocket.com/macros-in-rust-a-tutorial-with-examples/
https://blog.logrocket.com/procedural-macros-in-rust/
https://github.com/cksac/dataloader-rs
http://gradebot.org/doc/ipur/trait.html
https://doc.rust-lang.org/std/sync/struct.Arc.html
https://doc.rust-lang.org/std/sync/struct.Mutex.html
http://web.mit.edu/rust-lang_v1.25/arch/amd64_ubuntu1404/share/doc/rust/html/book/first-edition/procedural-macros.html
https://dev.to/5422m4n/rust-macro-rules-in-practice-40ne



TODO - custom derive macro or proc_macro for my own traits using trait scope orphan rule, closures, asynchronous (async move) and multithreaded pattern to simulate thousand of GPSes


*/



use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;


fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(move || {
            send_packet_task()
        });
    }

    println!("Shutting down.");
}

fn send_packet_task(){
    // TODO - tasks/job : connect to device socket server to send the packet
    // TODO - packet   : 000000000000003608010000016B40D8EA30010000000000000000000000000000000105021503010101425E0F01F10000601A014E0000000000000000010000C7CF
    // ...
}


pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.send(Message::NewJob(job)).unwrap();
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

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
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
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}
