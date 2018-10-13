use std::fmt::{Display, Error, Formatter};
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::prelude::*;

#[derive(Clone)]
enum CommentToken {
    Common(String),
    MultiLine(String, String),
}

pub struct CommonParser {
    path: String,
    comment_tokens: Vec<CommentToken>,
}

#[derive(Clone, Debug)]
pub struct CodeStats {
    code: u64,
    blank: u64,
    comment: u64,
    path: String,
    ext: String
}

pub trait Parser {
    fn parse(self) -> io::Result<CodeStats>;
    fn parse_line(&self, line: &str, stats: &mut CodeStats, being_in_multiline_token: &mut Option<CommentToken>);
}

impl CommonParser {
    fn new(path: &str, comment_tokens: Vec<CommentToken>) -> Self {
        Self {
            path: path.to_string(),
            comment_tokens,
        }
    }
}

impl Display for CodeStats {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{{ Code: {}, Comment: {}, Blank: {} }}", self.code, self.comment, self.blank)
    }
}

impl Parser for CommonParser {
    fn parse(self) -> io::Result<CodeStats> {
        let file = File::open(self.path.as_str())?;
        let reader = BufReader::new(file);
        let mut stats = CodeStats {
            code: 0u64,
            blank: 0u64,
            comment: 0u64,
            path: self.path.clone(),
            ext: String::from(self.path.rsplit(".").nth(0).unwrap())
        };
        let mut being_in_multiline_token: Option<CommentToken> = None;
        for line in reader.lines() {
            if let Ok(s) = line {
                self.parse_line(s.as_str(), &mut stats, &mut being_in_multiline_token);
            }
        }
        Ok(stats)
    }
    fn parse_line(&self, line: &str, stats: &mut CodeStats, being_in_multiline_token: &mut Option<CommentToken>) {
        match being_in_multiline_token {
            Some(being_in_token) => {
                for token in self.comment_tokens.iter() {
                    if let CommentToken::MultiLine(_s, e) = token {
                        if line.ends_with(e.as_str()) {
                            if let CommentToken::MultiLine(s, _e) = being_in_token {
                                if s == _s {
                                    stats.comment += 1;
                                    return;
                                }
                            }
                        }
                    }
                }
                stats.comment += 1;
            }
            None => {
                if line.trim().is_empty() {
                    stats.blank += 1;
                    return;
                }
                for token in self.comment_tokens.iter() {
                    match token {
                        CommentToken::Common(s) => {
                            if line.starts_with(s.as_str()) {
                                stats.comment += 1;
                                return;
                            }
                        }
                        CommentToken::MultiLine(s, _e) => {
                            if line.starts_with(s.as_str()) {
                                stats.comment += 1;
                                *being_in_multiline_token = Some((*token).clone());
                                return;
                            }
                        }
                    }
                }
                stats.code += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        let path = "G:\\code\\suggest.js";
        let comment_tokens = vec![CommentToken::Common("//".to_string()), CommentToken::MultiLine("/*".to_string(), "*/".to_string())];
        let parser = CommonParser::new(path, comment_tokens);
        let stats = parser.parse().unwrap();
        println!("{:#?}", stats);
    }
}