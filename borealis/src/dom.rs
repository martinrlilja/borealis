
use std::borrow::Cow;
use std::collections::HashSet;

use html5ever;
use html5ever::tree_builder::{TreeSink, QuirksMode, NodeOrText};
use html5ever::tendril::StrTendril;

use string_cache::QualName;

use html::{CommentNode, Doctype, Document, ElementNode, ElementType, Handle, Node, ParentHandle,
           TextNode};

#[derive(Debug)]
pub struct Dom {
    document: Handle,
    errors: Vec<Cow<'static, str>>,
    quirks_mode: QuirksMode,
}

impl Dom {
    pub fn new() -> Dom {
        Dom {
            document: Handle::new_document(Document::new(None, None)),
            errors: Vec::new(),
            quirks_mode: QuirksMode::NoQuirks,
        }
    }

    fn node_or_text_as_handle(child: &NodeOrText<Handle>) -> Handle {
        match child {
            &NodeOrText::AppendText(ref text) => {
                Handle::new_node(Node::Text(TextNode::new(text.clone())))
            }
            &NodeOrText::AppendNode(ref node) => node.clone(),
        }
    }

    fn remove_child_from_parent(parent: &ParentHandle, child: &Handle) {
        match *parent {
            ParentHandle::DocumentHandle(ref document) => {
                let document = document.upgrade();
                assert_eq!(document.borrow().child(), Some(child.expect_node()));

                document.borrow_mut().unset_child();
            }
            ParentHandle::NodeHandle(ref node) => {
                let node = node.upgrade();
                let mut node = node.borrow_mut();
                let mut children = node.expect_element_mut().expect_normal_mut();
                let index = children.iter()
                                    .position(|e| Handle::from(e.clone()) == *child)
                                    .expect(&format!("Dom::remove_child_from_parent(parent: \
                                                      {:?}, child: {:?}), child is child of \
                                                      parent.",
                                                     parent,
                                                     child));

                children.remove(index);
            }
        }
    }
}

impl Default for Dom {
    fn default() -> Dom {
        Dom::new()
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
        target.expect_node()
              .borrow()
              .expect_element()
              .expect_template()
              .clone()
              .into()
    }

    fn set_quirks_mode(&mut self, quirks_mode: QuirksMode) {
        self.quirks_mode = quirks_mode;
    }

    fn same_node(&self, a: Handle, b: Handle) -> bool {
        a == b
    }

    fn elem_name(&self, target: Handle) -> QualName {
        target.expect_node()
              .borrow()
              .expect_element()
              .name()
              .clone()
    }

    fn create_element(&mut self, name: QualName, attributes: Vec<html5ever::Attribute>) -> Handle {
        let element_type = match name {
            qualname!(html, "template") => ElementType::new_template(),
            _ => ElementType::new_normal(),
        };

        Handle::new_node(Node::Element(ElementNode::new(name,
                                                        element_type,
                                                        attributes.iter()
                                                                  .map(|a| a.clone().into())
                                                                  .collect())))
    }

    fn create_comment(&mut self, text: StrTendril) -> Handle {
        Handle::new_node(Node::Comment(CommentNode::new(text)))
    }

    fn append(&mut self, parent: Handle, child: NodeOrText<Handle>) {
        let child = Dom::node_or_text_as_handle(&child);
        let child = child.expect_node();
        child.borrow_mut()
             .set_parent(parent.clone().into());

        match parent {
            Handle::DoctypeHandle(_) => panic!("Cannot append to doctype."),
            Handle::DocumentHandle(ref document) => {
                document.borrow_mut()
                        .set_child(child.clone());
            }
            Handle::NodeHandle(ref node) => {
                let mut node = node.borrow_mut();
                let mut node = node.expect_element_mut();

                match *node.element_type_mut() {
                    ElementType::Normal(ref mut children) => {
                        children.push(child.clone());
                    }
                    ElementType::Template(ref document) => {
                        self.append(document.clone().into(),
                                    NodeOrText::AppendNode(child.clone().into()));
                    }
                }
            }
        }
    }

    fn append_before_sibling(&mut self,
                             sibling: Handle,
                             child: NodeOrText<Handle>)
                             -> Result<(), NodeOrText<Handle>> {
        let node = Dom::node_or_text_as_handle(&child);

        let parent = try!(sibling.expect_node().borrow().parent().ok_or(child)).expect_node();
        let child = node.expect_node();
        let mut children = parent.borrow_mut();
        let mut children = children.expect_element_mut().expect_normal_mut();
        let index = children.iter()
                            .position(|e| Handle::from(e.clone()) == sibling)
                            .expect(&format!("Dom::append_before_sibling(sibling: {:?}, child: \
                                              {:?}), before is not child of self.",
                                             sibling,
                                             child));

        child.borrow_mut()
             .set_parent(parent.downgrade().as_handle());
        children.insert(index, child.clone());

        Ok(())
    }

    fn append_doctype_to_document(&mut self,
                                  name: StrTendril,
                                  public_id: StrTendril,
                                  system_id: StrTendril) {
        self.document
            .expect_document()
            .borrow_mut()
            .set_doctype(Doctype::new(name, public_id, system_id));
    }

    fn add_attrs_if_missing(&mut self, target: Handle, attrs: Vec<html5ever::Attribute>) {
        let mut target = target.expect_node().borrow_mut();
        let mut attributes = target.expect_element_mut().attributes_mut();

        let names = attributes.iter().map(|a| a.name().clone()).collect::<HashSet<_>>();
        let missing = attrs.into_iter().filter(|a| !names.contains(&a.name));
        attributes.extend(missing.map(|a| a.clone().into()));
    }

    fn remove_from_parent(&mut self, target: Handle) {
        match target {
            Handle::DoctypeHandle(_) => panic!("Cannot remove doctype from parent."),
            Handle::DocumentHandle(ref child) => {
                let parent = {
                    let document = child.borrow();
                    document.parent()
                            .expect(&format!("Dom::remove_from_parent(target: {:?}), target \
                                              must have a parent.",
                                             target))
                            .clone()
                };

                Dom::remove_child_from_parent(&parent, &target);
                child.borrow_mut().unset_parent();
            }
            Handle::NodeHandle(ref child) => {
                let parent = {
                    let node = child.borrow();
                    node.parent()
                        .expect(&format!("Dom::remove_from_parent(target: {:?}), target must \
                                          have a parent.",
                                         target))
                        .clone()
                };

                Dom::remove_child_from_parent(&parent, &target);
                child.borrow_mut().unset_parent();
            }
        }
    }

    fn reparent_children(&mut self, old_parent: Handle, new_parent: Handle) {
        match old_parent {
            Handle::DoctypeHandle(_) => (),
            Handle::DocumentHandle(ref parent) => {
                if let Some(child) = parent.borrow().child() {
                    let child = NodeOrText::AppendNode(child.clone().into());
                    self.append(new_parent, child);
                }

                parent.borrow_mut().unset_child();
            }
            Handle::NodeHandle(ref parent) => {
                let mut parent = parent.borrow_mut();
                let mut children = parent.expect_element_mut().expect_normal_mut();

                for child in children.iter() {
                    let child = NodeOrText::AppendNode(child.clone().into());
                    self.append(new_parent.clone(), child);
                }

                children.clear();
            }
        }
    }

    fn mark_script_already_started(&mut self, _: Handle) {}
}

#[cfg(test)]
mod tests {
    use super::Dom;
    use test::Bencher;

    use html5ever::driver::{parse_document, ParseOpts};
    use html5ever::tree_builder::{TreeSink, NodeOrText};
    use html5ever::tendril::{StrTendril, TendrilSink};

    use string_cache::QualName;

    use html::{Attribute, Document, Handle};

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
            let parser = parse_document(Dom::default(), ParseOpts::default()).from_utf8();
            parser.one(DOCUMENT.as_bytes());
        });
    }

    #[test]
    fn test_get_template_contents() {
        let mut dom = Dom::new();
        let template = dom.create_element(qualname!(html, "template"), Vec::new());
        let element = dom.create_element(qualname!(html, "div"), Vec::new());
        let document = dom.get_document();

        dom.append(document.clone(), NodeOrText::AppendNode(template.clone()));
        dom.append(template.clone(), NodeOrText::AppendNode(element.clone()));

        let document = Handle::new_document(Document::new(None,
                                                          Some(element.expect_node().clone())));
        assert!(dom.same_node(dom.get_template_contents(template.clone()),
                              document.clone()));
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
        let attrs = &[Attribute::new_str("name", "test"), Attribute::new_str("id", "yup")];

        let element = dom.create_element(name.clone(),
                                         attrs.iter().map(|a| a.clone().into()).collect());
        let node = element.expect_node().borrow();
        let node = node.expect_element();

        assert_eq!(element.expect_node().borrow().expect_element().attributes(),
                   attrs);

        assert_eq!(name, *node.name());

        match name {
            qualname!(html, "template") => {
                node.expect_template();
            }
            _ => {
                node.expect_normal();
            }
        }
    }

    #[test]
    fn test_create_element() {
        let mut dom = Dom::new();

        do_test_create_element(&mut dom, qualname!(html, "template"));
        do_test_create_element(&mut dom, qualname!(html, "html"));
    }

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

        let comment = comment.expect_node().borrow();
        let comment = comment.expect_comment();

        assert_eq!(text, *comment.text());
    }

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

            let parent: Handle = html.expect_node()
                                     .borrow()
                                     .parent()
                                     .unwrap()
                                     .expect_document()
                                     .clone()
                                     .into();
            assert_eq!(parent, document);

            let child: Handle = document.expect_document()
                                        .borrow()
                                        .child()
                                        .unwrap()
                                        .clone()
                                        .into();
            assert_eq!(child, html);
        }

        {
            dom.append(html.clone(), NodeOrText::AppendNode(body.clone()));

            let parent: Handle = body.expect_node()
                                     .borrow()
                                     .parent()
                                     .unwrap()
                                     .expect_node()
                                     .clone()
                                     .into();
            assert_eq!(parent, html);

            assert!(html.expect_node()
                        .borrow()
                        .expect_element()
                        .expect_normal()
                        .iter()
                        .any(|c| Handle::from(c.clone()) == body));
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

        let html = html.expect_node().borrow();
        let children = html.expect_element().expect_normal();

        assert_eq!(children.iter().map(|c| c).collect::<Vec<_>>(),
                   &[head.expect_node(), body.expect_node()]);
    }

    #[test]
    fn test_append_doctype_to_document() {
        let mut dom = Dom::new();
        let document = dom.get_document();
        let name = StrTendril::from("a".to_owned());
        let public_id = StrTendril::from("b".to_owned());
        let system_id = StrTendril::from("c".to_owned());

        dom.append_doctype_to_document(name.clone(), public_id.clone(), system_id.clone());

        let document = document.expect_document().borrow();
        let doctype = document.doctype().unwrap();

        assert_eq!(*doctype.name(), name);
        assert_eq!(*doctype.public_id(), public_id);
        assert_eq!(*doctype.system_id(), system_id);
    }

    #[test]
    fn test_add_attrs_if_missing() {
        let mut dom = Dom::new();
        let attrs = &[Attribute::new_str("name", "test"), Attribute::new_str("id", "yup")];

        let html = dom.create_element(qualname!(html, "html"),
                                      attrs.iter().take(1).map(|a| a.clone().into()).collect());
        dom.add_attrs_if_missing(html.clone(),
                                 attrs.iter().map(|a| a.clone().into()).collect());

        assert_eq!(html.expect_node().borrow().expect_element().attributes(),
                   attrs);
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

        {
            dom.remove_from_parent(head.clone());

            let html = html.expect_node().borrow();
            let children = html.expect_element().expect_normal();
            assert_eq!(children.iter().map(|c| c).collect::<Vec<_>>(),
                       &[body.clone().expect_node()]);
        }

        {
            dom.remove_from_parent(html.clone());

            let document = document.expect_document().borrow();
            assert_eq!(document.child(), None);
        }

        dom.append(document.clone(), NodeOrText::AppendNode(html.clone()));

        {
            dom.remove_from_parent(body.clone());

            let document = document.expect_document().borrow();
            assert_eq!(document.child(), Some(html.expect_node()));
        }
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

        {
            assert!(html.expect_node().borrow().expect_element().expect_normal().is_empty());

            let target = target.expect_node().borrow();
            let children = target.expect_element().expect_normal();
            assert_eq!(children.iter().map(|c| c).collect::<Vec<_>>(),
                       &[head.clone().expect_node(), body.clone().expect_node()]);
        }

        dom.reparent_children(target.clone(), html.clone());
        dom.reparent_children(document.clone(), target.clone());

        {
            assert!(document.expect_document().borrow().child().is_none());

            let target = target.expect_node().borrow();
            let children = target.expect_element().expect_normal();
            assert_eq!(children.iter().map(|c| c).collect::<Vec<_>>(),
                       &[html.clone().expect_node()]);
        }
    }
}
