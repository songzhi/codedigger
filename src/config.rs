extern crate indicatif;
#[macro_use]
extern crate serde_derive;
extern crate toml;

use std::collections::BTreeMap;
use std::io::Error;

use indicatif::{ProgressBar, ProgressStyle};

use parser::CommentToken;

pub struct Config {
    pub progress_style: ProgressStyle,
    pub token_map: BTreeMap<String, Vec<CommentToken>>,
}


impl Config {
    pub fn new() -> Self {
        let progress_style = ProgressStyle::default_bar()
            .template("{bar:40.cyan/red} {pos:>7}/{len:7} {msg}")
            .progress_chars("##-");
        token_map.insert("js".to_string(), vec![CommentToken::Common("//".to_string()), CommentToken::MultiLine("/*".to_string(), "*/".to_string())]);
        Self {
            progress_style,
            token_map: self::get_token_map("g:/rust/codedigger/src/token_map.toml"),
        }
    }
    pub fn get_comment_tokens(&self, path: &str) -> Option<Vec<CommentToken>> {
        unimplemented!()
    }
    fn get_token_map(filename: &str) -> BTreeMap<String, Vec<CommentToken>> {
        unimplemented!()
    }
    pub fn set_comment_token(&self, filename: &str) -> Result<(), Error> {
        unimplemented!()
    }
}
