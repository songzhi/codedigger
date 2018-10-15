/// 调用parser解析根目录文件树的所有文件
extern crate threadpool;

use cache;
use config;
use parser;
use self::threadpool::ThreadPool;
use std::fs;
use std::io;
use std::sync::mpsc::{channel, Receiver, Sender};

pub struct Scheduler {
    pub init_path: String,
    pub results: Vec<parser::CodeStats>,
    pub errors: Vec<io::Error>,
    threadpool: ThreadPool,
    config: config::Config,
    cache_manager: cache::CacheManager,
}

impl Scheduler {
    pub fn new(path: &str, threadpool: ThreadPool, config: config::Config, cache: cache_manager::CacheManager) -> Self {
        Self {
            init_path: path.to_string(),
            results: vec![],
            errors: vec![],
            threadpool,
            config,
            cache_manager,
        }
    }
    pub fn start(&mut self) {
        let (tx, rx) = channel();
        self.schedule(self.init_path.as_str(), tx.clone());
        while let Ok(result) = rx.recv() {
            match result {
                Ok(stats) => {
                    self.results.push(stats);
                }
                Err(err) => {
                    self.errors.push(err);
                }
            }
        }
    }
    /// 递归调用方法
    fn schedule(&self, path: &str, tx: Sender<Result<parser::CodeStats, io::Error>>) {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    if let Ok(file_type) = entry.file_type() {
                        if file_type.is_dir() {
                            self.schedule(entry.path().to_str().unwrap(), tx.clone())
                        } else if file_type.is_file() {
                            let path = entry.path().to_str().unwrap();
                            if let Some(stats) = self.cache_manager.get_cache(path) {
                                tx.send(Ok(stats));
                            } else {
                                if let Some(tokens) = self.config.get_comment_tokens(path) {
                                    self.tp.execute(move || {
                                        tx.send(parser::Parser::new(path, tokens).parse())
                                            .expect("parse failed")
                                    })
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}