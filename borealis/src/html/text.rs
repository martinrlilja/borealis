
use html5ever::tendril::StrTendril;

#[derive(Clone, Debug, PartialEq)]
pub struct TextText(StrTendril);

impl From<StrTendril> for TextText {
    fn from(text: StrTendril) -> TextText {
        TextText(text)
    }
}

impl From<String> for TextText {
    fn from(text: String) -> TextText {
        TextText(text.into())
    }
}

impl<'a> From<&'a str> for TextText {
    fn from(text: &'a str) -> TextText {
        TextText(text.clone().into())
    }
}

/// Represents text in an HTML document.
#[derive(Clone, Debug, PartialEq)]
pub struct TextNode(TextText);

impl TextNode {
    pub fn new<T: Into<TextText>>(text: T) -> TextNode {
        TextNode(text.into())
    }

    #[inline]
    pub fn text(&self) -> &StrTendril {
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

        let text = TextNode::new(tendril.clone());
        assert_eq!(*text.text(), tendril);
    }
}
