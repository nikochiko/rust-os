pub struct ByteArrayWriter {
    buf: [u8; 1024],
    index: usize,
}

impl ByteArrayWriter {
    pub fn new(buf: [u8; 1024]) -> Self {
        ByteArrayWriter {
            buf,
            index: 0,
        }
    }

    pub fn bytes(&self) -> &[u8] {
        &self.buf[0..self.index]
    }

    pub fn as_string(&self) -> &str {
        core::str::from_utf8(self.bytes()).unwrap()
    }
}

impl core::fmt::Write for ByteArrayWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for &byte in s.as_bytes() {
            self.buf[self.index] = byte;
            self.index += 1;
        }
        Ok(())
    }
}
