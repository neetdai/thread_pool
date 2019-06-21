use super::event::Event;
use crossbeam_queue::ArrayQueue;
use std::sync::atomic::spin_loop_hint;
use std::sync::Arc;
use std::sync::{RwLock, Mutex, Condvar};
use std::thread::{current, spawn, JoinHandle, ThreadId};
// use std::thread::yield_now;

#[derive(Debug)]
enum Status {
    Stop,
    Running,
}

#[derive(Debug)]
pub(crate) struct Worker {
    // channel: & 'a Vec<Task>,
    handle: JoinHandle<()>,
    status: Arc<RwLock<Status>>,
    id: ThreadId,
}

impl Worker {
    pub(crate) fn new(channel: Arc<ArrayQueue<Event>>, condition: Arc<(Mutex<bool>, Condvar)>) -> Self {
        let status: Arc<RwLock<Status>> = Arc::new(RwLock::new(Status::Stop));
        let tmp: Arc<RwLock<Status>> = status.clone();
        let handle: JoinHandle<()> = spawn(move || 'run: loop {
            let &(ref lock, ref condi) = &*condition;
            let result = lock.lock().expect("worker.rs line 29 unlock");
            if *result == false {
                println!("waitting");
                condi.wait(result).expect("worker.rs line 31 can't wait");
            }

            match channel.pop() {
                Ok(event) => match event {
                    Event::Task(task) => match tmp.write() {
                        Ok(mut status) => {
                            *status = Status::Running;
                            task();
                        }
                        Err(error) => {
                            println!("{:?}", error);
                        }
                    },
                    Event::Stop => {
                        println!("stopping");
                        match tmp.write() {
                            Ok(mut status) => {
                                *status = Status::Stop;
                                
                                println!("stop");
                                break 'run;
                            }
                            Err(error) => {
                                println!("{:?}", error);
                            }
                        }
                        break 'run;
                    }
                    Event::Debug => {
                        println!("thread id = {:?}", current().id());
                    }
                },
                Err(_) => {
                    // println!("waitting");
                    spin_loop_hint();
                    
                    // yield_now();
                }
            }
        });

        let id: ThreadId = handle.thread().id();
        Worker { handle, status, id }
    }

    pub fn join(self) {
        self.handle.join();
    }
}
