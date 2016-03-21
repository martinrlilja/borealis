
use std::borrow::Cow;
use std::collections::HashSet;

use html5ever;
use html5ever::tree_builder::{TreeSink, QuirksMode, NodeOrText};
use html5ever::tendril::StrTendril;

use string_cache::QualName;

use html::{Attribute, CommentNode, Doctype, Document, ElementNode, ElementType, Handle, Node, ParentHandle, TextNode};

#[derive(Debug)]
pub struct Dom {
    document:    Handle,
    errors:      Vec<Cow<'static, str>>,
    quirks_mode: QuirksMode,
}

impl Dom {
    pub fn new() -> Dom {
        Dom {
            document:    Handle::new_document(Document::new(None, None)),
            errors:      Vec::new(),
            quirks_mode: QuirksMode::NoQuirks
        }
    }

    fn node_or_text_as_handle(child: &NodeOrText<Handle>) -> Handle {
        match child {
            &NodeOrText::AppendText(ref text) => {
                Handle::new_node(Node::Text(TextNode::new(text.clone())))
            },
            &NodeOrText::AppendNode(ref node) => node.clone(),
        }
    }

    fn remove_child_from_parent(parent: &ParentHandle, child: &Handle) {
        match *parent {
            ParentHandle::DocumentHandle(ref document) => {
                let document = document.upgrade();
                assert_eq!(document.borrow().child(), Some(child.expect_node()));

                document.borrow_mut().unset_child();
            },
            ParentHandle::NodeHandle(ref node) => {
                let node         = node.upgrade();
                let mut node     = node.borrow_mut();
                let mut children = node.expect_element_mut().expect_normal_mut();
                let index        = children.iter().position(|e| e.clone().as_handle() == *child)
                    .expect(&format!("Dom::remove_child_from_parent(parent: {:?}, child: {:?}), child is child of parent.",
                    parent, child));

                children.remove(index);
            },
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
            .as_handle()
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
            _ => ElementType::new_normal()
        };

        Handle::new_node(Node::Element(ElementNode::new(
            name, element_type,
            attributes.iter().map(|a| Attribute::from(a.clone()))
                .collect()
        )))
    }

    fn create_comment(&mut self, text: StrTendril) -> Handle {
        Handle::new_node(Node::Comment(CommentNode::new(text)))
    }

    fn append(&mut self, parent: Handle, child: NodeOrText<Handle>) {
        let child = Dom::node_or_text_as_handle(&child);

        match parent {
            Handle::DoctypeHandle(_) => panic!("Cannot append to doctype."),
            Handle::DocumentHandle(ref document) => {
                document.borrow_mut()
                    .set_child(child.expect_node().clone());
            },
            Handle::NodeHandle(ref node) => {
                let child = child.expect_node();
                child.borrow_mut()
                    .set_parent(parent.clone().into());
                node.borrow_mut()
                    .expect_element_mut()
                    .expect_normal_mut()
                    .push(child.clone());
            },
        }
    }

    fn append_before_sibling(&mut self, sibling: Handle, child: NodeOrText<Handle>)
        -> Result<(), NodeOrText<Handle>>
    {
        let node         = Dom::node_or_text_as_handle(&child);

        let parent       = try!(sibling.expect_node().borrow().parent().ok_or(child)).expect_node();
        let child        = node.expect_node();
        let mut children = parent.borrow_mut();
        let mut children = children.expect_element_mut().expect_normal_mut();
        let index        = children.iter()
            .position(|e| e.clone().as_handle() == sibling)
            .expect(&format!("Dom::append_before_sibling(sibling: {:?}, child: {:?}), before is not child of self.",
            sibling, child));

        child.borrow_mut()
            .set_parent(parent.downgrade().as_handle());
        children.insert(index, child.clone());

        Ok(())
    }

    fn append_doctype_to_document(&mut self, name: StrTendril, public_id: StrTendril, system_id: StrTendril) {
        self.document.expect_document().borrow_mut().set_doctype(Doctype::new(
            name, public_id, system_id
        ));
    }

    fn add_attrs_if_missing(&mut self, target: Handle, attrs: Vec<html5ever::Attribute>) {
        let mut target     = target.expect_node().borrow_mut();
        let mut attributes = target.expect_element_mut().attributes_mut();

        let names   = attributes.iter().map(|a| a.name().clone()).collect::<HashSet<_>>();
        let missing = attrs.into_iter().filter(|a| !names.contains(&a.name));
        attributes.extend(missing.map(|a| Attribute::from(a.clone())));
    }

    fn remove_from_parent(&mut self, target: Handle) {
        match target {
            Handle::DoctypeHandle(_) => panic!("Cannot remove doctype from parent."),
            Handle::DocumentHandle(ref child) => {
                let document = child.borrow();
                let parent   = document.parent()
                    .expect(&format!("Dom::remove_from_parent(target: {:?}), target must have a parent.", target));

                Dom::remove_child_from_parent(parent, &target);
                child.borrow_mut().unset_parent();
            },
            Handle::NodeHandle(ref child) => {
                let node   = child.borrow();
                let parent = node.parent()
                    .expect(&format!("Dom::remove_from_parent(target: {:?}), target must have a parent.", target));

                Dom::remove_child_from_parent(parent, &target);
                child.borrow_mut().unset_parent();
            },
        }
    }

    fn reparent_children(&mut self, old_parent: Handle, new_parent: Handle) {
        match old_parent {
            Handle::DoctypeHandle(_) => (),
            Handle::DocumentHandle(ref parent) => {
                if let Some(child) = parent.borrow().child() {
                    let child = NodeOrText::AppendNode(child.clone().as_handle());
                    self.append(new_parent, child);
                    parent.borrow_mut().unset_child();
                }
            },
            Handle::NodeHandle(ref parent) => {
                let mut parent   = parent.borrow_mut();
                let mut children = parent.expect_element_mut().expect_normal_mut();

                for child in children.iter() {
                    let child = NodeOrText::AppendNode(child.clone().as_handle());
                    self.append(new_parent.clone(), child);
                }

                children.clear();
            },
        }
    }

    fn mark_script_already_started(&mut self, _: Handle) {}
}

#[cfg(test)]
mod tests {
    use super::Dom;
    use html5ever::driver::{parse_document, parse_fragment, Parser, ParseOpts};
    use html5ever::tendril::{TendrilSink};

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

    const FRAGMENT: &'static str =
        "<p>
             Fragment
             <img src=\"test.flif\" alt=\"test\">
         </p>";

    #[test]
    fn test_parse_document() {
        let parser = parse_document(Dom::default(), ParseOpts::default())
            .from_utf8();

        let output = parser.one(DOCUMENT.as_bytes());
    }

    #[test]
    fn test_parse_fragment() {
        let parser = parse_fragment(Dom::default(), ParseOpts::default(), qualname!(html, "div"), Vec::new())
            .from_utf8();

        let output = parser.one(FRAGMENT.as_bytes());
    }
}
