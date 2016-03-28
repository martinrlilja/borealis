
use html5ever::tendril::StrTendril;

/// Represents a comment in an HTML document.
#[derive(Clone, Debug, PartialEq)]
pub struct CommentNode(StrTendril);

impl CommentNode {
    pub fn new(comment: StrTendril) -> CommentNode {
        CommentNode(comment)
    }

    pub fn new_str(text: &str) -> CommentNode {
        CommentNode::new_string(text.to_owned())
    }

    pub fn new_string(text: String) -> CommentNode {
        CommentNode::new(text.into())
    }

    pub fn comment(&self) -> &StrTendril {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use html5ever::tendril::StrTendril;

    #[test]
    fn test_new() {
        let string = "Test".to_owned();
        let tendril = StrTendril::from(string.clone());

        let comment = CommentNode::new(tendril.clone());
        assert_eq!(*comment.comment(), tendril);

        let comment = CommentNode::new_str(&string);
        assert_eq!(*comment.comment(), tendril);

        let comment = CommentNode::new_string(string.clone());
        assert_eq!(*comment.comment(), tendril);
    }
}
