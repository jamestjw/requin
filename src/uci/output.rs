use std::io::{self, Write};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Output<W>(Arc<Mutex<W>>);

impl<W: Write> Output<W> {
    pub fn get_inner(&self) -> Arc<Mutex<W>> {
        self.0.clone()
    }
}

impl<W: Write> Output<W> {
    pub fn new(w: W) -> Self {
        Output(Arc::new(Mutex::new(w)))
    }
}

impl<W: Write> Write for Output<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        (*self.0.lock().unwrap()).write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        (*self.0.lock().unwrap()).flush()
    }
}
