
use std::cell::RefCell;
use std::io::{Result, Write};
use std::mem;

use html5ever;
use html5ever::serialize::{SerializeOpts, TraversalScope};

pub use self::document::{DocumentSerializer, DocumentDoctypeSerializer};
pub use self::empty_attrs::EmptyAttrs;
pub use self::node::NodeSerializer;
use self::serializer::Serializer;

mod document;
mod empty_attrs;
mod node;
mod serializer;

pub trait SerializeDocument {
    fn serialize_document<W: Write>(self, DocumentSerializer<W>);
}

pub trait SerializeNode {
    fn serialize_node<W: Write>(self, &mut NodeSerializer<W>);
}

impl SerializeNode for String {
    fn serialize_node<W: Write>(self, s: &mut NodeSerializer<W>) {
        s.text(&self);
    }
}

impl<'a> SerializeNode for &'a String {
    fn serialize_node<W: Write>(self, s: &mut NodeSerializer<W>) {
        s.text(&self);
    }
}

impl<'a> SerializeNode for &'a str {
    fn serialize_node<W: Write>(self, s: &mut NodeSerializer<W>) {
        s.text(self);
    }
}

pub trait SerializeNodes {
    fn serialize_node<W: Write>(self, &mut NodeSerializer<W>);
}

impl<I: SerializeNode, T: IntoIterator<Item = I>> SerializeNodes for T {
    fn serialize_node<W: Write>(self, s: &mut NodeSerializer<W>) {
        for node in self {
            node.serialize_node(s);
        }
    }
}

struct Serializable<T: SerializeDocument>(RefCell<Option<T>>);

impl<T: SerializeDocument> html5ever::serialize::Serializable for Serializable<T> {
    fn serialize<'w, W: Write>(&self,
                               serializer: &mut html5ever::serialize::Serializer<'w, W>,
                               _: TraversalScope)
                               -> Result<()> {
        let mut serializer = Serializer::new(serializer);

        {
            let mut doc = self.0.borrow_mut();
            let doc = if doc.is_some() {
                let mut x = None;
                mem::swap(&mut (*doc), &mut x);
                x.unwrap()
            } else {
                panic!("value already used");
            };

            let doc_ser = document::new_doc_ser(&mut serializer);
            doc.serialize_document(doc_ser);
        }

        serializer.error()
    }
}

pub fn serialize<'w, W, T>(writer: &mut W, document: T) -> Result<()>
    where W: 'w + Write,
          T: SerializeDocument
{
    html5ever::serialize::serialize(writer, &Serializable(RefCell::new(Some(document))), SerializeOpts::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "nightly")]
    use test::Bencher;
    use std::io::Write;

    #[test]
    fn test_empty_document() {
        struct Doc;

        impl SerializeDocument for Doc {
            fn serialize_document<W: Write>(self, _: DocumentSerializer<W>) {}
        }

        assert_eq!(ser(Doc), "");
    }

    #[test]
    fn test_basic_document() {
        struct Doc;

        impl SerializeDocument for Doc {
            fn serialize_document<W: Write>(self, s: DocumentSerializer<W>) {
                let mut s = s.doctype("html").node();
                s.element_normal(qualname!(html, "html"), EmptyAttrs::new());
            }
        }

        assert_eq!(ser(Doc), "<!DOCTYPE html><html></html>");
    }

    #[test]
    fn test_nodes() {
        struct Doc;

        impl SerializeDocument for Doc {
            fn serialize_document<W: Write>(self, s: DocumentSerializer<W>) {
                let mut s = s.doctype("html").node();
                let mut html = s.element_normal(qualname!(html, "html"), EmptyAttrs::new());
                {
                    let mut head = html.element_normal(qualname!(html, "head"), EmptyAttrs::new());
                    {
                        let mut title = head.element_normal(qualname!(html, "title"),
                                                            EmptyAttrs::new());
                        title.text("test");
                    }
                }
                {
                    let mut body = html.element_normal(qualname!(html, "body"), EmptyAttrs::new());
                    body.text("more tests!");
                }
            }
        }

        assert_eq!(ser(Doc),
                   "<!DOCTYPE html><html><head><title>test</title></head><body>more \
                    tests!</body></html>");
    }

    #[test]
    fn test_nodes_ser() {
        struct Doc;

        impl SerializeDocument for Doc {
            fn serialize_document<W: Write>(self, s: DocumentSerializer<W>) {
                let mut s = s.doctype("html").node();
                let mut html = s.element_normal(qualname!(html, "html"), EmptyAttrs::new());
                let mut body = html.element_normal(qualname!(html, "body"), EmptyAttrs::new());
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
                let mut p = s.element_normal(qualname!(html, "p"), EmptyAttrs::new());
                p.text(&format!("{}", self.0));
            }
        }

        assert_eq!(ser(Doc),
                   "<!DOCTYPE \
                    html><html><body><p>0</p><p>1</p><p>2</p><p>3</p><p>4</p></body></html>");
    }

    #[cfg(feature = "nightly")]
    #[bench]
    fn bench_serialize_document(b: &mut Bencher) {
        struct Doc;

        impl SerializeDocument for Doc {
            fn serialize_document<W: Write>(self, s: DocumentSerializer<W>) {
                let mut s = s.doctype("html").node();
                let mut html = s.element_normal(qualname!(html, "html"), EmptyAttrs::new());
                {
                    let mut head = html.element_normal(qualname!(html, "head"), EmptyAttrs::new());
                    {
                        let mut title = head.element_normal(qualname!(html, "title"),
                                                            EmptyAttrs::new());
                        title.text("test");
                    }
                }
                {
                    let mut body = html.element_normal(qualname!(html, "body"), EmptyAttrs::new());
                    body.text("more tests!");
                }
            }
        }

        b.iter(|| {
            let mut writer = Vec::new();
            serialize(&mut writer, Doc).unwrap();
            writer
        });
    }

    fn ser<T: SerializeDocument>(document: T) -> String {
        let mut writer = Vec::new();
        serialize(&mut writer, document).unwrap();
        String::from_utf8(writer).unwrap()
    }
}
