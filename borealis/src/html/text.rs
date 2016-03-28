
use html5ever::tendril::StrTendril;

#[derive(Clone, Debug, PartialEq)]
pub struct TextNode(StrTendril);

impl TextNode {
    pub fn new(text: StrTendril) -> TextNode {
        TextNode(text)
    }

    pub fn new_string(text: String) -> TextNode {
        TextNode::new(text.into())
    }

    pub fn new_str(text: &str) -> TextNode {
        TextNode::new_string(text.to_owned())
    }

    pub fn text(&self) -> &StrTendril {
        &self.0
    }
}
