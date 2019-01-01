use std::{
    fmt::{Display, Error, Formatter},
    fs::File,
    io,
    io::BufReader,
    io::prelude::*,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommentToken {
    Line(String),
    Block(String, String),
}

impl CommentToken {
    pub fn as_line_token(&self) -> Option<&str> {
        match *self {
            CommentToken::Line(ref s) => Some(&**s),
            _ => None,
        }
    }
    pub fn as_block_tokens(&self) -> Option<(&str, &str)> {
        match *self {
            CommentToken::Block(ref s, ref t) => Some((&**s, &**t)),
            _ => None,
        }
    }
    pub fn is_line(&self) -> bool {
        self.as_line_token().is_some()
    }
    pub fn is_block(&self) -> bool {
        self.as_block_tokens().is_some()
    }
}

#[derive(Clone, Debug)]
pub struct CodeStat {
    code: u64,
    blank: u64,
    comment: u64,
    path: String,
    ext: String,
}

impl CodeStat {
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
    fn parse(self) -> io::Result<CodeStat>;
    fn parse_line(&mut self, line: &str);
}

impl Display for CodeStat {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(
            f,
            "{{ Code: {}, Comment: {}, Blank: {} }}",
            self.code, self.comment, self.blank
        )
    }
}

enum ParserInternalState {
    Normal,
    InBlockComment,
}

pub enum ParserState {
    Ready,
    Parsing,
    Complete(CodeStat),
    Error(io::Error),
}

impl ParserState {}

struct ParserContext {
    stat: CodeStat,
    comment_tokens: Vec<CommentToken>,
    state: ParserInternalState,
    block_token: Option<CommentToken>,
}

impl ParserContext {
    fn new(path: &str, comment_tokens: Vec<CommentToken>) -> Self {
        Self {
            stat: CodeStat::new(path),
            comment_tokens,
            state: ParserInternalState::Normal,
            block_token: None,
        }
    }
}

pub struct CommonParser {
    context: ParserContext,
}

impl CommonParser {
    pub fn new(path: &str, comment_tokens: Vec<CommentToken>) -> Self {
        Self {
            context: ParserContext::new(path, comment_tokens),
        }
    }

    fn parse_line_normal(&mut self, line: &str) {
        let stat = &mut self.context.stat;
        if line.is_empty() {
            stat.blank += 1;
            return;
        }
        for token in self.context.comment_tokens.iter() {
            match token {
                CommentToken::Line(t) => {
                    if line.starts_with(t.as_str()) {
                        stat.comment += 1;
                        return;
                    }
                }
                CommentToken::Block(s, _e) => {
                    if line.contains(s.as_str()) {
                        stat.comment += 1;
                        self.context.state = ParserInternalState::InBlockComment;
                        self.context.block_token = Some(token.clone());
                        return;
                    }
                }
            }
        }
        stat.code += 1;
    }
    fn parse_line_in_block_comment(&mut self, line: &str) {
        let stat = &mut self.context.stat;
        for token in self.context.comment_tokens.iter() {
            if let CommentToken::Block(_s, e) = token {
                if line.contains(e.as_str()) {
                    if token == self.context.block_token.as_ref().unwrap() {
                        self.context.state = ParserInternalState::Normal;
                        break;
                    }
                }
            }
        }
        stat.comment += 1;
    }
}

impl Parser for CommonParser {
    fn parse(mut self) -> io::Result<CodeStat> {
        let file = File::open(self.context.stat.path.as_str())?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            self.parse_line(line?.trim());
        }
        Ok(self.context.stat)
    }
    fn parse_line(&mut self, line: &str) {
        match self.context.state {
            ParserInternalState::Normal => self.parse_line_normal(line),
            ParserInternalState::InBlockComment => self.parse_line_in_block_comment(line),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        let path = "tests/eng.cpp";
        let comment_tokens = vec![
            CommentToken::Line("//".to_string()),
            CommentToken::Block("/*".to_string(), "*/".to_string()),
        ];
        let parser = CommonParser::new(path, comment_tokens);
        let stats = parser.parse().unwrap();
        println!("{:#?}", stats);
    }
}
