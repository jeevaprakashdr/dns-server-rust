#[derive(Default)]
pub(crate) struct DNSHeader {
    inner: [u8; 4],
}

impl DNSHeader {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub fn set_id(&mut self, id: u16) {
        self.inner[..2].copy_from_slice(&id.to_be_bytes())
    }

    pub(crate) fn as_slice(&self) -> &[u8] {
        self.inner.as_slice()
    }
}
