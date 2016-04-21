
use std::io::{Error, Result, Write};

use html5ever;

use string_cache::QualName;

pub struct Serializer<'a, 'w: 'a, W: 'w + Write> {
    inner: &'a mut html5ever::serialize::Serializer<'w, W>,
    error: Option<Error>,
}

impl<'a, 'w, W: Write> Serializer<'a, 'w, W> {
    pub fn new(ser: &'a mut html5ever::serialize::Serializer<'w, W>) -> Serializer<'a, 'w, W> {
        Serializer {
            inner: ser,
            error: None,
        }
    }

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

    pub fn start_elem<'i, T>(&mut self, name: QualName, attrs: T)
        where T: Iterator<Item = (&'i QualName, &'i str)>
    {
        self.do_cond(|s| s.inner.start_elem(name.into(), attrs))
    }

    pub fn end_elem(&mut self, name: QualName) {
        self.do_cond(|s| s.inner.end_elem(name.into()));
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

    pub fn error(self) -> Result<()> {
        match self.error {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }
}
