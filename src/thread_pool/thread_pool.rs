use super::event::Event;
use super::worker::Worker;
use crossbeam_queue::ArrayQueue;
use failure::{err_msg, Error as FailError};
use num_cpus;
use std::cmp::Ordering;
use std::default::Default;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use std::sync::{Mutex, Condvar};

const MAX_TASK_TOTAL: usize = 100;

#[derive(Debug)]
pub struct Builder {
    worker_total: usize,
    channel_total: usize,
}

impl Default for Builder {
    fn default() -> Self {
        Builder {
            worker_total: num_cpus::get(),
            channel_total: 100,
        }
    }
}

impl Builder {
    pub fn with_worker_total(mut self, total: usize) -> Self {
        self.worker_total = total;
        self
    }

    pub fn with_channel_total(mut self, total: usize) -> Self {
        self.channel_total = total;
        self
    }

    pub fn build(self) -> ThreadPool {
        ThreadPool {
            worker_total: self.worker_total,
            channels: Vec::new(),
            queue_total: self.channel_total,
        }
    }
}

#[derive(Debug)]
pub struct ThreadPool {
    worker_total: usize,
    queue_total: usize,
    channels: Vec<(Arc<ArrayQueue<Event>>, Worker, Arc<(Mutex<bool>, Condvar)>)>,
}

impl ThreadPool {
    pub fn new() -> Self {
        ThreadPool::default()
    }

    pub fn with_worker_total(total: usize) -> Self {
        let mut tp: ThreadPool = ThreadPool::default();
        tp.worker_total = total;
        tp
    }

    pub fn run(&mut self) {
        self.build_channel();
    }

    fn build_channel(&mut self) {
        for _ in 0..self.worker_total {
            let list: Arc<ArrayQueue<Event>> = Arc::new(ArrayQueue::new(self.queue_total));
            let condition: Arc<(Mutex<bool>, Condvar)> = Arc::new((Mutex::new(false), Condvar::new()));
            let worker: Worker = Worker::new(list.clone(), condition.clone());
            self.channels.push((list, worker, condition));
        }
    }

    pub fn push<T>(&self, func: T) -> Result<(), FailError>
    where
        T: Fn() + 'static + Send + Sync,
    {
        let task: Event = Event::with_task(func);
        match self
            .channels
            .iter()
            .min_by(|item1, item2| -> Ordering { item1.0.len().cmp(&item2.0.len()) })
        {
            Some(item) => {
                let &(ref lock, ref condi) = &*item.2;
                let mut result = lock.lock().expect("thread_pool.rs line 93 unlock");
                if *result == false {
                    condi.notify_one();
                    *result = true;
                }

                item.0
                .push(task)
                .map(|_| ())
                .map_err(|error| FailError::from(error))
            },
            None => Err(err_msg("not found a channel to push")),
        }
    }

    pub fn join_all(self) {
        self.channels.into_iter().for_each(|item| {
            let wait_time: Duration = Duration::from_millis(10);
            loop {
                let &(ref lock, ref condi) = &*item.2;
                let mut result = lock.lock().expect("thread_pool.rs line 113 unlock");
                if *result == false {
                    condi.notify_one();
                    *result = true;
                }
                match item.0.push(Event::Stop) {
                    Ok(_) => break,
                    Err(_) => sleep(wait_time),
                }
            }
            item.1.join();
        });
    }
}

impl Default for ThreadPool {
    fn default() -> Self {
        ThreadPool {
            worker_total: num_cpus::get(),
            channels: Vec::new(),
            queue_total: MAX_TASK_TOTAL,
        }
    }
}

// impl Drop for ThreadPool {
//     fn drop(&mut self) {
//         self.channels.iter().for_each(|item| {
//             item.push(Event::Stop).expect("can't stop thread, because channel is full");
//         });
//     }
// }