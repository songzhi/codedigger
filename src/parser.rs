/// 解析文件
use std::fmt::{Display, Error, Formatter};
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::prelude::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommentToken {
    Common(String),
    MultiLine(String, String),
}


#[derive(Clone, Debug)]
pub struct CodeStats {
    code: u64,
    blank: u64,
    comment: u64,
    path: String,
    ext: String
}

impl CodeStats {
    pub fn new(path: &str) -> Self {
        Self {
            code: 0u64,
            blank: 0u64,
            comment: 0u64,
            path: path.to_string(),
            ext: String::from(path.rsplit(".").nth(0).unwrap()),
        }
    }
}

pub trait Parser {
    fn parse(mut self) -> io::Result<CodeStats>;
    fn parse_line(&mut self, line: &str);
}


impl Display for CodeStats {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{{ Code: {}, Comment: {}, Blank: {} }}", self.code, self.comment, self.blank)
    }
}


enum ParserState {
    Normal,
    InMultilineComment,
}

struct ParserContext {
    stats: CodeStats,
    comment_tokens: Vec<CommentToken>,
    state: ParserState,
    multiline_token: Option<CommentToken>,
}

impl ParserContext {
    fn new(path: &str, comment_tokens: Vec<CommentToken>) -> Self {
        Self {
            stats: CodeStats::new(path),
            comment_tokens,
            state: ParserState::Normal,
            multiline_token: None,
        }
    }
}

pub struct CommonParser {
    context: ParserContext
}

impl CommonParser {
    pub fn new(path: &str, comment_tokens: Vec<CommentToken>) -> Self {
        Self {
            context: ParserContext::new(path, comment_tokens)
        }
    }

    pub fn parse_line_normal(&mut self, line: &str) {
        let stats = &mut self.context.stats;
        if line.is_empty() {
            stats.blank += 1;
            return;
        }
        for token in self.context.comment_tokens.iter() {
            match token {
                CommentToken::Common(t) => {
                    if line.starts_with(t.as_str()) {
                        stats.comment += 1;
                        return;
                    }
                }
                CommentToken::MultiLine(s, _e) => {
                    if line.starts_with(s.as_str()) {
                        stats.comment += 1;
                        self.context.state = ParserState::InMultilineComment;
                        self.context.multiline_token = Some(token.clone());
                        return;
                    }
                }
            }
        }
        stats.code += 1;
    }
    pub fn parse_line_in_multiline_comment(&mut self, line: &str) {
        let stats = &mut self.context.stats;
        for token in self.context.comment_tokens.iter() {
            if let CommentToken::MultiLine(_s, e) = token {
                if line.ends_with(e.as_str()) {
                    if token == self.context.multiline_token.as_ref().unwrap() {
                        self.context.state = ParserState::Normal;
                        break;
                    }
                }
            }
        }
        stats.comment += 1;
    }
}

impl Parser for CommonParser {
    fn parse(mut self) -> io::Result<CodeStats> {
        let file = File::open(self.context.stats.path.as_str())?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            if let Ok(s) = line {
                self.parse_line(s.as_str().trim());
            }
        }
        Ok(self.context.stats)
    }
    fn parse_line(&mut self, line: &str) {
        match self.context.state {
            ParserState::Normal => self.parse_line_normal(line),
            ParserState::InMultilineComment => self.parse_line_in_multiline_comment(line)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        let path = "G:\\C_C++code\\eng.cpp";
        let comment_tokens = vec![CommentToken::Common("//".to_string()), CommentToken::MultiLine("/*".to_string(), "*/".to_string())];
        let parser = CommonParser::new(path, comment_tokens);
        let stats = parser.parse().unwrap();
        println!("{:#?}", stats);
    }
}