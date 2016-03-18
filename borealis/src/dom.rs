
use std::ascii::AsciiExt;
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashSet;
use std::io::{self, Write};
use std::ops::Deref;
use std::rc::{Rc, Weak};

use html5ever::Attribute;
use html5ever::tree_builder::{TreeSink, QuirksMode, NodeOrText};
use html5ever::tendril::StrTendril;
use html5ever::serialize::{Serializable, Serializer, TraversalScope};

use string_cache::QualName;

#[derive(Debug)]
pub enum Html {
    Node(Node),
    Nodes(Vec<Node>),
}

#[derive(Debug, PartialEq)]
pub enum ElementType {
    Normal,
    Template(Handle),
    Script { already_started: bool },
}

#[derive(Debug, PartialEq)]
pub enum NodeType {
    Document(Vec<Handle>),
    Doctype { name: StrTendril, public_id: StrTendril, system_id: StrTendril },
    Text(StrTendril),
    Comment(StrTendril),
    Element {
        name:         QualName,
        element_type: ElementType,
        attributes:   Vec<Attribute>,
        children:     Vec<Handle>,
    },
}

#[derive(Debug)]
pub struct Node {
    node_type: NodeType,
    parent:    Option<Weak<RefCell<Node>>>,
}

impl Node {
    pub fn new(node_type: NodeType) -> Node {
        Node {
            node_type: node_type,
            parent:    None,
        }
    }

    fn node_type(&self) -> &NodeType {
        &self.node_type
    }

    fn parent(&self) -> Option<Handle> {
        match self.parent {
            Some(ref parent) => {
                parent.upgrade().map(Handle)
            },
            None => None
        }
    }

    fn set_parent(&mut self, parent: &Handle) {
        self.parent = Some(Rc::downgrade(parent));
    }

    fn unset_parent(&mut self) {
        self.parent = None;
    }

    fn children(&self) -> &Vec<Handle> {
        if let NodeType::Element { ref children, .. } = self.node_type {
            children
        } else if let NodeType::Document(ref children) = self.node_type {
            children
        } else {
            panic!("Node::children(self: {:?}), self cannot have children.", self);
        }
    }

    fn children_mut(&mut self) -> &mut Vec<Handle> {
        if let NodeType::Element { ref mut children, .. } = self.node_type {
            children
        } else if let NodeType::Document(ref mut children) = self.node_type {
            children
        } else {
            panic!("Node::children_mut(self: {:?}), self cannot have children.", self);
        }
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Node) -> bool {
        self.node_type == other.node_type
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Handle(Rc<RefCell<Node>>);

impl Handle {
    fn new(node_type: NodeType) -> Handle {
        Handle(Rc::new(RefCell::new(Node::new(node_type))))
    }

    pub fn append(&self, child: Handle) {
        let mut node     = self.borrow_mut();
        let mut children = node.children_mut();

        child.borrow_mut().set_parent(self);
        children.push(child);
    }

    pub fn append_before(&self, before: Handle, child: Handle) {
        let mut node     = self.borrow_mut();
        let mut children = node.children_mut();

        let index = children.iter().position(|e| *e == before)
            .expect(&format!("Node::append_before(self: {:?}, before: {:?}, child: {:?}), before is not child of self.",
            self, before, child));

        child.borrow_mut().set_parent(self);
        children.insert(index, child);
    }

    pub fn remove_child(&self, child: Handle) {
        let mut node     = self.borrow_mut();
        let mut children = node.children_mut();

        let index = children.iter().position(|e| *e == child)
            .expect(&format!("Node::remove:child(self: {:?}, child: {:?}), child is not child of parent.", self, child));

        children.remove(index);
        child.borrow_mut().unset_parent();
    }
}

impl Deref for Handle {
    type Target = Rc<RefCell<Node>>;

    fn deref(&self) -> &Rc<RefCell<Node>> {
        &self.0
    }
}

impl Serializable for Handle {
    fn serialize<'wr, Wr: Write>(&self, serializer: &mut Serializer<'wr, Wr>,
                                 traversal_scope: TraversalScope) -> io::Result<()>
    {
        let node = self.borrow();
        match (traversal_scope, node.node_type()) {
            (_, &NodeType::Element { ref name, ref attributes, ref children, .. }) => {
                if traversal_scope == TraversalScope::IncludeNode {
                    try!(serializer.start_elem(name.clone(),
                        attributes.iter().map(|a| (&a.name, &a.value[..]))));
                }

                for handle in children.iter() {
                    try!(handle.serialize(serializer, TraversalScope::IncludeNode));
                }

                if traversal_scope == TraversalScope::IncludeNode {
                    try!(serializer.end_elem(name.clone()));
                }

                Ok(())
            },
            (TraversalScope::ChildrenOnly, &NodeType::Document(ref children)) => {
                for handle in children.iter() {
                    try!(handle.serialize(serializer, TraversalScope::IncludeNode));
                }

                Ok(())
            },
            (TraversalScope::ChildrenOnly, _) => Ok(()),
            (TraversalScope::IncludeNode, &NodeType::Doctype { ref name, .. }) => {
                serializer.write_doctype(&name)
            },
            (TraversalScope::IncludeNode, &NodeType::Text(ref text)) => {
                serializer.write_text(&text)
            },
            (TraversalScope::IncludeNode, &NodeType::Comment(ref comment)) => {
                serializer.write_comment(&comment)
            },
            (TraversalScope::IncludeNode, &NodeType::Document(_)) => {
                panic!("Handle::serialize(self: {:?}), cannot serialize document.", self);
            }
        }
    }
}

#[derive(Debug)]
pub struct Dom {
    document:    Handle,
    errors:      Vec<Cow<'static, str>>,
    quirks_mode: QuirksMode,
}

impl Dom {
    pub fn new() -> Dom {
        Dom {
            document:    Handle::new(NodeType::Document(Vec::new())),
            errors:      Vec::new(),
            quirks_mode: QuirksMode::NoQuirks
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
        if let NodeType::Element { element_type: ElementType::Template(ref contents), .. } = target.borrow().node_type {
            contents.clone()
        } else {
            panic!("Dom::get_template_contents(target: {:?}), target is not a template.", target);
        }
    }

    fn set_quirks_mode(&mut self, quirks_mode: QuirksMode) {
        self.quirks_mode = quirks_mode;
    }

    fn same_node(&self, a: Handle, b: Handle) -> bool {
        a == b
    }

    fn elem_name(&self, target: Handle) -> QualName {
        if let NodeType::Element { ref name, .. } = target.borrow().node_type {
            name.clone()
        } else {
            panic!("Dom::elem_name(target: {:?}), target is not an element.", target);
        }
    }

    fn create_element(&mut self, name: QualName, attributes: Vec<Attribute>) -> Handle {
        let element_type = match name {
            qualname!(html, "script")   => ElementType::Script { already_started: false },
            qualname!(html, "template") => ElementType::Template(Handle::new(NodeType::Document(Vec::new()))),
            _ => ElementType::Normal
        };

        Handle::new(NodeType::Element {
            name:         name,
            element_type: element_type,
            attributes:   attributes,
            children:     Vec::new(),
        })
    }

    fn create_comment(&mut self, text: StrTendril) -> Handle {
        Handle::new(NodeType::Comment(text))
    }

    fn append(&mut self, parent: Handle, child: NodeOrText<Handle>) {
        parent.append(match child {
            NodeOrText::AppendText(text) => Handle::new(NodeType::Text(text)),
            NodeOrText::AppendNode(node) => node,
        });
    }

    fn append_before_sibling(&mut self, sibling: Handle, child: NodeOrText<Handle>)
        -> Result<(), NodeOrText<Handle>>
    {
        let node = match child {
            NodeOrText::AppendText(ref text) => Handle::new(NodeType::Text(text.clone())),
            NodeOrText::AppendNode(ref node) => node.clone(),
        };

        let parent = try!(sibling.borrow().parent().ok_or(child));
        parent.append_before(sibling, node);
        Ok(())
    }

    fn append_doctype_to_document(&mut self, name: StrTendril, public_id: StrTendril, system_id: StrTendril) {
        self.document.append(Handle::new(NodeType::Doctype {
            name:      name,
            public_id: public_id,
            system_id: system_id,
        }));
    }

    fn add_attrs_if_missing(&mut self, target: Handle, attrs: Vec<Attribute>) {
        if let NodeType::Element { ref mut attributes, .. } = target.borrow_mut().node_type {
            let names   = attributes.iter().map(|a| a.name.clone()).collect::<HashSet<_>>();
            let missing = attrs.into_iter().filter(|a| !names.contains(&a.name));
            attributes.extend(missing);
        } else {
            panic!("Dom::add_attrs_if_missing(target: {:?}, attrs: {:?}), target is not an element.",
                target, attrs);
        }
    }

    fn remove_from_parent(&mut self, target: Handle) {
        let parent = target.borrow().parent()
            .expect(&format!("Dom::remove_from_parent(target: {:?}), target has no parent.", target));
        parent.remove_child(target);
    }

    fn reparent_children(&mut self, old_parent: Handle, new_parent: Handle) {
        if let NodeType::Element { ref mut children, .. } = old_parent.borrow_mut().node_type {
            for child in children.iter() {
                new_parent.append(child.clone());
            }

            children.clear();
        } else {
            panic!("Dom::reparent_children(old_parent: {:?}, new_parent: {:?}), old_parent is not an element.",
                old_parent, new_parent);
        }
    }

    fn mark_script_already_started(&mut self, node: Handle) {
        if let NodeType::Element {
                element_type: ElementType::Script {
                    ref mut already_started, ..
                }, ..
            } = node.borrow_mut().node_type
        {
            *already_started = true;
        } else {
            panic!("Dom::mark_script_already_started(node: {:?}), node is not a script.", node);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Dom, Handle};
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
        panic!("{:?}", output);
    }

    #[test]
    fn test_parse_fragment() {
        let parser = parse_fragment(Dom::default(), ParseOpts::default(), qualname!(html, "div"), Vec::new())
            .from_utf8();

        let output = parser.one(FRAGMENT.as_bytes());
        panic!("{:?}", output);
    }
}
