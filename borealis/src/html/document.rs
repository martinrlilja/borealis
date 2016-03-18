
use std::io::{self, Write};

use html5ever;
use html5ever::tendril::StrTendril;
use html5ever::serialize::{Serializable, Serializer, TraversalScope};

use string_cache::QualName;

use super::Doctype;
use super::Node;

#[derive(Clone, Debug, PartialEq)]
pub struct Document {
    doctype: Option<Doctype>,
    child:   Option<Box<Node>>,
}

impl Document {
    pub fn new(doctype: Option<Doctype>, child: Option<Node>) -> Document {
        Document {
            doctype: doctype,
            child:   child.map(|c| Box::new(c)),
        }
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
