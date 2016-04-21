
use std::marker::PhantomData;

use super::node::Attr;

pub struct EmptyAttrs<'a>(PhantomData<&'a ()>);

impl<'a> EmptyAttrs<'a> {
    pub fn new() -> EmptyAttrs<'a> {
        EmptyAttrs(PhantomData)
    }
}

impl<'a> Iterator for EmptyAttrs<'a> {
    type Item = Attr<'a>;

    fn next(&mut self) -> Option<Attr<'a>> {
        None
    }
}
