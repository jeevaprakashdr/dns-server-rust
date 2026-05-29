
#[derive(Clone, Debug)]
pub(crate) struct Message {
    pub(crate) header: Header,
    questions: Vec<Question>,
    answers: Vec<Answer>,
}

impl Message {
    pub(crate) fn new() -> Self {
        Self {
            header: Header::default(),
            questions: Vec::new(),
            answers: Vec::new(),
        }
    }

    pub(crate) fn to_vec(&mut self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&self.header.inner);
        for ele in &self.questions {
            data.extend_from_slice(&ele.to_vec());
        }

        let mut answer_section = Vec::new();
        for ele in &self.answers {
            answer_section.extend_from_slice(&ele.to_vec());
        }
        [&data[..], &answer_section[..]].concat()
    }

    pub(crate) fn set_question(&mut self, questions: Vec<Question>) {
        self.header.set_question(questions.len());

        for q in questions {
            self.questions.push(q);
        }
    }

    pub(crate) fn set_fowarder_answer(&mut self, answers: Vec<Answer>) {
        self.header.set_answer(answers.len());

        for a in answers {
            self.answers.push(a);
        }
    }

    pub(crate) fn parse(buf: &[u8]) -> Message {
        let inner = buf[..12].to_vec();
        let header = Header {
            inner: inner.try_into().expect("failed to extract"),
        };
        let buf = buf[12..].to_vec();
        let name = parse_name(buf.clone());

        let mut offset = name.len();

        let qtype_slice = &buf[offset..=offset + 1];
        let qtype_bytes: [u8; 2] = qtype_slice.try_into().unwrap();
        let qtype = QType::try_from(u16::from_be_bytes(qtype_bytes)).unwrap();
        offset += 2;

        let qclass_slice = &buf[offset..=offset + 1];
        let qclass_bytes: [u8; 2] = qclass_slice.try_into().unwrap();
        let qclass = QClass::try_from(u16::from_be_bytes(qclass_bytes)).unwrap();
        offset += 2;

        let question = Question::new(name.clone(), qtype.clone(), qclass.clone());
        offset += question.to_vec().len();

        let ttl_slice = &buf[offset..offset + 4];
        let ttl_bytes: [u8; 4] = ttl_slice.try_into().unwrap();
        let ttl = u32::from_be_bytes(ttl_bytes);
        offset += 4;

        let rdlength_slice = &buf[offset..offset + 2];
        let rdlength_bytes: [u8; 2] = rdlength_slice.try_into().unwrap();
        let rdlength = u16::from_be_bytes(rdlength_bytes);
        offset += 2;

        let rdata_slice = &buf[offset..];
        println!("{:?}", rdata_slice);
        let rdata_bytes: [u8; 4] = rdata_slice.try_into().unwrap();
        let rdata = u32::from_be_bytes(rdata_bytes);

        let answer = Answer {
            name,
            qtype,
            qclass,
            ttl,
            rdlength,
            rdata,
        };

        Message {
            header,
            questions: vec![question],
            answers: vec![answer],
        }
    }

    pub(crate) fn get_answers(&self) -> Vec<Answer> {
        self.answers.clone()
    }
}

fn parse_name(buf: Vec<u8>) -> Vec<u8> {
    let mut current_index = 0;
    let mut name = Vec::new();
    loop {
        let current = buf.get(current_index);

        if current.is_none() {
            return Vec::new();
        }

        name.push(*current.unwrap());

        if current == Some(&0x00) {
            return std::mem::take(&mut name);
        }

        current_index += 1;
    }
}

#[derive(Default, Clone, Debug)]
pub(crate) struct Question {
    name: Vec<u8>,
    qtype: QType,
    qclass: QClass,
}

impl Question {
    pub(crate) fn new(name: Vec<u8>, qtype: QType, qclass: QClass) -> Self {
        Self {
            name,
            qtype,
            qclass,
        }
    }

    pub(crate) fn to_vec(&self) -> Vec<u8> {
        let mut inner = Vec::<u8>::new();
        inner.extend_from_slice(self.name.as_slice());
        inner.extend_from_slice(&self.qtype.to_byte().to_be_bytes());
        inner.extend_from_slice(&self.qclass.to_byte().to_be_bytes());
        inner
    }

    pub(crate) fn parse(buf: &[u8]) -> Vec<Question> {
        let header_len = 12;
        let qc = parse_questions_count(buf);
        let mut count = 0;
        let buf = buf[header_len..].to_vec();
        let mut current_index = 0;

        let mut questions = Vec::<Question>::new();
        let mut name = Vec::new();
        let mut base_domain_name = Vec::<u8>::new();
        loop {
            let current = buf.get(current_index);

            if current.is_none() {
                break;
            }

            name.push(*current.unwrap());

            if current == Some(&0x00) {
                if base_domain_name.is_empty() {
                    base_domain_name = name.clone().as_slice()[..].to_vec();
                }

                let n = std::mem::take(&mut name);
                let question = Question::new(n, QType::A, QClass::IN);
                questions.push(question);
                current_index += 4;
                count += 1;
            } else if current == Some(&0xC0) {
                name = name[..name.len() - 1].to_vec();
                name.extend_from_slice(&base_domain_name[4..]);

                let n = std::mem::take(&mut name);
                let question = Question::new(n, QType::A, QClass::IN);
                questions.push(question);
                current_index += 4;
                count += 1;
            }

            if count >= qc {
                break;
            }
            current_index += 1;
        }

        questions
    }
}

fn parse_questions_count(buf: &[u8]) -> u16 {
    let header = &buf[..12];
    let qc_byte = (header[4..=5]).as_array::<2>().unwrap();
    u16::from_be_bytes(*qc_byte)
}

#[derive(Clone, Default, Debug)]
pub(crate) struct Answer {
    name: Vec<u8>,
    qtype: QType,
    qclass: QClass,
    ttl: u32,
    rdlength: u16,
    rdata: u32,
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

#[derive(Debug, Clone)]
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

impl TryFrom<u16> for QClass {
    type Error = ();

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(QClass::IN),
            _ => Err(()),
        }
    }
}

impl Default for QClass {
    fn default() -> Self {
        QClass::IN
    }
}

#[derive(Debug, Clone, PartialEq)]
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

impl TryFrom<u16> for QType {
    type Error = ();

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(QType::A),
            _ => Err(()),
        }
    }
}

impl Default for QType {
    fn default() -> Self {
        QType::A
    }
}

#[derive(Clone, Default, Debug)]
pub(crate) struct Header {
    inner: [u8; 12],
}

impl Header {
    pub(crate) fn set_id(&mut self, id: &[u8]) {
        self.inner[..2].copy_from_slice(&id)
    }

    pub(crate) fn set_qr(&mut self) {
        self.inner[2] |= 0x80;
    }

    pub(crate) fn reset_qr(&mut self) {
        self.inner[2] = 0x00;
    }

    pub(crate) fn set_rcode(&mut self) {
        self.inner[3] |= 0x04
    }

    pub(crate) fn reset_rcode(&mut self) {
        self.inner[3] = 0x00
    }

    pub(crate) fn set_opcode(&mut self, opcode: &u8) {
        self.inner[2] |= *opcode;
    }

    fn set_question(&mut self, count: usize) {
        self.inner[4..6].copy_from_slice(&(count as u16).to_be_bytes())
    }

    fn set_answer(&mut self, count: usize) {
        self.inner[6..8].copy_from_slice(&(count as u16).to_be_bytes())
    }
}

#[cfg(test)]
mod test {
    use crate::core::{Message, QType, Question};

    #[test]
    fn parse_qtype_from_bytes() {
        let bytes: [u8; 2] = [0, 1];
        let qtype = QType::try_from(u16::from_be_bytes(bytes)).unwrap();
        assert_eq!(qtype, QType::A);
    }

    #[test]
    fn parse_questions_from_request_payload() {
        let request_payload: Vec<u8> = vec![
            194, 154, 1, 0, 0, 3, 0, 0, 0, 0, 0, 0, 3, 97, 98, 99, 17, 108, 111, 110, 103, 97, 115,
            115, 100, 111, 109, 97, 105, 110, 110, 97, 109, 101, 3, 99, 111, 109, 0, 0, 1, 0, 1, 3,
            100, 101, 102, 192, 16, 0, 1, 0, 1, 3, 104, 101, 102, 192, 16, 0, 1, 0, 1, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        println!("{:?}", request_payload);
        let questions = Question::parse(request_payload.as_slice());

        assert_eq!(questions.len(), 3);
    }

    #[test]
    fn parse_bytes_into_message() {
        let buf = vec![
            197, 90, 128, 0, 0, 1, 0, 1, 0, 0, 0, 0, 3, 97, 98, 99, 12, 99, 111, 100, 101, 99, 114,
            97, 102, 116, 101, 114, 115, 2, 105, 111, 0, 0, 1, 0, 1, 3, 97, 98, 99, 12, 99, 111,
            100, 101, 99, 114, 97, 102, 116, 101, 114, 115, 2, 105, 111, 0, 0, 1, 0, 1, 0, 0, 14,
            16, 0, 4, 76, 76, 21, 21,
        ];

        let mut message = Message::parse(&buf);

        assert_eq!(message.to_vec(), buf)
    }
}
