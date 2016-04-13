
use std::io::{self, Write};

use html5ever::driver::{parse_fragment, ParseOpts};
use html5ever::tree_builder::TreeSink;
use html5ever::tendril::TendrilSink;
use html5ever::serialize::{Serializable, Serializer, TraversalScope};

use super::{CommentNode, Document, ElementNode, ElementType, TextNode};
use dom;

/// Represents a node, which can be a comment, element or text.
#[derive(Clone, Debug, PartialEq)]
pub enum Node {
    /// Wrapper around a comment node.
    Comment(CommentNode),
    /// Wrapper around an element node.
    Element(ElementNode),
    /// Wrapper around a text node.
    Text(TextNode),
}

impl Node {
    /// Takes a string and parses it like a fragment of a document.
    /// Note that this only expects a single root node, which cannot be
    /// an html, head or body tag.
    ///
    /// # Example
    ///
    ///     use borealis::html::{Attribute, Node, ElementNode, ElementType};
    ///
    ///     let fragment = r#"<img src="test.jpg">"#;
    ///     let nodes = Node::parse_str(fragment);
    ///
    ///     assert_eq!(nodes,
    ///                vec![ElementNode::new("img",
    ///                                 vec![Attribute::new("src", "test.jpg")],
    ///                                 ElementType::new_normal())
    ///                    .into()]);
    pub fn parse_str(string: &str) -> Vec<Node> {
        let parser = parse_fragment(dom::Dom::new(),
                                    ParseOpts::default(),
                                    qualname!(html, "body"),
                                    Vec::new())
                         .from_utf8();
        let dom = parser.one(string.as_bytes());

        dom.fragment().iter().map(|e| e.into()).collect()
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
            (_, &Node::Element(ref node)) => node.serialize(serializer, traversal_scope),
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

    use html::{Attribute, ElementNode, TextNode};

    #[cfg_attr(feature = "nightly", rustfmt_skip)]
    const FRAGMENT_1: &'static str =
        r#"<div id="test">Hello!</div>"#;

    #[cfg_attr(feature = "nightly", rustfmt_skip)]
    const FRAGMENT_2: &'static str =
        "<br>test!";

    #[test]
    fn test_parse_str() {
        let nodes = Node::parse_str(FRAGMENT_1);
        assert_eq!(nodes,
                   vec![ElementNode::new_normal(qualname!(html, "div"),
                                                vec![Attribute::new("id", "test")],
                                                vec![TextNode::new("Hello!").into()])
                            .into()]);

        let nodes = Node::parse_str(FRAGMENT_2);
        assert_eq!(nodes,
                   vec![ElementNode::new_normal(qualname!(html, "br"), vec![], vec![]).into(),
                        TextNode::new("test!").into()]);
    }
}
