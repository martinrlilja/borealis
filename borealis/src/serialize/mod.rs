
use std::io::{Error, Result, Write};

use html5ever;
use html5ever::serialize::{SerializeOpts, TraversalScope};

use string_cache::QualName;

pub use self::document::{DocumentSerializer, DocumentDoctypeSerializer};
pub use self::node::NodeSerializer;

mod document;
mod node;

pub trait SerializeDocument {
    fn serialize_document<W: Write>(&self, DocumentSerializer<W>);
}

pub trait SerializeNode {
    fn serialize_node<W: Write>(self, &mut NodeSerializer<W>);
}

impl<I: SerializeNode, T: Iterator<Item = I>> SerializeNode for T {
    fn serialize_node<W: Write>(self, s: &mut NodeSerializer<W>) {
        for node in self.into_iter() {
            node.serialize_node(s);
        }
    }
}

pub struct Serializer<'a, 'w: 'a, W: 'w + Write> {
    inner: &'a mut html5ever::serialize::Serializer<'w, W>,
    error: Option<Error>,
}

impl<'a, 'w, W: Write> Serializer<'a, 'w, W> {
    fn do_cond<F>(&mut self, f: F)
        where F: FnOnce(&mut Serializer<W>) -> Result<()>
    {
        if self.error.is_some() {
            return;
        }

        let result = f(self);
        if let Err(err) = result {
            self.error = Some(err);
        }
    }

    pub fn start_elem<'i, T>(&mut self, name: &QualName, attrs: T)
        where T: Iterator<Item = (&'i QualName, &'i str)>
    {
        self.do_cond(|s| s.inner.start_elem(name.clone(), attrs))
    }

    pub fn end_elem(&mut self, name: &QualName) {
        self.do_cond(|s| s.inner.end_elem(name.clone()));
    }

    pub fn write_text(&mut self, text: &str) {
        self.do_cond(|s| s.inner.write_text(text));
    }

    pub fn write_comment(&mut self, comment: &str) {
        self.do_cond(|s| s.inner.write_comment(comment));
    }

    pub fn write_doctype(&mut self, name: &str) {
        self.do_cond(|s| s.inner.write_doctype(name));
    }
}

struct Serializable<'a, T: 'a + SerializeDocument>(&'a T);

impl<'a, T: SerializeDocument> html5ever::serialize::Serializable for Serializable<'a, T> {
    fn serialize<'w, W: Write>(&self,
                               serializer: &mut html5ever::serialize::Serializer<'w, W>,
                               _: TraversalScope)
                               -> Result<()> {
        let mut ser = Serializer {
            inner: serializer,
            error: None,
        };

        {
            let doc_ser = document::new_doc_ser(&mut ser);
            self.0.serialize_document(doc_ser);
        }

        if let Some(err) = ser.error {
            Err(err)
        } else {
            Ok(())
        }
    }
}

pub fn serialize<'w, W, T>(writer: &mut W, document: &T) -> Result<()>
    where W: 'w + Write,
          T: SerializeDocument
{
    html5ever::serialize::serialize(writer, &Serializable(document), SerializeOpts::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_empty_document() {
        struct Doc;

        impl SerializeDocument for Doc {
            fn serialize_document<W: Write>(&self, _: DocumentSerializer<W>) {}
        }

        assert_eq!(ser(&Doc), "");
    }

    #[test]
    fn test_basic_document() {
        struct Doc;

        impl SerializeDocument for Doc {
            fn serialize_document<W: Write>(&self, s: DocumentSerializer<W>) {
                s.doctype("html").node(&qualname!(html, "html"), vec![]);
            }
        }

        assert_eq!(ser(&Doc), "<!DOCTYPE html>\n<html></html>");
    }

    #[test]
    fn test_nodes() {
        struct Doc;

        impl SerializeDocument for Doc {
            fn serialize_document<W: Write>(&self, s: DocumentSerializer<W>) {
                let mut html = s.doctype("html").node(&qualname!(html, "html"), vec![]);
                {
                    let mut head = html.element_normal(&qualname!(html, "head"), vec![]);
                    {
                        let mut title = head.element_normal(&qualname!(html, "title"), vec![]);
                        title.text("test");
                    }
                }
                {
                    let mut body = html.element_normal(&qualname!(html, "body"), vec![]);
                    body.text("more tests!");
                }
            }
        }

        assert_eq!(ser(&Doc),
                   "<!DOCTYPE html>\n<html><head><title>test</title></head><body>more \
                    tests!</body></html>");
    }

    #[test]
    fn test_nodes_ser() {
        struct Doc;

        impl SerializeDocument for Doc {
            fn serialize_document<W: Write>(&self, s: DocumentSerializer<W>) {
                let mut html = s.doctype("html").node(&qualname!(html, "html"), vec![]);
                let mut body = html.element_normal(&qualname!(html, "body"), vec![]);
                Node(0).serialize_node(&mut body);

                vec![Node(1), Node(2)].iter().serialize_node(&mut body);
                (3..5).map(Node).serialize_node(&mut body);
            }
        }

        struct Node(i32);

        impl SerializeNode for Node {
            fn serialize_node<W: Write>(self, s: &mut NodeSerializer<W>) {
                (&self).serialize_node(s);
            }
        }

        impl<'a> SerializeNode for &'a Node {
            fn serialize_node<W: Write>(self, s: &mut NodeSerializer<W>) {
                let mut p = s.element_normal(&qualname!(html, "p"), vec![]);
                p.text(&format!("{}", self.0));
            }
        }

        assert_eq!(ser(&Doc),
                   "<!DOCTYPE \
                    html>\n<html><body><p>0</p><p>1</p><p>2</p><p>3</p><p>4</p></body></html>");
    }

    fn ser<T: SerializeDocument>(document: &T) -> String {
        let mut writer = Vec::new();
        serialize(&mut writer, document).unwrap();
        String::from_utf8(writer).unwrap()
    }
}
