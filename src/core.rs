#[derive(Default)]
pub(crate) struct Message {
    inner: [u8; 12],
    pub(crate) header: Header,
}

impl Message {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn as_slice(&mut self) -> &[u8] {
        self.inner[..4].copy_from_slice(&self.header.inner);
        self.inner.as_slice()
    }
}

#[derive(Default)]
pub(crate) struct Header {
    inner: [u8; 4],
}

impl Header {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn set_id(&mut self, id: u16) {
        self.inner[..2].copy_from_slice(&id.to_be_bytes())
    }

    pub(crate) fn set_qr(&mut self, qr: bool) {
        if qr {
            self.inner[2] |= 1 << 7
        } else {
            self.inner[2] &= !(1 << 7)
        }
    }
}
