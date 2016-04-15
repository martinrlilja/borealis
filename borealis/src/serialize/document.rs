
use std::io::Write;

use super::serializer::Serializer;

use string_cache::QualName;

use super::NodeSerializer;
use super::node::element_normal;

pub struct DocumentSerializer<'a, 'b: 'a, 'w: 'b, W: 'w + Write> {
    serializer: &'a mut Serializer<'b, 'w, W>,
}

impl<'a, 'b, 'w, W: Write> DocumentSerializer<'a, 'b, 'w, W> {
    pub fn doctype(self, name: &str) -> DocumentDoctypeSerializer<'a, 'b, 'w, W> {
        self.serializer.write_doctype(name);
        DocumentDoctypeSerializer { internal: self }
    }

    pub fn node<'i, I>(self, name: QualName, attrs: I) -> NodeSerializer<'a, 'b, 'w, W>
        where I: Iterator<Item = (&'i QualName, &'i str)>
    {
        element_normal(self.serializer, name, attrs)
    }
}

pub fn new_doc_ser<'a, 'b, 'w, W: Write>(ser: &'a mut Serializer<'b, 'w, W>)
                                         -> DocumentSerializer<'a, 'b, 'w, W> {
    DocumentSerializer { serializer: ser }
}

pub struct DocumentDoctypeSerializer<'a, 'b: 'a, 'w: 'b, W: 'w + Write> {
    internal: DocumentSerializer<'a, 'b, 'w, W>,
}

impl<'a, 'b, 'w, W: Write> DocumentDoctypeSerializer<'a, 'b, 'w, W> {
    pub fn node<'i, I>(self, name: QualName, attrs: I) -> NodeSerializer<'a, 'b, 'w, W>
        where I: Iterator<Item = (&'i QualName, &'i str)>
    {
        self.internal.node(name, attrs)
    }
}
