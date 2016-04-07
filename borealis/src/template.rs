
use super::html::{Node, TextNode};

pub trait IntoNode : Sized {
    fn into_node(self) -> Node;

    fn into_nodes(self) -> Vec<Node> {
        vec![self.into_node()]
    }
}

pub trait IntoNodes {
    fn into_nodes(self) -> Vec<Node>;
}

impl<I: IntoNode, T: IntoIterator<Item=I>> IntoNodes for T {
    fn into_nodes(self) -> Vec<Node> {
        self.into_iter().map(|n| n.into_node()).collect()
    }
}

impl IntoNode for String {
    fn into_node(self) -> Node {
        TextNode::new(self).into()
    }
}

impl<'a> IntoNode for &'a str {
    fn into_node(self) -> Node {
        TextNode::new(self).into()
    }
}
