extern crate indicatif;
extern crate toml;

use std::{
    collections::BTreeMap,
    fs::File,
    io::Read,
    path::{Path, PathBuf}
};

use super::parser::CommentToken;

use self::indicatif::ProgressStyle;

pub struct Config {
    pub progress_style: ProgressStyle,
    pub token_map: BTreeMap<String, Vec<CommentToken>>,
}


impl Config {
    pub fn new(config_file_name: &str) -> Self {
        let progress_style = ProgressStyle::default_bar()
            .template("{bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .progress_chars("##-");
        let token_map_file = PathBuf::new();
        Self {
            progress_style,
            token_map: Self::get_token_map(token_map_file.as_path()).expect("注释Token配置文件不存在"),
        }
    }
    pub fn get_comment_tokens(&self, ext: &str) -> Option<Vec<CommentToken>> {
        self.token_map.get(ext).map(|vec| vec.to_vec())
    }
    pub fn get_token_map(path: &Path) -> Option<BTreeMap<String, Vec<CommentToken>>> {
        let mut file = File::open(path).ok()?;
        let mut buf = String::with_capacity(file.metadata().ok()?.len() as usize);
        file.read_to_string(&mut buf).ok()?;
        let config: toml::Value = toml::from_str(&buf).ok()?;
        let config = config.as_table()?;
        let mut result_map = BTreeMap::new();
        for (ext, table) in config {
            let table = table.as_table()?;
            let mut tokens = Vec::new();
            let common = table.get("line")?.as_array()?;
            let multi = table.get("block")?.as_array()?;
            for token in common {
                let token = token.as_str()?;
                tokens.push(CommentToken::Line(token.to_string()));
            }
            for pair in multi {
                let pair = pair.as_array()?;
                let (token1, token2) = (pair[0].as_str().unwrap(), pair[1].as_str().unwrap());
                tokens.push(CommentToken::Block(token1.to_string(), token2.to_string()));
            }
            result_map.insert(ext.clone(), tokens);
        }
        Some(result_map)
    }
    pub fn set_comment_token(&mut self, path: &Path) {
        if let Some(mut map) = Self::get_token_map(path) {
            self.token_map.append(&mut map);
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_token_map() {
        let map = Config::get_token_map(Path::new("src/token_map.toml")).unwrap();
        assert_eq!(map.get("py").unwrap().len(), 3);
        println!("{:#?}", map);
    }
}