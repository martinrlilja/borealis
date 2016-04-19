
use std::borrow::Cow;
use std::cell::{Ref, RefCell};
use std::collections::HashSet;
use std::io::Write;
use std::ops::Deref;
use std::rc::{Rc, Weak};

use html5ever::{ParseOpts, parse_document, parse_fragment};
use html5ever::Attribute;
use html5ever::tree_builder::{TreeSink, QuirksMode, NodeOrText};
use html5ever::tendril::{StrTendril, TendrilSink};

use string_cache::QualName;

use serializer::{SerializeDocument, SerializeNode, DocumentSerializer, NodeSerializer};

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
    fn downgrade(&self) -> WeakHandle {
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
                let mut node = s.element_normal(name.clone(),
                                                attributes.iter().map(|a| (&a.0, &a.1[..])));

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
    fn upgrade(&self) -> Handle {
        Handle(self.0.upgrade().unwrap())
    }
}

impl Deref for WeakHandle {
    type Target = Weak<RefCell<(Node, Option<WeakHandle>)>>;

    fn deref(&self) -> &Weak<RefCell<(Node, Option<WeakHandle>)>> {
        &self.0
    }
}

#[derive(Debug)]
pub struct Dom {
    document: Handle,
    errors: Vec<Cow<'static, str>>,
    quirks_mode: QuirksMode,
}

impl Dom {
    pub fn new() -> Dom {
        Dom {
            document: Node::Document(None, None).into(),
            errors: Vec::new(),
            quirks_mode: QuirksMode::NoQuirks,
        }
    }

    pub fn document(&self) -> Handle {
        self.document.clone()
    }

    pub fn fragment(&self) -> Vec<Handle> {
        fn element_children<'a>(node: &'a Ref<(Node, Option<WeakHandle>)>) -> &'a [Handle] {
            match **node {
                (Node::Element(_, _, ref children), _) => &children,
                _ => panic!("expected element, got: {:?}", node),
            }
        }

        fn is_user_element(name: &QualName) -> bool {
            match *name {
                qualname!(html, "head") => false,
                qualname!(html, "body") => false,
                _ => true,
            }
        }

        let html = match *self.document.borrow() {
            (Node::Document(_, ref child), _) => child.clone().unwrap(),
            _ => panic!("expected document, got: {:?}", self.document),
        };

        let mut children = Vec::new();
        for child in element_children(&html.borrow()).iter() {
            match *child.borrow() {
                (Node::Element(ref name, _, _), _) => {
                    if is_user_element(name) {
                        children.push(child.clone());
                    }
                }
                (Node::Comment(..), _) => children.push(child.clone()),
                (Node::Text(..), _) => children.push(child.clone()),
                _ => panic!("expected element, got: {:?}", child),
            }
        }

        children
    }

    fn node_or_text_as_handle(child: &NodeOrText<Handle>) -> Handle {
        match child {
            &NodeOrText::AppendText(ref text) => Node::Text(text.clone()).into(),
            &NodeOrText::AppendNode(ref node) => node.clone(),
        }
    }
}

impl TreeSink for Dom {
    type Output = Self;
    type Handle = Handle;

    fn finish(self) -> Dom {
        self
    }

    fn parse_error(&mut self, message: Cow<'static, str>) {
        self.errors.push(message);
    }

    fn get_document(&mut self) -> Handle {
        self.document.clone()
    }

    fn get_template_contents(&self, target: Handle) -> Handle {
        target
    }

    fn set_quirks_mode(&mut self, quirks_mode: QuirksMode) {
        self.quirks_mode = quirks_mode;
    }

    fn same_node(&self, a: Handle, b: Handle) -> bool {
        a == b
    }

    fn elem_name(&self, target: Handle) -> QualName {
        match *target.borrow() {
            (Node::Element(ref name, _, _), _) => name.clone(),
            _ => panic!("expected element, got: {:?}", target),
        }
    }

    fn create_element(&mut self, name: QualName, attributes: Vec<Attribute>) -> Handle {
        let attributes = attributes.into_iter().map(|a| (a.name, a.value)).collect();
        Node::Element(name, attributes, Vec::new()).into()
    }

    fn create_comment(&mut self, text: StrTendril) -> Handle {
        Node::Comment(text).into()
    }

    fn append(&mut self, parent: Handle, child: NodeOrText<Handle>) {
        match *parent.borrow_mut() {
            (Node::Document(_, ref mut old_child), _) => {
                assert!(old_child.is_none());

                let child = Dom::node_or_text_as_handle(&child);
                child.borrow_mut().1 = Some(parent.downgrade());

                *old_child = Some(child);
            }
            (Node::Element(_, _, ref mut children), _) => {
                let child = Dom::node_or_text_as_handle(&child);
                child.borrow_mut().1 = Some(parent.downgrade());

                children.push(child);
            }
            _ => panic!("expected document or element, got: {:?}", parent),
        }
    }

    fn append_before_sibling(&mut self,
                             sibling: Handle,
                             child: NodeOrText<Handle>)
                             -> Result<(), NodeOrText<Handle>> {
        let (_, ref parent) = *sibling.borrow();
        let child = Dom::node_or_text_as_handle(&child);
        child.borrow_mut().1 = parent.clone();

        let parent = parent.clone().unwrap().upgrade();

        match *parent.borrow_mut() {
            (Node::Element(_, _, ref mut children), _) => {
                let index = children.iter()
                                    .position(|e| *e == sibling)
                                    .unwrap();

                children.insert(index, child);
            }
            _ => panic!("expected element, got: {:?}", parent),
        }

        Ok(())
    }

    fn append_doctype_to_document(&mut self, name: StrTendril, _: StrTendril, _: StrTendril) {
        match *self.document.borrow_mut() {
            (Node::Document(ref mut doctype, _), _) => {
                *doctype = Some(name);
            }
            _ => panic!("expected document"),
        }
    }

    fn add_attrs_if_missing(&mut self, target: Handle, attrs: Vec<Attribute>) {
        fn join_attributes(old_attrs: &mut Vec<(QualName, StrTendril)>,
                           new_attrs: Vec<Attribute>) {
            let names = old_attrs.iter().map(|a| a.0.clone()).collect::<HashSet<_>>();
            let missing = new_attrs.into_iter().filter(|a| !names.contains(&a.name));

            old_attrs.extend(missing.map(|a| (a.name, a.value)));
        }

        match *target.borrow_mut() {
            (Node::Element(_, ref mut old_attrs, _), _) => {
                join_attributes(old_attrs, attrs);
            }
            _ => panic!("expected element, got {:?}", target),
        }
    }

    fn remove_from_parent(&mut self, target: Handle) {
        let parent = {
            let (_, ref mut target_parent) = *target.borrow_mut();
            let parent = target_parent.clone().unwrap().upgrade();

            *target_parent = None;
            parent
        };

        match *parent.borrow_mut() {
            (Node::Document(_, ref mut old_child), _) => {
                assert_eq!(old_child.as_ref(), Some(&target));
                *old_child = None;
            }
            (Node::Element(_, _, ref mut children), _) => {
                let index = children.iter()
                                    .position(|e| *e == target)
                                    .unwrap();
                children.remove(index);
            }
            _ => panic!("expected document or element handle, got {:?}", parent),
        };
    }

    fn reparent_children(&mut self, old_parent: Handle, new_parent: Handle) {
        match *old_parent.borrow_mut() {
            (Node::Document(_, ref mut child), _) => {
                if let &mut Some(ref child) = child {
                    self.append(new_parent, NodeOrText::AppendNode(child.clone()));
                }

                *child = None;
            }
            (Node::Element(_, _, ref mut children), _) => {
                for child in children.iter() {
                    let child = NodeOrText::AppendNode(child.clone());
                    self.append(new_parent.clone(), child);
                }

                children.clear();
            }
            _ => panic!("expected document or element, got: {:?}", old_parent),
        }
    }

    fn mark_script_already_started(&mut self, _: Handle) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "nightly")]
    use test::Bencher;

    use html5ever::Attribute;
    use html5ever::tree_builder::{TreeSink, NodeOrText};
    use html5ever::tendril::StrTendril;

    use string_cache::QualName;

    fn convert_attr(a: &(QualName, StrTendril)) -> Attribute {
        Attribute {
            name: a.0.clone(),
            value: a.1.clone(),
        }
    }


    #[test]
    fn test_fragment() {
        let mut dom = Dom::new();
        let document = dom.get_document();
        let html = dom.create_element(qualname!(html, "html"), Vec::new());
        let head = dom.create_element(qualname!(html, "head"), Vec::new());
        let body = dom.create_element(qualname!(html, "body"), Vec::new());
        let element = dom.create_element(qualname!(html, "div"), Vec::new());

        dom.append(document, NodeOrText::AppendNode(html.clone()));
        dom.append(html.clone(), NodeOrText::AppendNode(head.clone()));
        dom.append(html.clone(), NodeOrText::AppendNode(body.clone()));
        dom.append(html.clone(), NodeOrText::AppendNode(element.clone()));

        let fragment = dom.fragment();
        assert!(fragment.len() == 1);
        assert_eq!(fragment[0], element);
    }

    #[test]
    fn test_get_template_contents() {
        let mut dom = Dom::new();
        let template = dom.create_element(qualname!(html, "template"), Vec::new());
        let element = dom.create_element(qualname!(html, "div"), Vec::new());
        let document = dom.get_document();

        dom.append(document.clone(), NodeOrText::AppendNode(template.clone()));
        dom.append(template.clone(), NodeOrText::AppendNode(element.clone()));

        assert_eq!(dom.get_template_contents(template.clone()), template);
    }

    #[test]
    fn test_same_node() {
        let mut dom = Dom::new();

        let handle_a = dom.create_element(qualname!(html, "div"), Vec::new());
        let handle_b = dom.create_element(qualname!(html, "p"), Vec::new());

        assert!(dom.same_node(handle_a.clone(), handle_a.clone()));
        assert!(dom.same_node(handle_b.clone(), handle_b.clone()));
        assert!(!dom.same_node(handle_a.clone(), handle_b.clone()));
        assert!(!dom.same_node(handle_b.clone(), handle_a.clone()));
    }

    #[test]
    fn test_elem_name() {
        let mut dom = Dom::new();
        let name = qualname!(html, "div");
        let element = dom.create_element(name.clone(), Vec::new());

        assert_eq!(name, dom.elem_name(element));
    }

    fn do_test_create_element(dom: &mut Dom, name: QualName) {
        let attrs = &[(qualname!("", "name"), "test".into()), (qualname!("", "id"), "yup".into())];
        let element = dom.create_element(name.clone(), attrs.iter().map(convert_attr).collect());

        match *element.borrow() {
            (Node::Element(ref elem_name, ref elem_attrs, ref elem_children),
             ref elem_parent) => {
                assert_eq!(name, *elem_name);
                assert_eq!(attrs, &elem_attrs[..]);
                assert!(elem_children.is_empty());
                assert!(elem_parent.is_none());
            }
            _ => panic!("created element is not an element: {:?}", element),
        };
    }

    #[test]
    fn test_create_element() {
        let mut dom = Dom::new();

        do_test_create_element(&mut dom, qualname!(html, "template"));
        do_test_create_element(&mut dom, qualname!(html, "html"));
    }

    #[cfg(feature = "nightly")]
    #[bench]
    fn bench_create_element(b: &mut Bencher) {
        let mut dom = Dom::new();

        b.iter(|| {
            dom.create_element(qualname!(html, "html"), Vec::new());
        });
    }

    #[test]
    fn test_create_comment() {
        let mut dom = Dom::new();
        let text = StrTendril::from("sup".to_owned());
        let comment = dom.create_comment(text.clone());

        match *comment.borrow() {
            (Node::Comment(ref comment_text), ref comment_parent) => {
                assert_eq!(text, *comment_text);
                assert!(comment_parent.is_none());
            }
            _ => panic!("created comment is not a comment: {:?}", comment),
        };
    }

    #[cfg(feature = "nightly")]
    #[bench]
    fn bench_create_comment(b: &mut Bencher) {
        let mut dom = Dom::new();
        let text = StrTendril::from("sup".to_owned());

        b.iter(|| {
            dom.create_comment(text.clone());
        });
    }

    #[test]
    fn test_append() {
        let mut dom = Dom::new();
        let document = dom.get_document();
        let html = dom.create_element(qualname!(html, "html"), Vec::new());
        let body = dom.create_element(qualname!(html, "body"), Vec::new());

        {
            dom.append(document.clone(), NodeOrText::AppendNode(html.clone()));

            match *html.borrow() {
                (Node::Element(_, _, _), ref parent) => {
                    assert_eq!(document, parent.clone().unwrap().upgrade());
                }
                _ => panic!("html is not an element: {:?}", html),
            }

            match *document.borrow() {
                (Node::Document(_, ref child), _) => {
                    assert_eq!(html, child.clone().unwrap());
                }
                _ => panic!("document is not a document: {:?}", document),
            }
        }

        {
            dom.append(html.clone(), NodeOrText::AppendNode(body.clone()));

            match *body.borrow() {
                (Node::Element(_, _, _), ref parent) => {
                    assert_eq!(html, parent.clone().unwrap().upgrade());
                }
                _ => panic!("body is not an element: {:?}", body),
            }

            match *html.borrow() {
                (Node::Element(_, _, ref children), _) => {
                    assert!(children.iter().any(|c| *c == body));
                }
                _ => panic!("html is not an element: {:?}", html),
            };
        }
    }

    #[test]
    fn test_append_before_sibling() {
        let mut dom = Dom::new();
        let document = dom.get_document();
        let html = dom.create_element(qualname!(html, "html"), Vec::new());
        let head = dom.create_element(qualname!(html, "head"), Vec::new());
        let body = dom.create_element(qualname!(html, "body"), Vec::new());

        dom.append(document.clone(), NodeOrText::AppendNode(html.clone()));
        dom.append(html.clone(), NodeOrText::AppendNode(body.clone()));

        assert!(dom.append_before_sibling(body.clone(), NodeOrText::AppendNode(head.clone()))
                   .is_ok());

        match *html.borrow() {
            (Node::Element(_, _, ref children), _) => {
                assert_eq!(&children[..], &[head, body]);
            }
            _ => panic!("html is not an element: {:?}", html),
        };
    }

    #[test]
    fn test_append_doctype_to_document() {
        let mut dom = Dom::new();
        let document = dom.get_document();
        let name = StrTendril::from("a".to_owned());
        let public_id = StrTendril::from("b".to_owned());
        let system_id = StrTendril::from("c".to_owned());

        dom.append_doctype_to_document(name.clone(), public_id.clone(), system_id.clone());

        match *document.borrow() {
            (Node::Document(ref doctype, _), _) => {
                let doctype = doctype.as_ref().unwrap();
                assert_eq!(*doctype, name);
            }
            _ => panic!("document is not a document: {:?}", document),
        };
    }

    #[test]
    fn test_add_attrs_if_missing() {
        let mut dom = Dom::new();
        let attrs = &[(qualname!("", "name"), "test".into()), (qualname!("", "id"), "yup".into())];

        let html = dom.create_element(qualname!(html, "html"),
                                      attrs.iter().take(1).map(convert_attr).collect());
        dom.add_attrs_if_missing(html.clone(), attrs.iter().map(convert_attr).collect());

        match *html.borrow() {
            (Node::Element(_, ref elem_attrs, _), _) => {
                assert_eq!(attrs, &elem_attrs[..]);
            }
            _ => panic!("html is not an element: {:?}", html),
        };
    }

    #[test]
    fn test_remove_from_parent() {
        let mut dom = Dom::new();
        let document = dom.get_document();
        let html = dom.create_element(qualname!(html, "html"), Vec::new());
        let head = dom.create_element(qualname!(html, "head"), Vec::new());
        let body = dom.create_element(qualname!(html, "body"), Vec::new());

        dom.append(document.clone(), NodeOrText::AppendNode(html.clone()));
        dom.append(html.clone(), NodeOrText::AppendNode(head.clone()));
        dom.append(html.clone(), NodeOrText::AppendNode(body.clone()));

        dom.remove_from_parent(head.clone());

        match *html.borrow() {
            (Node::Element(_, _, ref children), _) => {
                assert_eq!(&children[..], &[body.clone()]);
            }
            _ => panic!("html is not an element: {:?}", html),
        }

        dom.remove_from_parent(html.clone());

        match *document.borrow() {
            (Node::Document(_, ref child), _) => {
                assert!(child.is_none());
            }
            _ => panic!("document is not a document: {:?}", document),
        }

        dom.append(document.clone(), NodeOrText::AppendNode(html.clone()));

        dom.remove_from_parent(body.clone());

        match *document.borrow() {
            (Node::Document(_, ref child), _) => {
                assert_eq!(*child, Some(html));
            }
            _ => panic!("document is not a document: {:?}", document),
        };
    }

    #[test]
    fn test_reparent_children() {
        let mut dom = Dom::new();
        let document = dom.get_document();
        let html = dom.create_element(qualname!(html, "html"), Vec::new());
        let head = dom.create_element(qualname!(html, "head"), Vec::new());
        let body = dom.create_element(qualname!(html, "body"), Vec::new());
        let target = dom.create_element(qualname!(html, "div"), Vec::new());

        dom.append(document.clone(), NodeOrText::AppendNode(html.clone()));
        dom.append(html.clone(), NodeOrText::AppendNode(head.clone()));
        dom.append(html.clone(), NodeOrText::AppendNode(body.clone()));

        dom.reparent_children(html.clone(), target.clone());

        match *html.borrow() {
            (Node::Element(_, _, ref children), _) => {
                assert!(children.is_empty());
            }
            _ => panic!("html is not an element: {:?}", html),
        }

        match *target.borrow() {
            (Node::Element(_, _, ref children), _) => {
                assert_eq!(&children[..], &[head, body]);
            }
            _ => panic!("target is not an element: {:?}", target),
        }

        dom.reparent_children(target.clone(), html.clone());
        dom.reparent_children(document.clone(), target.clone());

        match *document.borrow() {
            (Node::Document(_, ref child), _) => {
                assert!(child.is_none());
            }
            _ => panic!("document is not a document: {:?}", document),
        }

        match *target.borrow() {
            (Node::Element(_, _, ref children), _) => {
                assert_eq!(&children[..], &[html]);
            }
            _ => panic!("target is not an element: {:?}", target),
        };
    }
}
