
use std::io::{self, Write};

use html5ever::tree_builder::TreeSink;
use html5ever::tendril::TendrilSink;
use html5ever::serialize::{Serializable, Serializer, TraversalScope};

use string_cache::QualName;

use super::{Attribute, Document, Node};

/// The type of an element.
#[derive(Clone, Debug, PartialEq)]
pub enum ElementType {
    /// An element with zero or more child nodes.
    Normal(Vec<Node>),
    /// An element containing another document.
    Template(Document),
}

impl ElementType {
    /// Creates a normal element with no children.
    pub fn new_normal() -> ElementType {
        ElementType::Normal(Vec::new())
    }

    /// Creates a template element with an empty document.
    pub fn new_template() -> ElementType {
        ElementType::Template(Document::new(None, None))
    }
}

/// Converts a document into a template element.
impl From<Document> for ElementType {
    fn from(document: Document) -> ElementType {
        ElementType::Template(document)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ElementName(QualName);

impl From<QualName> for ElementName {
    fn from(name: QualName) -> ElementName {
        ElementName(name)
    }
}

impl From<String> for ElementName {
    fn from(name: String) -> ElementName {
        ElementName(QualName::new(ns!(html), name.into()))
    }
}

impl<'a> From<&'a str> for ElementName {
    fn from(name: &'a str) -> ElementName {
        ElementName(QualName::new(ns!(html), name.clone().into()))
    }
}

/// Represents an element with a name, attributes and children
/// depending on it's type.
#[derive(Clone, Debug, PartialEq)]
pub struct ElementNode {
    name: ElementName,
    attributes: Vec<Attribute>,
    element_type: ElementType,
}

impl ElementNode {
    pub fn new<N: Into<ElementName>>(name: N,
                                     attributes: Vec<Attribute>,
                                     element_type: ElementType)
                                     -> ElementNode {
        ElementNode {
            name: name.into(),
            attributes: attributes,
            element_type: element_type,
        }
    }

    pub fn new_normal<N: Into<ElementName>>(name: N,
                                            attributes: Vec<Attribute>,
                                            children: Vec<Node>)
                                            -> ElementNode {
        ElementNode::new(name.into(), attributes, ElementType::Normal(children))
    }

    pub fn new_template<N: Into<ElementName>>(name: N,
                                              attributes: Vec<Attribute>,
                                              document: Document)
                                              -> ElementNode {
        ElementNode::new(name.into(), attributes, ElementType::Template(document))
    }

    #[inline]
    pub fn name(&self) -> &QualName {
        &self.name.0
    }

    #[inline]
    pub fn element_type(&self) -> &ElementType {
        &self.element_type
    }

    #[inline]
    pub fn attributes(&self) -> &[Attribute] {
        &self.attributes
    }
}

impl Serializable for ElementNode {
    fn serialize<'wr, Wr: Write>(&self,
                                 serializer: &mut Serializer<'wr, Wr>,
                                 traversal_scope: TraversalScope)
                                 -> io::Result<()> {
        if traversal_scope == TraversalScope::IncludeNode {
            try!(serializer.start_elem(self.name().clone(),
                                       self.attributes.iter().map(|a| (a.name(), &a.value()[..]))));
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
            try!(serializer.end_elem(self.name().clone()));
        }

        Ok(())
    }
}
