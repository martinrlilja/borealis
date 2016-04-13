
use super::html::{Document, Node, TextNode};

pub trait IntoDocument {
    fn into_document(self) -> Document;
}

pub trait IntoNode : Sized {
    fn into_node(self) -> Option<Node>;
}

pub trait IntoNodes {
    fn into_nodes(self) -> Vec<Node>;
}

impl<T: IntoNode> IntoNodes for T {
    fn into_nodes(self) -> Vec<Node> {
        self.into_node().into_iter().collect()
    }
}

pub trait IntoNodesIter {
    fn into_nodes(self) -> Vec<Node>;
}

impl<I: IntoNodes, T: IntoIterator<Item = I>> IntoNodesIter for T {
    fn into_nodes(self) -> Vec<Node> {
        self.into_iter().flat_map(|n| n.into_nodes()).collect()
    }
}

impl IntoNode for String {
    fn into_node(self) -> Option<Node> {
        Some(TextNode::new(self).into())
    }
}

impl<'a> IntoNode for &'a str {
    fn into_node(self) -> Option<Node> {
        Some(TextNode::new(self).into())
    }
}
