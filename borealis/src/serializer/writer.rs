
use std::cell::RefCell;
use std::fmt::Arguments;
use std::io::{Result, Write};
use std::rc::Rc;

pub struct Writer<'a, W: 'a + Write>(Rc<RefCell<&'a mut W>>);

impl<'a, W: 'a + Write> Writer<'a, W> {
    pub fn new(w: &'a mut W) -> Writer<'a, W> {
        Writer(Rc::new(RefCell::new(w)))
    }
}

impl<'a, W: 'a + Write> Write for Writer<'a, W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.0.borrow_mut().write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        self.0.borrow_mut().flush()
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        self.0.borrow_mut().write_all(buf)
    }

    fn write_fmt(&mut self, fmt: Arguments) -> Result<()> {
        self.0.borrow_mut().write_fmt(fmt)
    }
}

impl<'a, W: 'a + Write> Clone for Writer<'a, W> {
    fn clone(&self) -> Writer<'a, W> {
        Writer(self.0.clone())
    }
}
