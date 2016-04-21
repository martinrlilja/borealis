
use std::io::Write;

use html5ever::{ParseOpts, parse_document};
use html5ever::tree_builder::TreeSink;
use html5ever::tendril::TendrilSink;

use serializer::{SerializeDocument, DocumentSerializer};
use super::{Dom, Handle};

#[derive(Clone, Debug, PartialEq)]
pub struct Document {
    node: Handle,
}

impl Document {
    pub fn parse_str(s: &str) -> Document {
        let parser = parse_document(Dom::new(), ParseOpts::default()).from_utf8();
        let dom = parser.one(s.as_bytes());
        Document { node: dom.document() }
    }

    pub fn handle(self) -> Handle {
        self.node
    }
}

impl SerializeDocument for Document {
    fn serialize_document<W: Write>(self, s: DocumentSerializer<W>) {
        self.node.serialize_document(s);
    }
}
