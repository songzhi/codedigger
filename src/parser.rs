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

impl CommentToken {
    pub fn as_common_token(&self) -> Option<&str> {
        match *self {
            CommentToken::Common(ref s) => Some(&**s),
            _ => None
        }
    }
    pub fn as_multiline_tokens(&self) -> Option<(&str, &str)> {
        match *self {
            CommentToken::MultiLine(ref s, ref t) => Some((&**s, &**t)),
            _ => None
        }
    }
    pub fn is_common(&self) -> bool {
        self.as_common_token().is_some()
    }
    pub fn is_multiline(&self) -> bool {
        self.as_multiline_tokens().is_some()
    }
}


#[derive(Clone, Debug)]
pub struct CodeStats {
    code: u64,
    blank: u64,
    comment: u64,
    path: String,
    ext: String,
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
    fn parse(self) -> io::Result<CodeStats>;
    fn parse_line(&mut self, line: &str);
}


impl Display for CodeStats {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{{ Code: {}, Comment: {}, Blank: {} }}", self.code, self.comment, self.blank)
    }
}


enum ParserInternalState {
    Normal,
    InMultilineComment,
}

struct ParserContext {
    stats: CodeStats,
    comment_tokens: Vec<CommentToken>,
    state: ParserInternalState,
    multiline_token: Option<CommentToken>,
}

impl ParserContext {
    fn new(path: &str, comment_tokens: Vec<CommentToken>) -> Self {
        Self {
            stats: CodeStats::new(path),
            comment_tokens,
            state: ParserInternalState::Normal,
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

    fn parse_line_normal(&mut self, line: &str) {
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
                    if line.contains(s.as_str()) {
                        stats.comment += 1;
                        self.context.state = ParserInternalState::InMultilineComment;
                        self.context.multiline_token = Some(token.clone());
                        return;
                    }
                }
            }
        }
        stats.code += 1;
    }
    fn parse_line_in_multiline_comment(&mut self, line: &str) {
        let stats = &mut self.context.stats;
        for token in self.context.comment_tokens.iter() {
            if let CommentToken::MultiLine(_s, e) = token {
                if line.contains(e.as_str()) {
                    if token == self.context.multiline_token.as_ref().unwrap() {
                        self.context.state = ParserInternalState::Normal;
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
            self.parse_line(line?.trim());
        }
        Ok(self.context.stats)
    }
    fn parse_line(&mut self, line: &str) {
        match self.context.state {
            ParserInternalState::Normal => self.parse_line_normal(line),
            ParserInternalState::InMultilineComment => self.parse_line_in_multiline_comment(line)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        let path = "tests/eng.cpp";
        let comment_tokens = vec![CommentToken::Common("//".to_string()), CommentToken::MultiLine("/*".to_string(), "*/".to_string())];
        let parser = CommonParser::new(path, comment_tokens);
        let stats = parser.parse().unwrap();
        println!("{:#?}", stats);
    }
}