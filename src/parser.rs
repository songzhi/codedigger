use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::fmt::{Display, Formatter, Error};

enum CommentToken {
    Common(String),
    MultiLine(String, String),
}

enum LineType {
    Blank,
    Comment,
    Code,
}

struct CommonParser {
    reader: BufReader<File>,
    comment_tokens: Vec<CommentToken>,
    is_in_comment: bool,
    being_in_multiline_token: Option<CommentToken>,
    blank_count: u64,
    comment_count: u64,
    code_count: u64,
}

trait Parser {
    fn parse(&mut self);
    fn parse_line(&mut self, line: &str);
}

impl CommonParser {
    fn new(file: File, comment_tokens: Vec<CommentToken>) -> Self {
        Self {
            reader:BufReader::new(file),
            comment_tokens,
            is_in_comment: false,
            being_in_multiline_token: None,
            blank_count: 0,
            comment_count: 0,
            code_count: 0,
        }
    }
}

impl Display for CommonParser {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{{ Code: {}, Comment: {}, Blank: {} }}", self.code_count, self.comment_count, self.blank_count)
    }
}

impl Parser for CommonParser {
    fn parse(&mut self) {
        for line in self.reader.lines() {
            if let Ok(s) = line {
                self.parse_line(s.as_str())
            }
        }
    }
    fn parse_line(&mut self, line: &str) {
        match self.being_in_multiline_token {
            Some(ref comment_token) => {
                for ref token in self.comment_tokens.iter() {
                    if let CommentToken::MultiLine(s, e) = token {
                        if line.ends_with(e.as_str()){
                            self.comment_count += 1;
                            self.is_in_comment = false;
                            return;
                        }
                    }
                }
                self.comment_count += 1;
            }
            None => {
                if line.trim().is_empty() {
                    self.blank_count += 1;
                    return;
                }
                for &token in self.comment_tokens.iter() {
                    match token {
                        CommentToken::Common(s) => {
                            if line.starts_with(s.as_str()) {
                                self.comment_count += 1;
                                return;
                            }
                        }
                        CommentToken::MultiLine(s, e) => {
                            if line.starts_with(s.as_str()) {
                                self.comment_count += 1;
                                self.being_in_multiline_token = Some(token);
                                return;
                            }
                        }
                    }
                }
                self.code_count += 1;
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
        let file = File::open(path).unwrap();
        let comment_tokens = vec![CommentToken::Common("//".to_string())];
        let mut parser = CommonParser::new(file, comment_tokens);
        parser.parse();
        println!("{}",parser);
    }
}