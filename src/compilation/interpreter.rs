use std::io;

pub struct InterpreterContext {
    writer: Box<dyn io::Write>,
    reader: Box<dyn io::Read>,
}

impl std::default::Default for InterpreterContext {
    fn default() -> Self {
        let stdin = io::stdin();
        let stdout = io::stdout();
        Self {
            reader: Box::new(stdin),
            writer: Box::new(stdout),
        }
    }
}

impl io::Write for InterpreterContext {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.writer.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

impl io::Read for InterpreterContext {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.reader.read(buf)
    }
}
