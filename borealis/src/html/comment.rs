
use html5ever::tendril::StrTendril;

#[derive(Clone, Debug, PartialEq)]
pub struct CommentText(StrTendril);

impl From<StrTendril> for CommentText {
    fn from(text: StrTendril) -> CommentText {
        CommentText(text)
    }
}

impl From<String> for CommentText {
    fn from(text: String) -> CommentText {
        CommentText(text.into())
    }
}

impl<'a> From<&'a str> for CommentText {
    fn from(text: &'a str) -> CommentText {
        CommentText(text.clone().into())
    }
}

/// Represents a comment in an HTML document.
#[derive(Clone, Debug, PartialEq)]
pub struct CommentNode(CommentText);

impl CommentNode {
    pub fn new<T: Into<CommentText>>(comment: T) -> CommentNode {
        CommentNode(comment.into())
    }

    #[inline]
    pub fn comment(&self) -> &StrTendril {
        &(self.0).0
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
    }
}
