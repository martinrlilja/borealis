
use html5ever::tendril::StrTendril;

#[derive(Clone, Debug, PartialEq)]
pub struct CommentNode(StrTendril);

impl CommentNode {
    pub fn new(comment: StrTendril) -> CommentNode {
        CommentNode(comment)
    }

    pub fn comment(&self) -> &StrTendril {
        &self.0
    }
}
