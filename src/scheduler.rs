extern crate threadpool;

use parser;
use std::sync::mpsc::{channel, Receiver, Sender};
use threadpool::ThreadPool;

pub struct Scheduler {
    pub init_path: String,
    pub results: Vec<parser::CodeStats>,
    n_workers: usize,
    tp: threadpool::ThreadPool,
}

impl Scheduler {
    pub fn new(path: &str) -> Self {
        Self::new_with_n_workers(path, 8)
    }
    pub fn new_with_n_workers(path: &str, n_workers: usize) -> Self {
        Self {
            init_path: path.to_string(),
            results: vec![],
            n_workers,
            tp: threadpool::ThreadPool::new(n_workers),
        }
    }
    pub fn start(&mut self) {
        let (tx, rx) = channel();
        self.schedule(self.init_path.as_str(), tx.clone());
        while let Ok(res) = rx.recv() {
            self.results.push(res);
        }
    }
    /// 递归调用方法
    fn schedule(&self, path: &str, tx: Sender<parser::CodeStats>) {}
}