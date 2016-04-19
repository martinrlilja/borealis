
use std::marker::PhantomData;
use string_cache::QualName;

pub struct EmptyAttrs<'a>(PhantomData<&'a ()>);

impl<'a> EmptyAttrs<'a> {
    pub fn new() -> EmptyAttrs<'a> {
        EmptyAttrs(PhantomData)
    }
}

impl<'a> Iterator for EmptyAttrs<'a> {
    type Item = (&'a QualName, &'a str);

    fn next(&mut self) -> Option<(&'a QualName, &'a str)> {
        None
    }
}
