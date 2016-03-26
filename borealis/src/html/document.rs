
use std::io::{self, Write};

use html5ever::serialize::{Serializable, Serializer, TraversalScope};

use super::{Doctype, Node};

#[derive(Clone, Debug, PartialEq)]
pub struct Document {
    doctype: Option<Doctype>,
    child: Option<Box<Node>>,
}

impl Document {
    pub fn new(doctype: Option<Doctype>, child: Option<Node>) -> Document {
        Document {
            doctype: doctype,
            child: child.map(Box::new),
        }
    }

    pub fn doctype(&self) -> Option<&Doctype> {
        self.doctype.as_ref()
    }

    pub fn child(&self) -> Option<&Node> {
        match self.child {
            Some(ref child) => Some(child.as_ref()),
            None => None,
        }
    }

    pub fn set_doctype(&mut self, doctype: Doctype) {
        self.doctype = Some(doctype);
    }

    pub fn set_child(&mut self, node: Node) {
        self.child = Some(Box::new(node));
    }

    pub fn unset_child(&mut self) {
        self.child = None;
    }
}

impl Serializable for Document {
    fn serialize<'wr, Wr: Write>(&self,
                                 serializer: &mut Serializer<'wr, Wr>,
                                 traversal_scope: TraversalScope)
                                 -> io::Result<()> {
        match traversal_scope {
            TraversalScope::IncludeNode => panic!("Cannot serialize the Document node itself."),
            TraversalScope::ChildrenOnly => {
                if let Some(ref doctype) = self.doctype {
                    try!(doctype.clone().serialize(serializer, TraversalScope::IncludeNode));
                }

                if let Some(ref child) = self.child {
                    try!(child.clone().serialize(serializer, TraversalScope::IncludeNode));
                }

                Ok(())
            }
        }
    }
}
