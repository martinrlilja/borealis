
use std::io::{self, Write};

use html5ever;
use html5ever::tendril::StrTendril;
use html5ever::serialize::{Serializable, Serializer, TraversalScope};

use string_cache::QualName;

use super::{Document, TreeHandle};

#[derive(Clone, Debug, PartialEq)]
pub enum Node {
    Text(TextNode),
    Comment(CommentNode),
    Element(ElementNode),
}

impl Node {
    pub fn expect_text(&self) -> &TextNode {
        if let &Node::Text(ref node) = self {
            node
        } else {
            panic!("Expected text node, got: {:?}", self);
        }
    }

    pub fn expect_comment(&self) -> &CommentNode {
        if let &Node::Comment(ref node) = self {
            node
        } else {
            panic!("Expected comment node, got: {:?}", self);
        }
    }

    pub fn expect_element(&self) -> &ElementNode {
        if let &Node::Element(ref node) = self {
            node
        } else {
            panic!("Expected element node, got: {:?}", self);
        }
    }

    pub fn expect_element_mut(&mut self) -> &mut ElementNode {
        if let &mut Node::Element(ref mut node) = self {
            node
        } else {
            panic!("Expected element node, got: {:?}", self);
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
        Attribute::new(QualName::new(ns!(html), name.into()), value.into())
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
    Normal(Vec<TreeHandle<Node>>),
    Template(TreeHandle<Document>),
}

impl ElementType {
    pub fn new_normal() -> ElementType {
        ElementType::Normal(Vec::new())
    }

    pub fn new_template() -> ElementType {
        ElementType::Template(TreeHandle::new(Document::new(None, None)))
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
               element_type: ElementType,
               attributes: Vec<Attribute>)
               -> ElementNode {
        ElementNode {
            name: name,
            element_type: element_type,
            attributes: attributes,
        }
    }

    pub fn expect_normal(&self) -> &[TreeHandle<Node>] {
        if let ElementType::Normal(ref children) = self.element_type {
            children
        } else {
            panic!("Expected normal element, got: {:?}", self);
        }
    }

    pub fn expect_normal_mut(&mut self) -> &mut Vec<TreeHandle<Node>> {
        if let ElementType::Normal(ref mut children) = self.element_type {
            children
        } else {
            panic!("Expected normal element, got: {:?}", self);
        }
    }

    pub fn expect_template(&self) -> &TreeHandle<Document> {
        if let ElementType::Template(ref document) = self.element_type {
            document
        } else {
            panic!("Expected template element, got: {:?}", self);
        }
    }

    pub fn name(&self) -> &QualName {
        &self.name
    }

    pub fn element_type(&self) -> &ElementType {
        &self.element_type
    }

    pub fn element_type_mut(&mut self) -> &mut ElementType {
        &mut self.element_type
    }

    pub fn attributes(&self) -> &[Attribute] {
        &self.attributes
    }

    pub fn attributes_mut(&mut self) -> &mut Vec<Attribute> {
        &mut self.attributes
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
