
use std::cell::{Ref, RefMut, RefCell};
use std::fmt::Debug;
use std::io::{self, Write};
use std::ops::{Deref, DerefMut};
use std::rc::{Rc, Weak};

use html5ever::serialize::{Serializable, Serializer, TraversalScope};

use super::{Doctype, Document, Node};

#[derive(Clone, Debug, PartialEq)]
pub enum Handle {
    DoctypeHandle(Doctype),
    DocumentHandle(TreeHandle<Document>),
    NodeHandle(TreeHandle<Node>),
}

impl Handle {
    pub fn new_document(document: Document) -> Handle {
        Handle::DocumentHandle(TreeHandle::new(document))
    }

    pub fn new_node(node: Node) -> Handle {
        Handle::NodeHandle(TreeHandle::new(node))
    }

    pub fn expect_document(&self) -> &TreeHandle<Document> {
        if let &Handle::DocumentHandle(ref handle) = self {
            handle
        } else {
            panic!("Expected document, got {:?}.", self);
        }
    }

    pub fn expect_node(&self) -> &TreeHandle<Node> {
        if let &Handle::NodeHandle(ref handle) = self {
            handle
        } else {
            panic!("Expected node, got {:?}.", self);
        }
    }
}

impl Into<ParentHandle> for Handle {
    fn into(self) -> ParentHandle {
        match self {
            Handle::DoctypeHandle(_) => panic!("Doctype cannot be a parent."),
            Handle::DocumentHandle(ref handle) => ParentHandle::DocumentHandle(handle.downgrade()),
            Handle::NodeHandle(ref handle) => ParentHandle::NodeHandle(handle.downgrade()),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ParentHandle {
    DocumentHandle(WeakTreeHandle<Document>),
    NodeHandle(WeakTreeHandle<Node>),
}

impl ParentHandle {
    pub fn expect_document(&self) -> TreeHandle<Document> {
        if let &ParentHandle::DocumentHandle(ref handle) = self {
            handle.upgrade()
        } else {
            panic!("Expected document, got {:?}.", self);
        }
    }

    pub fn expect_node(&self) -> TreeHandle<Node> {
        if let &ParentHandle::NodeHandle(ref handle) = self {
            handle.upgrade()
        } else {
            panic!("Expected node, got {:?}.", self);
        }
    }
}

#[derive(Clone, Debug)]
pub struct TreeNode<T> {
    node: T,
    parent: Option<ParentHandle>,
}

impl<T> TreeNode<T> {
    pub fn parent(&self) -> Option<&ParentHandle> {
        self.parent.as_ref()
    }

    pub fn set_parent(&mut self, parent: ParentHandle) {
        self.parent = Some(parent);
    }

    pub fn unset_parent(&mut self) {
        self.parent = None;
    }
}

impl<T> Deref for TreeNode<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.node
    }
}

impl<T: PartialEq> PartialEq for TreeNode<T> {
    fn eq(&self, other: &TreeNode<T>) -> bool {
        self.node == other.node
    }
}

impl<T> DerefMut for TreeNode<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.node
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TreeHandle<T>(Rc<RefCell<TreeNode<T>>>);

impl<T> TreeHandle<T> {
    pub fn new(node: T) -> TreeHandle<T> {
        TreeHandle(Rc::new(RefCell::new(TreeNode {
            node: node,
            parent: None,
        })))
    }

    pub fn downgrade(&self) -> WeakTreeHandle<T> {
        WeakTreeHandle(Rc::downgrade(&self.0))
    }

    pub fn borrow(&self) -> Ref<TreeNode<T>> {
        self.0.borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<TreeNode<T>> {
        self.0.borrow_mut()
    }
}

impl TreeHandle<Document> {
    pub fn as_handle(&self) -> Handle {
        Handle::DocumentHandle(self.clone())
    }
}

impl TreeHandle<Node> {
    pub fn as_handle(&self) -> Handle {
        Handle::NodeHandle(self.clone())
    }
}

impl<T: Serializable> Serializable for TreeHandle<T> {
    fn serialize<'wr, Wr: Write>(&self,
                                 serializer: &mut Serializer<'wr, Wr>,
                                 traversal_scope: TraversalScope)
                                 -> io::Result<()> {
        self.borrow().node.serialize(serializer, traversal_scope)
    }
}

#[derive(Clone, Debug)]
pub struct WeakTreeHandle<T>(Weak<RefCell<TreeNode<T>>>);

impl<T: Debug> WeakTreeHandle<T> {
    pub fn upgrade(&self) -> TreeHandle<T> {
        self.0
            .upgrade()
            .map(TreeHandle)
            .expect(&format!("WeakTreeHandle::upgrade(self: {:?}), self leads nowhere. :(",
                             self))
    }
}

impl WeakTreeHandle<Document> {
    pub fn as_handle(&self) -> ParentHandle {
        ParentHandle::DocumentHandle(self.clone())
    }
}

impl WeakTreeHandle<Node> {
    pub fn as_handle(&self) -> ParentHandle {
        ParentHandle::NodeHandle(self.clone())
    }
}
