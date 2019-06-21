extern crate thread_pool as tp;
use std::thread::{sleep};
use std::time::{Duration};
use thread_pool::thread_pool::thread_pool::{ThreadPool, Builder};
use std::thread::{current};

fn main() {
    let mut tp: ThreadPool = Builder::default().with_worker_total(2).with_channel_total(100).build();
    tp.run();

    tp.push(move || {
        let time: Duration = Duration::from_secs(10);
        println!("{:?}", current().id());
        sleep(time);
        println!("wake");
    }).map_err(|error| {
        println!("{:?}", error);
    });

    tp.push(move || {
        println!("1");
    });

    tp.push(move || {
        println!("2");
    });
    tp.join_all();
    // sleep(Duration::new(5, 0));
}
