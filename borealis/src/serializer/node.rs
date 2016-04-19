
use std::convert::From;
use std::io::Write;

use super::serializer::Serializer;

use string_cache::QualName;

pub struct Attr<'a>(&'a QualName, &'a str);

impl<'a> From<(&'a QualName, &'a str)> for Attr<'a> {
    fn from((key, value): (&'a QualName, &'a str)) -> Attr {
        Attr(key, value)
    }
}

impl<'a, 'b: 'a> From<&'a (&'b QualName, &'b str)> for Attr<'b> {
    fn from(&(key, value): &'a (&'b QualName, &'b str)) -> Attr<'b> {
        Attr(key, value)
    }
}

pub struct NodeSerializer<'a, 'b: 'a, 'w: 'b, W: 'w + Write> {
    name: Option<QualName>,
    serializer: &'a mut Serializer<'b, 'w, W>,
}

impl<'a, 'b: 'a, 'c: 'b, 'd: 'c, 'w: 'd, W: Write> NodeSerializer<'c, 'd, 'w, W> {
    pub fn text(&mut self, text: &str) {
        self.serializer.write_text(text);
    }

    pub fn comment(&mut self, comment: &str) {
        self.serializer.write_comment(comment);
    }

    pub fn element_normal<'i, I, II>(&'a mut self,
                                     name: QualName,
                                     attrs: I)
                                     -> NodeSerializer<'a, 'd, 'w, W>
        where I: Iterator<Item = II>,
              II: Into<Attr<'i>>
    {
        self.serializer.start_elem(name.clone(),
                                   attrs.into_iter().map(|a| {
                                       let a = a.into();
                                       (a.0, a.1)
                                   }));
        NodeSerializer {
            name: Some(name),
            serializer: self.serializer,
        }
    }
}

impl<'a, 'b, 'w, W: Write> Drop for NodeSerializer<'a, 'b, 'w, W> {
    fn drop(&mut self) {
        match self.name {
            Some(ref name) => self.serializer.end_elem(name.clone()),
            None => (),
        }
    }
}

pub fn new_node_ser<'a, 'b: 'a, 'w: 'b, W>(s: &'a mut Serializer<'b, 'w, W>)
                                           -> NodeSerializer<'a, 'b, 'w, W>
    where W: 'w + Write
{
    NodeSerializer {
        name: None,
        serializer: s,
    }
}
