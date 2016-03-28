
use std::io::{self, Write};

use html5ever::tree_builder::TreeSink;
use html5ever::tendril::TendrilSink;
use html5ever::serialize::{Serializable, Serializer, TraversalScope};

use string_cache::QualName;

use super::{Attribute, Document, Node};

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

impl Serializable for ElementNode {
    fn serialize<'wr, Wr: Write>(&self,
                                 serializer: &mut Serializer<'wr, Wr>,
                                 traversal_scope: TraversalScope)
                                 -> io::Result<()> {
        if traversal_scope == TraversalScope::IncludeNode {
            try!(serializer.start_elem(self.name.clone(),
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
            try!(serializer.end_elem(self.name.clone()));
        }

        Ok(())
    }
}
