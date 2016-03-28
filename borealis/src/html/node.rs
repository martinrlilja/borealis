
use std::io::{self, Write};

use html5ever::driver::{parse_document, ParseOpts};
use html5ever::tree_builder::TreeSink;
use html5ever::tendril::TendrilSink;
use html5ever::serialize::{Serializable, Serializer, TraversalScope};

use super::{CommentNode, Document, ElementNode, ElementType, TextNode};
use dom;

#[derive(Clone, Debug, PartialEq)]
pub enum Node {
    Comment(CommentNode),
    Element(ElementNode),
    Text(TextNode),
}

impl Node {
    pub fn parse_str(string: &str) -> Node {
        let parser = parse_document(dom::Dom::new(), ParseOpts::default()).from_utf8();
        let dom = parser.one(string.as_bytes());

        (&dom.fragment()).into()
    }
}

impl<'a> From<&'a dom::Handle> for Node {
    fn from(handle: &'a dom::Handle) -> Node {
        match *handle.borrow() {
            (dom::Node::Comment(ref text), _) => CommentNode::new(text.clone()).into(),
            (dom::Node::Element(ref name, ref attributes, ref children),
             _) => {
                let mut children = children.iter().map(|c| c.into());
                let element_type = match *name {
                    qualname!(html, "template") => {
                        let document = Document::new(None, children.next());
                        ElementType::Template(document)
                    }
                    _ => ElementType::Normal(children.collect()),
                };

                ElementNode::new(name.clone(), attributes.clone(), element_type).into()
            }
            (dom::Node::Text(ref text), _) => TextNode::new(text.clone()).into(),
            _ => panic!("expected comment, element or text, got: {:?}", handle),
        }
    }
}

impl From<TextNode> for Node {
    fn from(node: TextNode) -> Node {
        Node::Text(node)
    }
}

impl From<CommentNode> for Node {
    fn from(node: CommentNode) -> Node {
        Node::Comment(node)
    }
}

impl From<ElementNode> for Node {
    fn from(node: ElementNode) -> Node {
        Node::Element(node)
    }
}

impl Serializable for Node {
    fn serialize<'wr, Wr: Write>(&self,
                                 serializer: &mut Serializer<'wr, Wr>,
                                 traversal_scope: TraversalScope)
                                 -> io::Result<()> {
        match (traversal_scope, self) {
            (_, &Node::Element(ref node)) => node.clone().serialize(serializer, traversal_scope),
            (TraversalScope::ChildrenOnly, _) => Ok(()),
            (TraversalScope::IncludeNode, &Node::Text(ref node)) => {
                serializer.write_text(&node.text())
            }
            (TraversalScope::IncludeNode, &Node::Comment(ref node)) => {
                serializer.write_comment(&node.comment())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use html::{Attribute, ElementNode, ElementType, TextNode};

    #[rustfmt_skip]
    const FRAGMENT: &'static str =
        "<div id=\"test\">Hello!</div>";

    #[test]
    fn test_parse_str() {
        let node = Node::parse_str(FRAGMENT);
        assert_eq!(node,
                   ElementNode::new(qualname!(html, "div"),
                                    vec![Attribute::new_str("id", "test")],
                                    ElementType::Normal(vec![TextNode::new_str("Hello!").into()]))
                       .into());
    }
}
