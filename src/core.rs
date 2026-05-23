pub(crate) struct Message<'a> {
    inner: [u8; 10],
    pub(crate) header: Header,
    pub(crate) question: Question<'a>,
}

impl<'a> Message<'a> {
    pub(crate) fn new() -> Self {
        Self {
            inner: [0u8; 10],
            header: Header::default(),
            question: Question::default(),
        }
    }

    pub(crate) fn to_vec(&mut self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&self.header.inner);
        data.extend_from_slice(&self.question.as_bytes());
        data
    }

    pub(crate) fn set_question(&mut self, name: &'a str, qtype: QType, qclass: QClass) {
        self.question.name = name;
        self.question.qtype = qtype;
        self.question.qclass = qclass;
        self.header.set_question_no(self.question.as_bytes().len() as u16);
    }
}

#[derive(Default)]
pub(crate) struct Question<'a> {
    name: &'a str,
    qtype: QType,
    qclass: QClass,
}

impl<'a> Question<'a> {
    pub(crate) fn new(name: &'a str, qtype: QType, qclass: QClass) -> Self {
        Self {
            name,
            qtype,
            qclass,
        }
    }

    fn as_bytes(&self) -> Vec<u8> {
        let mut inner = Vec::<u8>::new();
        inner.extend_from_slice(self.encode_name().as_slice());
        inner.extend_from_slice(b"0".as_slice());
        inner.extend_from_slice(&self.qtype.to_byte().to_be_bytes());
        inner.extend_from_slice(&self.qclass.to_byte().to_be_bytes());
        inner
    }

    fn encode_name(&self) -> Vec<u8> {
        let domain = self.name.split(".").collect::<Vec<_>>();
        let base = domain.first().unwrap();
        let tld = domain.last().unwrap();
        let encoded_string = format!("{}{base}{}{tld}", base.len(), tld.len());
        encoded_string.as_bytes().to_vec()
    }
}

pub(crate) enum QClass {
    IN,
}

impl QClass {
    pub(crate) fn to_byte(&self) -> u8 {
        match self {
            QClass::IN => 1,
        }
    }
}

impl Default for QClass {
    fn default() -> Self {
        QClass::IN
    }
}

pub(crate) enum QType {
    A,
}

impl QType {
    pub(crate) fn to_byte(&self) -> u8 {
        match self {
            QType::A => 1,
        }
    }
}

impl Default for QType {
    fn default() -> Self {
        QType::A
    }
}

#[derive(Default)]
pub(crate) struct Header {
    inner: [u8; 12],
}

impl Header {
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

    fn set_question_no(&mut self, count: u16) {
        self.inner[4..6].copy_from_slice(&count.to_be_bytes());
    }
}
