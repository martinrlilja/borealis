
use std::io::Write;

use html5ever::{ParseOpts, parse_fragment};
use html5ever::tree_builder::TreeSink;
use html5ever::tendril::TendrilSink;

use serializer::{SerializeNode, NodeSerializer};
use super::{Dom, Handle};

#[derive(Clone, Debug, PartialEq)]
pub struct Fragment {
    nodes: Vec<Handle>,
}

impl Fragment {
    pub fn parse_str(s: &str) -> Fragment {
        let parser = parse_fragment(Dom::new(),
                                    ParseOpts::default(),
                                    qualname!(html, "body"),
                                    Vec::new())
                         .from_utf8();
        let dom = parser.one(s.as_bytes());
        Fragment { nodes: dom.fragment() }
    }

    pub fn handles(self) -> Vec<Handle> {
        self.nodes
    }
}

impl SerializeNode for Fragment {
    fn serialize_node<W: Write>(self, s: &mut NodeSerializer<W>) {
        for child in self.nodes.iter() {
            child.serialize_node(s);
        }
    }
}
