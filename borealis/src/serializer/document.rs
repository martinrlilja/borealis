
use std::io::Write;

use super::serializer::Serializer;

use super::NodeSerializer;
use super::node::new_node_ser;

pub struct DocumentSerializer<'a, 'b: 'a, 'w: 'b, W: 'w + Write> {
    serializer: &'a mut Serializer<'b, 'w, W>,
}

impl<'a, 'b, 'w, W: Write> DocumentSerializer<'a, 'b, 'w, W> {
    pub fn doctype(self, name: &str) -> DocumentDoctypeSerializer<'a, 'b, 'w, W> {
        self.serializer.write_doctype(name);
        DocumentDoctypeSerializer { internal: self }
    }

    pub fn node(self) -> NodeSerializer<'a, 'b, 'w, W> {
        new_node_ser(self.serializer)
    }
}

pub struct DocumentDoctypeSerializer<'a, 'b: 'a, 'w: 'b, W: 'w + Write> {
    internal: DocumentSerializer<'a, 'b, 'w, W>,
}

impl<'a, 'b, 'w, W: Write> DocumentDoctypeSerializer<'a, 'b, 'w, W> {
    pub fn node(self) -> NodeSerializer<'a, 'b, 'w, W> {
        self.internal.node()
    }
}

pub fn new_doc_ser<'a, 'b, 'w, W: Write>(ser: &'a mut Serializer<'b, 'w, W>)
                                         -> DocumentSerializer<'a, 'b, 'w, W> {
    DocumentSerializer { serializer: ser }
}
