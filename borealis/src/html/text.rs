
use html5ever::tendril::StrTendril;

/// Represents text in an HTML document.
#[derive(Clone, Debug, PartialEq)]
pub struct TextNode(StrTendril);

impl TextNode {
    pub fn new(text: StrTendril) -> TextNode {
        TextNode(text)
    }

    pub fn new_str(text: &str) -> TextNode {
        TextNode::new_string(text.to_owned())
    }

    pub fn new_string(text: String) -> TextNode {
        TextNode::new(text.into())
    }

    pub fn text(&self) -> &StrTendril {
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

        let text = TextNode::new(tendril.clone());
        assert_eq!(*text.text(), tendril);

        let text = TextNode::new_str(&string);
        assert_eq!(*text.text(), tendril);

        let text = TextNode::new_string(string.clone());
        assert_eq!(*text.text(), tendril);
    }
}
