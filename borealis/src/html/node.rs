
use std::io::{self, Write};

use html5ever;
use html5ever::driver::{parse_document, ParseOpts};
use html5ever::tree_builder::TreeSink;
use html5ever::tendril::TendrilSink;
use html5ever::tendril::StrTendril;
use html5ever::serialize::{Serializable, Serializer, TraversalScope};

use string_cache::QualName;

use super::Document;
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

#[derive(Clone, Debug, PartialEq)]
pub struct Attribute {
    name: QualName,
    value: StrTendril,
}

impl Attribute {
    pub fn new(name: QualName, value: StrTendril) -> Attribute {
        Attribute {
            name: name,
            value: value,
        }
    }

    pub fn new_string(name: String, value: String) -> Attribute {
        Attribute::new(QualName::new(ns!(), name.into()), value.into())
    }

    pub fn new_str(name: &str, value: &str) -> Attribute {
        Attribute::new_string(name.to_owned(), value.to_owned())
    }

    pub fn name(&self) -> &QualName {
        &self.name
    }

    pub fn value(&self) -> &StrTendril {
        &self.value
    }
}

impl From<html5ever::Attribute> for Attribute {
    fn from(attr: html5ever::Attribute) -> Attribute {
        Attribute {
            name: attr.name,
            value: attr.value,
        }
    }
}

impl Into<html5ever::Attribute> for Attribute {
    fn into(self) -> html5ever::Attribute {
        html5ever::Attribute {
            name: self.name,
            value: self.value,
        }
    }
}

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

#[derive(Clone, Debug, PartialEq)]
pub struct CommentNode(StrTendril);

impl CommentNode {
    pub fn new(comment: StrTendril) -> CommentNode {
        CommentNode(comment)
    }

    pub fn text(&self) -> &StrTendril {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ElementType {
    Normal(Vec<Node>),
    Template(Document),
}

impl ElementType {
    pub fn new_normal() -> ElementType {
        ElementType::Normal(Vec::new())
    }

    pub fn new_template() -> ElementType {
        ElementType::Template(Document::new(None, None))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ElementNode {
    name: QualName,
    element_type: ElementType,
    attributes: Vec<Attribute>,
}

impl ElementNode {
    pub fn new(name: QualName,
               attributes: Vec<Attribute>,
               element_type: ElementType)
               -> ElementNode {
        ElementNode {
            name: name,
            element_type: element_type,
            attributes: attributes,
        }
    }

    pub fn name(&self) -> &QualName {
        &self.name
    }

    pub fn element_type(&self) -> &ElementType {
        &self.element_type
    }

    pub fn attributes(&self) -> &[Attribute] {
        &self.attributes
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
            (TraversalScope::IncludeNode, &Node::Text(ref node)) => serializer.write_text(&node.0),
            (TraversalScope::IncludeNode, &Node::Comment(ref node)) => {
                serializer.write_comment(&node.0)
            }
        }
    }
}

impl Serializable for ElementNode {
    fn serialize<'wr, Wr: Write>(&self,
                                 serializer: &mut Serializer<'wr, Wr>,
                                 traversal_scope: TraversalScope)
                                 -> io::Result<()> {
        if traversal_scope == TraversalScope::IncludeNode {
            try!(serializer.start_elem(self.name.clone(),
                                       self.attributes.iter().map(|a| (&a.name, &a.value[..]))));
        }

        match self.element_type {
            ElementType::Normal(ref children) => {
                for child in children.iter() {
                    try!(child.clone().serialize(serializer, TraversalScope::IncludeNode));
                }
            }
            ElementType::Template(ref document) => {
                try!(document.clone().serialize(serializer, traversal_scope));
            }
        }

        if traversal_scope == TraversalScope::IncludeNode {
            try!(serializer.end_elem(self.name.clone()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
