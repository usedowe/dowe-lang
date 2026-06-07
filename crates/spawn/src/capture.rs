#[derive(Clone, Debug, Default)]
pub struct CaptureState {
    pub stdout: CaptureBuffer,
    pub stderr: CaptureBuffer,
    pub terminal: CaptureBuffer,
}

#[derive(Clone, Debug, Default)]
pub struct CaptureBuffer {
    pub bytes: Vec<u8>,
    pub truncated: bool,
}

impl CaptureBuffer {
    pub fn append(&mut self, bytes: &[u8], max_bytes: Option<usize>) {
        if self.truncated {
            return;
        }

        let Some(max_bytes) = max_bytes else {
            self.bytes.extend_from_slice(bytes);
            return;
        };

        if self.bytes.len() + bytes.len() <= max_bytes {
            self.bytes.extend_from_slice(bytes);
            return;
        }

        let remaining = max_bytes.saturating_sub(self.bytes.len());
        self.bytes.extend_from_slice(&bytes[..remaining]);
        self.truncated = true;
    }
}
