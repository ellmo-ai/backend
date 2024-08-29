#![allow(dead_code)]

use crossbeam_channel::{unbounded, Receiver, Sender};
use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    pub static ref JOB_QUEUE: Mutex<JobQueue> = Mutex::new(JobQueue::new());
}

#[async_trait::async_trait]
pub trait Job {
    async fn execute(&self);
}

pub struct JobQueue {
    sender: Sender<Box<dyn Job + Send>>,
}

impl JobQueue {
    fn new() -> Self {
        let (sender, receiver) = unbounded();
        let queue = JobQueue { sender };
        queue.start_worker(receiver);
        queue
    }

    fn start_worker(&self, receiver: Receiver<Box<dyn Job + Send>>) {
        tokio::spawn(async move {
            while let Ok(job) = receiver.recv() {
                job.execute().await;
            }
        });
    }

    pub fn add_job(&self, job: Box<dyn Job + Send>) {
        self.sender.send(job).expect("Failed to send job");
    }
}
