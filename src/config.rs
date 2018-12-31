extern crate indicatif;
extern crate toml;

use std::collections::BTreeMap;
use std::error::Error;
use std::fs::File;
use std::io::Read;

use parser::CommentToken;

use self::indicatif::{ProgressBar, ProgressStyle};

pub struct Config {
    pub progress_style: ProgressStyle,
    pub token_map: BTreeMap<String, Vec<CommentToken>>,
}


impl Config {
    pub fn new(config_file_name: &str) -> Self {
        let progress_style = ProgressStyle::default_bar()
            .template("{bar:40.cyan/red} {pos:>7}/{len:7} {msg}")
            .progress_chars("##-");
        let token_map_file = String::new();
        Self {
            progress_style,
            token_map: Self::get_token_map(&token_map_file).expect("注释Token配置文件不存在"),
        }
    }
    pub fn get_comment_tokens(&self, path: &str) -> Option<Vec<CommentToken>> {
        unimplemented!()
    }
    pub fn get_token_map(path: &str) -> Option<BTreeMap<String, Vec<CommentToken>>> {
        let mut file = File::open(path).ok()?;
        let mut buf = String::with_capacity(file.metadata().ok()?.len() as usize);
        file.read_to_string(&mut buf).ok()?;
        let config: toml::Value = toml::from_str(&buf).ok()?;
        let config = config.as_table()?;
        let mut result_map = BTreeMap::new();
        for (ext, table) in config {
            let table = table.as_table()?;
            let mut tokens = Vec::new();
            let common = table.get("common")?.as_array()?;
            let multi = table.get("multi")?.as_array()?;
            for token in common {
                let token = token.as_str()?;
                tokens.push(CommentToken::Common(token.to_string()));
            }
            for pair in multi {
                let pair = pair.as_array()?;
                let (token1, token2) = (pair[0].as_str().unwrap(), pair[1].as_str().unwrap());
                tokens.push(CommentToken::MultiLine(token1.to_string(), token2.to_string()));
            }
            result_map.insert(ext.clone(), tokens);
        }
        Some(result_map)
    }
    pub fn set_comment_token(&self, filename: &str) -> Result<(), ()> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_token_map() {
        let map = Config::get_token_map("src/token_map.toml").unwrap();
        assert_eq!(map.get("py").unwrap().len(), 3);
        println!("{:#?}", map);
    }
}