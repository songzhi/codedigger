use parser::CommentToken;

pub struct Config {}

impl Config {
    pub fn get_comment_tokens(&self, path: &str) -> Option<Vec<CommentToken>> {
        unimplemented!()
    }
}
