
use std::io::{self, Write};

use html5ever::driver::{parse_document, ParseOpts};
use html5ever::tree_builder::TreeSink;
use html5ever::tendril::TendrilSink;
use html5ever::serialize::{Serializable, Serializer, serialize, SerializeOpts, TraversalScope};

use super::{Doctype, Node};
use dom;

/// Represents a document containing a doctype and a root node.
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

    pub fn serialize<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        serialize(writer, self, SerializeOpts::default())
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
                    try!(doctype.serialize(serializer, TraversalScope::IncludeNode));
                }

                if let Some(ref child) = self.child {
                    try!(child.serialize(serializer, TraversalScope::IncludeNode));
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

    use html::{Attribute, Doctype, ElementNode, TextNode};

    #[rustfmt_skip]
    const DOCUMENT: &'static str =
        "<!DOCTYPE html>\
         <html lang=\"en\">\
            <head>\
                <title>Test</title>\
            </head>\
            <body>\
                <h1>Document</h1>\
                <img src=\"test.flif\" alt=\"test\">\
            </body>\
         </html>";

    #[bench]
    fn bench_parse_document(b: &mut Bencher) {
        b.iter(|| {
            Document::parse_str(DOCUMENT);
        });
    }

    #[test]
    fn test_parse_document() {
        let document = Document::parse_str(DOCUMENT);

        let title = ElementNode::new_normal("title", vec![], vec![TextNode::new("Test").into()]);
        let head = ElementNode::new_normal("head", vec![], vec![title.into()]);
        let h1 = ElementNode::new_normal("h1", vec![], vec![TextNode::new("Document").into()]);
        let img = ElementNode::new_normal("img",
                                          vec![Attribute::new("src", "test.flif"),
                                               Attribute::new("alt", "test")],
                                          vec![]);
        let body = ElementNode::new_normal("body", vec![], vec![h1.into(), img.into()]);
        let html = ElementNode::new_normal("html",
                                           vec![Attribute::new("lang", "en")],
                                           vec![head.into(), body.into()]);
        assert_eq!(document,
                   Document::new(Some(Doctype::new_html5()), Some(html.into())));
    }
}
