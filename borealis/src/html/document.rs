
use std::io::{self, Write};

use html5ever::driver::{parse_document, ParseOpts};
use html5ever::tree_builder::TreeSink;
use html5ever::tendril::TendrilSink;
use html5ever::serialize::{Serializable, Serializer, TraversalScope};

use super::{Doctype, Node};
use dom;

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

    pub fn parse_str(string: &str) -> Document {
        let parser = parse_document(dom::Dom::new(), ParseOpts::default()).from_utf8();
        let dom = parser.one(string.as_bytes());

        dom.document().into()
    }
}

impl<'a> From<&'a dom::Handle> for Document {
    fn from(handle: &'a dom::Handle) -> Document {
        match *handle.borrow() {
            (dom::Node::Document(ref doctype, ref child), _) => {
                Document::new(doctype.clone(), child.as_ref().map(Node::from))
            }
            _ => panic!("expected document, got: {:?}", handle),
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[rustfmt_skip]
    const DOCUMENT: &'static str =
        "<!DOCTYPE html>
         <html lang=\"en\">
            <head>
                <title>Test</title>
            </head>
            <body>
                Document
                <img src=\"test.flif\" alt=\"test\">
            </body>
         </html>";

    #[bench]
    fn bench_parse_document(b: &mut Bencher) {
        b.iter(|| {
            Document::parse_str(DOCUMENT);
        });
    }
}
