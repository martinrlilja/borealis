
use std::io::Write;

use super::Serializer;

use string_cache::QualName;

pub struct NodeSerializer<'a, 'b: 'a, 'w: 'b, W: 'w + Write> {
    name: QualName,
    serializer: &'a mut Serializer<'b, 'w, W>,
}

impl<'a, 'b: 'a, 'c: 'b, 'd: 'c, 'w: 'd, W: Write> NodeSerializer<'c, 'd, 'w, W> {
    pub fn text(&mut self, text: &str) {
        self.serializer.write_text(text);
    }

    pub fn comment(&mut self, comment: &str) {
        self.serializer.write_comment(comment);
    }

    pub fn element_normal<'i, I>(&'a mut self,
                                 name: &QualName,
                                 attrs: I)
                                 -> NodeSerializer<'a, 'd, 'w, W>
        where I: IntoIterator<Item = (&'i QualName, &'i str)>
    {
        element_normal::<'a, 'd, 'i, 'w, I, W>(self.serializer, name, attrs)
    }
}

impl<'a, 'b, 'w, W: Write> Drop for NodeSerializer<'a, 'b, 'w, W> {
    fn drop(&mut self) {
        self.serializer.end_elem(&self.name);
    }
}

pub fn element_normal<'a, 'b: 'a, 'i, 'w: 'b, I, W>(parent: &'a mut Serializer<'b, 'w, W>,
                                                    name: &QualName,
                                                    attrs: I)
                                                    -> NodeSerializer<'a, 'b, 'w, W>
    where I: IntoIterator<Item = (&'i QualName, &'i str)>,
          W: 'w + Write
{
    parent.start_elem(name, attrs.into_iter());
    NodeSerializer {
        name: name.clone(),
        serializer: parent,
    }
}
