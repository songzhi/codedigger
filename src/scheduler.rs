/// 调用parser解析根目录文件树的所有文件
extern crate threadpool;

use std::{
    collections::BTreeMap,
    fs,
    io,
    sync::mpsc::{channel, Sender},
};

use super::cache;
use super::config;
use super::parser::{CodeStat, CommonParser, Parser, ParserState};

use self::threadpool::ThreadPool;

pub struct Scheduler {
    pub init_path: String,
    pub parsers: BTreeMap<String, ParserState>,
    parser_left_count: u64,
    threadpool: ThreadPool,
    config: config::Config,
    cache_manager: cache::CacheManager,
}

impl Scheduler {
    pub fn new(path: &str, threadpool: ThreadPool, config: config::Config, cache_manager: cache::CacheManager) -> Self {
        Self {
            init_path: path.to_string(),
            parsers: BTreeMap::new(),
            parser_left_count: 0,
            threadpool,
            config,
            cache_manager,
        }
    }
    pub fn start(mut self) -> Result<BTreeMap<String, ParserState>, io::Error> {
        let (tx, rx) = channel();
        self.schedule(self.init_path.as_str(), tx.clone())?;
        while let Ok((path, state)) = rx.recv() {
            self.parsers.insert(path, state);
            match state {
                ParserState::Ready => self.parser_left_count += 1,
                ParserState::Complete(_) => self.parser_left_count -= 1,
                _ => {}
            }
            if !self.parser_left_count {
                break
            }
        };
        Ok(self.parsers)
    }
    /// 递归调用方法
    fn schedule(&self, path: &str, tx: Sender<(String, ParserState)>) -> Result<(), io::Error> {
        let path = path.trim();
        let entries = fs::read_dir(path)?;
        for entry in entries {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let path = entry.path().to_str().unwrap().to_string();
            if file_type.is_dir() {
                self.schedule(&path, tx.clone())?;
            } else if file_type.is_file() {
                let tx = tx.clone();
                if let Some(stats) = self.cache_manager.get_cache(&path) {
                    tx.send((path, ParserState::Complete(stats)));
                } else if let Some(tokens) = self.config.get_comment_tokens(&path) {
                    tx.send((path.clone(), ParserState::Ready));
                    let path = path.clone();
                    self.threadpool.execute(move || {
                        let result = CommonParser::new(path.as_str(), tokens).parse();
                        match result {
                            Ok(stats) => tx.send((path, ParserState::Complete(stats))).expect("发送失败"),
                            Err(err) => tx.send((path, ParserState::Error(err))).expect("发送失败")
                        };
                    })
                }
            }
        }
        Ok(())
    }
}