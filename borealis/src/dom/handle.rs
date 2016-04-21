
use std::cell::RefCell;
use std::io::Write;
use std::ops::Deref;
use std::rc::{Rc, Weak};

use html5ever::tree_builder::TreeSink;
use html5ever::tendril::{StrTendril, TendrilSink};

use string_cache::QualName;

use serializer::{SerializeDocument, SerializeNode, DocumentSerializer, NodeSerializer};

#[derive(Clone, Debug, PartialEq)]
pub enum Node {
    Comment(StrTendril),
    Document(Option<StrTendril>, Option<Handle>),
    Element(QualName, Vec<(QualName, StrTendril)>, Vec<Handle>),
    Text(StrTendril),
}

#[derive(Clone, Debug)]
pub struct Handle(Rc<RefCell<(Node, Option<WeakHandle>)>>);

impl Handle {
    pub fn downgrade(&self) -> WeakHandle {
        WeakHandle(Rc::downgrade(&self.0))
    }
}

impl From<Node> for Handle {
    fn from(node: Node) -> Handle {
        Handle(Rc::new(RefCell::new((node, None))))
    }
}

impl Deref for Handle {
    type Target = Rc<RefCell<(Node, Option<WeakHandle>)>>;

    fn deref(&self) -> &Rc<RefCell<(Node, Option<WeakHandle>)>> {
        &self.0
    }
}

impl PartialEq for Handle {
    fn eq(&self, other: &Handle) -> bool {
        self.borrow().0 == other.borrow().0
    }
}

impl SerializeDocument for Handle {
    fn serialize_document<W: Write>(self, s: DocumentSerializer<W>) {
        match *self.borrow() {
            (Node::Document(ref doctype, ref node), _) => {
                let mut s = match *doctype {
                    Some(ref name) => s.doctype(&name).node(),
                    None => s.node(),
                };

                if let Some(ref node) = *node {
                    node.serialize_node(&mut s)
                }
            }
            _ => panic!("expected document, got: {:?}", self),
        }
    }
}

impl<'a> SerializeNode for &'a Handle {
    fn serialize_node<W: Write>(self, s: &mut NodeSerializer<W>) {
        match *self.borrow() {
            (Node::Comment(ref comment), _) => s.comment(&comment),
            (Node::Element(ref name, ref attributes, ref children), _) => {
                let mut node = s.element(name.clone(), attributes.iter().map(|a| (&a.0, &a.1[..])));

                for child in children.iter() {
                    child.serialize_node(&mut node);
                }
            }
            (Node::Text(ref text), _) => s.text(&text),
            _ => panic!("expected comment, element or text, got: {:?}", self),
        }
    }
}

#[derive(Clone, Debug)]
pub struct WeakHandle(Weak<RefCell<(Node, Option<WeakHandle>)>>);

impl WeakHandle {
    pub fn upgrade(&self) -> Handle {
        Handle(self.0.upgrade().unwrap())
    }
}

impl Deref for WeakHandle {
    type Target = Weak<RefCell<(Node, Option<WeakHandle>)>>;

    fn deref(&self) -> &Weak<RefCell<(Node, Option<WeakHandle>)>> {
        &self.0
    }
}
