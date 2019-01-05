use std::{
    collections::BTreeMap,
    path::{Path, PathBuf}
};

use super::parser::CodeStat;

pub struct CacheManager {
    cache: BTreeMap<PathBuf, CodeStat>
}

impl CacheManager {
    pub fn get_cache(&self, path: &Path) -> Option<CodeStat> {
        unimplemented!()
    }
    pub fn set_cache(&mut self, code_stat: CodeStat) {
        self.cache.insert(code_stat.path.clone(), code_stat);
    }
}