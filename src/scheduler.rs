extern crate num_cpus;
/// 调用parser解析根目录文件树的所有文件
extern crate threadpool;

// TODO: 注释
// TODO: 使用failure crate统一Error
// TODO: benchmark
use std::{
    collections::BTreeMap,
    fs,
    io,
    path::{Path, PathBuf},
    sync::mpsc::{channel, Sender},
};

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

use super::cache;
use super::config;
use super::parser::{CodeStat, CommonParser, Parser, ParserState};

use self::threadpool::ThreadPool;

pub struct Scheduler {
    pub init_path: PathBuf,
    pub parsers: BTreeMap<PathBuf, ParserState>,
    task_left: u64,
    threadpool: ThreadPool,
    config: config::Config,
    cache_manager: cache::CacheManager,
    multi_bars: MultiProgress,
}

impl Scheduler {
    pub fn new(path: &Path, threadpool: ThreadPool, config: config::Config, cache_manager: cache::CacheManager) -> Self {
        Self {
            init_path: path.to_path_buf(),
            parsers: BTreeMap::new(),
            task_left: 0,
            multi_bars: MultiProgress::new(),
            threadpool,
            config,
            cache_manager,
        }
    }
    pub fn start(mut self) -> Result<BTreeMap<PathBuf, ParserState>, io::Error> {
        let (tx, rx) = channel();
        self.schedule(self.init_path.as_path(), tx.clone())?;
        drop(tx);
        while let Ok((path, state)) = rx.recv() {
            match state {
                ParserState::Ready => self.task_left += 1,
                ParserState::Complete(_) => self.task_left -= 1,
                _ => {}
            }
            self.parsers.insert(path, state);
        };
        self.multi_bars.join();
        Ok(self.parsers)
    }
    /// 递归调用方法
    fn schedule(&self, path: &Path, tx: Sender<(PathBuf, ParserState)>) -> Result<(), io::Error> {
        let entries = fs::read_dir(path)?;
        for entry in entries {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let path = entry.path();
            if file_type.is_dir() {
                self.schedule(path.as_path(), tx.clone())?;
            } else if file_type.is_file() {
                let tx = tx.clone();
                tx.send((path.clone(), ParserState::Ready));
                if let Some(stats) = self.cache_manager.get_cache(path.as_path()) {
                    tx.send((path, ParserState::Complete(stats)));
                } else if let Some(tokens) = self.config.get_comment_tokens(path.extension().unwrap().to_str().unwrap()) {
                    let path = path.clone();
                    let file_size = entry.metadata().unwrap().len();
                    let bar = self.multi_bars.add(ProgressBar::new(file_size));
                    bar.set_style(self.config.progress_style.clone());
                    bar.set_draw_delta(file_size / 1000);
                    let file_name = entry.path().file_name().unwrap().to_str().unwrap().to_string();
                    self.threadpool.execute(move || {
                        bar.set_message(&format!("Parsing:{}", file_name));
                        tx.send((path.clone(), ParserState::Parsing)).expect("发送失败");
                        let result = CommonParser::new(path.as_path(), tokens, bar).parse();
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