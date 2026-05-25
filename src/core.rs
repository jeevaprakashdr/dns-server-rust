use std::net::Ipv4Addr;

use rand::RngExt;

pub(crate) struct Message {
    pub(crate) header: Header,
    question: Question,
    answer: Answer,
}

impl Message {
    pub(crate) fn new() -> Self {
        Self {
            header: Header::default(),
            question: Question::default(),
            answer: Answer::default(),
        }
    }

    pub(crate) fn to_vec(&mut self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&self.header.inner);
        data.extend_from_slice(&self.question.to_vec());
        data.extend_from_slice(&self.answer.to_vec());
        data
    }

    pub(crate) fn set_question(&mut self, name: Vec<u8>, qtype: QType, qclass: QClass) {
        self.question.name = name;
        self.question.qtype = qtype;
        self.question.qclass = qclass;
        self.header.set_question();
    }

    pub(crate) fn set_answer(&mut self, name: Vec<u8>, qtype: QType, qclass: QClass) {
        self.answer.name = name;
        self.answer.qtype = qtype;
        self.answer.qclass = qclass;
        let mut rng = rand::rng();
        self.answer.ttl = rng.random::<u32>();

        self.answer.rdata = match qtype {
            QType::A => {
                let localhost = Ipv4Addr::new(127, 0, 0, 1);
                localhost.to_bits()
            }
        };

        self.answer.rdlength = match qtype {
            QType::A => 4,
        };

        self.header.set_answer()
    }
}

#[derive(Default)]
pub(crate) struct Question {
    name: Vec<u8>,
    qtype: QType,
    qclass: QClass,
}

impl Question {
    fn to_vec(&self) -> Vec<u8> {
        let mut inner = Vec::<u8>::new();
        inner.extend_from_slice(self.name.as_slice());
        inner.extend_from_slice(&self.qtype.to_byte().to_be_bytes());
        inner.extend_from_slice(&self.qclass.to_byte().to_be_bytes());
        inner
    }
}

#[derive(Default)]
pub(crate) struct Answer {
    name: Vec<u8>,
    qtype: QType,
    qclass: QClass,
    ttl: u32,
    rdata: u32,
    rdlength: u16,
}

impl Answer {
    fn to_vec(&self) -> Vec<u8> {
        let mut inner = Vec::<u8>::new();
        inner.extend_from_slice(self.name.as_slice());
        inner.extend_from_slice(&self.qtype.to_byte().to_be_bytes());
        inner.extend_from_slice(&self.qclass.to_byte().to_be_bytes());
        inner.extend_from_slice(&self.ttl.to_be_bytes());
        inner.extend_from_slice(&self.rdlength.to_be_bytes());
        inner.extend_from_slice(&self.rdata.to_be_bytes());
        inner
    }
}

pub(crate) enum QClass {
    IN,
}

impl QClass {
    pub(crate) fn to_byte(&self) -> u16 {
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
    pub(crate) fn to_byte(&self) -> u16 {
        match self {
            QType::A => 1 as u16,
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
    pub(crate) fn set_id(&mut self, id: &[u8]) {
        self.inner[..2].copy_from_slice(&id)
    }

    pub(crate) fn set_qr(&mut self) {
        self.inner[2] |= 0x80
    }

    pub(crate) fn set_rcode(&mut self) {
        self.inner[3] |= 0x04
    }

    pub(crate) fn set_opcode(&mut self, opcode: &u8) {
        self.inner[2] |= *opcode;
    }

    fn set_question(&mut self) {
        let count: u16 = 1;
        self.inner[4..6].copy_from_slice(&count.to_be_bytes())
    }

    fn set_answer(&mut self) {
        let count: u16 = 1;
        self.inner[6..8].copy_from_slice(&count.to_be_bytes())
    }
}
