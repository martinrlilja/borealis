
use super::html::{Document, Node, TextNode};

pub trait DocumentTemplate {
    fn document_template(self) -> Document;
}

pub trait FragmentTemplate {
    fn fragment_template(self) -> Node;
}

impl FragmentTemplate for String {
    fn fragment_template(self) -> Node {
        TextNode::new(self).into()
    }
}

impl<'a> FragmentTemplate for &'a str {
    fn fragment_template(self) -> Node {
        TextNode::new(self).into()
    }
}
