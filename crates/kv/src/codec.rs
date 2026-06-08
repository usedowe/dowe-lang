use crate::error::{KvError, KvResult};

pub struct Writer {
    bytes: Vec<u8>,
}

pub struct Reader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl Writer {
    pub fn new() -> Self {
        Self { bytes: Vec::new() }
    }

    pub fn bytes(&mut self, value: &[u8]) {
        self.bytes.extend_from_slice(value);
    }

    pub fn u32(&mut self, value: u32) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub fn string(&mut self, value: &str) {
        self.raw(value.as_bytes());
    }

    pub fn raw(&mut self, value: &[u8]) {
        self.u32(value.len() as u32);
        self.bytes.extend_from_slice(value);
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}

impl<'a> Reader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    pub fn magic(&mut self, magic: &[u8]) -> KvResult<()> {
        if self.bytes.len() < self.offset + magic.len()
            || &self.bytes[self.offset..self.offset + magic.len()] != magic
        {
            return Err(KvError::Corruption("invalid KV file header".to_string()));
        }
        self.offset += magic.len();
        Ok(())
    }

    pub fn u32(&mut self) -> KvResult<u32> {
        if self.bytes.len() < self.offset + 4 {
            return Err(KvError::Corruption("unexpected end of KV file".to_string()));
        }
        let mut value = [0u8; 4];
        value.copy_from_slice(&self.bytes[self.offset..self.offset + 4]);
        self.offset += 4;
        Ok(u32::from_le_bytes(value))
    }

    pub fn string(&mut self) -> KvResult<String> {
        String::from_utf8(self.raw()?)
            .map_err(|_| KvError::Corruption("KV file contains invalid UTF-8".to_string()))
    }

    pub fn raw(&mut self) -> KvResult<Vec<u8>> {
        let len = self.u32()? as usize;
        if self.bytes.len() < self.offset + len {
            return Err(KvError::Corruption("unexpected end of KV file".to_string()));
        }
        let output = self.bytes[self.offset..self.offset + len].to_vec();
        self.offset += len;
        Ok(output)
    }

    pub fn is_done(&self) -> bool {
        self.offset == self.bytes.len()
    }
}
