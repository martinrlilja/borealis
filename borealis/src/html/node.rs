
use std::io::{self, Write};

use html5ever;
use html5ever::tendril::StrTendril;
use html5ever::serialize::{Serializable, Serializer, TraversalScope};

use string_cache::QualName;

use super::Document;

#[derive(Clone, Debug, PartialEq)]
pub enum ElementType {
    Normal(Vec<Node>),
    Template(Document),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Node {
    Text(TextNode),
    Comment(CommentNode),
    Element(ElementNode),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Attribute {
    name:  QualName,
    value: StrTendril,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextNode(StrTendril);

#[derive(Clone, Debug, PartialEq)]
pub struct CommentNode(StrTendril);

#[derive(Clone, Debug, PartialEq)]
pub struct ElementNode {
    name:         QualName,
    element_type: ElementType,
    attributes:   Vec<Attribute>,
}

impl Serializable for Node {
    fn serialize<'wr, Wr: Write>(&self, serializer: &mut Serializer<'wr, Wr>,
                                 traversal_scope: TraversalScope) -> io::Result<()>
    {
        match (traversal_scope, self) {
            (_, &Node::Element(ref node)) => {
                node.clone().serialize(serializer, traversal_scope)
            },
            (TraversalScope::ChildrenOnly, _) => Ok(()),
            (TraversalScope::IncludeNode, &Node::Text(ref node)) => {
                serializer.write_text(&node.0)
            },
            (TraversalScope::IncludeNode, &Node::Comment(ref node)) => {
                serializer.write_comment(&node.0)
            },
        }
    }
}

impl Serializable for ElementNode {
    fn serialize<'wr, Wr: Write>(&self, serializer: &mut Serializer<'wr, Wr>,
                                 traversal_scope: TraversalScope) -> io::Result<()>
    {
        if traversal_scope == TraversalScope::IncludeNode {
            try!(serializer.start_elem(self.name.clone(),
                self.attributes.iter().map(|a| (&a.name, &a.value[..]))));
        }

        match self.element_type {
            ElementType::Normal(ref children) => {
                for child in children.iter() {
                    try!(child.clone().serialize(serializer, TraversalScope::IncludeNode));
                }
            },
            ElementType::Template(ref document) => {
                try!(document.clone().serialize(serializer, traversal_scope));
            },
        }

        if traversal_scope == TraversalScope::IncludeNode {
            try!(serializer.end_elem(self.name.clone()));
        }

        Ok(())
    }
}
