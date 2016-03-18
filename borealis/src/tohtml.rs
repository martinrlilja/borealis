
use super::html::{Html, Node};

pub trait ToHtmlDocument {
    fn to_html_document(&self) -> Dom;
}

pub trait ToHtmlFragment {
    fn to_html_fragment(&self) -> Node;
}

impl ToHtml for String {
    fn to_html_fragment(&self) -> Node {
        Html::Node(Node::new_text(&self))
    }
}
