
use std::io::{self, Write};

use html5ever::serialize::{Serializable, Serializer, TraversalScope};

use super::{Doctype, Node, TreeHandle};

#[derive(Clone, Debug, PartialEq)]
pub struct Document {
    doctype: Option<Doctype>,
    child:   Option<TreeHandle<Node>>,
}

impl Document {
    pub fn new(doctype: Option<Doctype>, child: Option<TreeHandle<Node>>) -> Document {
        Document {
            doctype: doctype,
            child:   child,
        }
    }

    pub fn doctype(&self) -> Option<&Doctype> {
        self.doctype.as_ref()
    }

    pub fn child(&self) -> Option<&TreeHandle<Node>> {
        self.child.as_ref()
    }

    pub fn set_doctype(&mut self, doctype: Doctype) {
        self.doctype = Some(doctype);
    }

    pub fn set_child(&mut self, node: TreeHandle<Node>) {
        self.child = Some(node);
    }

    pub fn unset_child(&mut self) {
        self.child = None;
    }
}

impl Serializable for Document {
    fn serialize<'wr, Wr: Write>(&self, serializer: &mut Serializer<'wr, Wr>,
                                 traversal_scope: TraversalScope) -> io::Result<()>
    {
        match traversal_scope {
            TraversalScope::IncludeNode  => panic!("Cannot serialize the Document node itself."),
            TraversalScope::ChildrenOnly => {
                if let Some(ref doctype) = self.doctype {
                    try!(doctype.clone().serialize(serializer, TraversalScope::IncludeNode));
                }

                if let Some(ref child) = self.child {
                    try!(child.clone().serialize(serializer, TraversalScope::IncludeNode));
                }

                Ok(())
            },
        }
    }
}
