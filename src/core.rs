use std::net::Ipv4Addr;

pub(crate) struct Message {
    pub(crate) header: Header,
    question: Vec<Question>,
    answer: Vec<Answer>,
}

impl Message {
    pub(crate) fn new() -> Self {
        Self {
            header: Header::default(),
            question: Vec::new(),
            answer: Vec::new(),
        }
    }

    pub(crate) fn to_vec(&mut self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&self.header.inner);
        for ele in &self.question {
            data.extend_from_slice(&ele.to_vec());
        }

        let mut answer_section = Vec::new();
        for ele in &self.answer {
            answer_section.extend_from_slice(&ele.to_vec());
        }
        [&data[..], &answer_section[..]].concat()
    }

    pub(crate) fn set_question(&mut self, questions: Vec<Question>) {
        self.header.set_question(questions.len());

        for q in questions {
            self.question.push(q);
        }
    }

    pub(crate) fn set_answer(&mut self, answers: Vec<Answer>) {
        self.header.set_answer(answers.len());

        for a in answers {
            self.answer.push(a);
        }
    }
}

#[derive(Default, Clone, Debug)]
pub(crate) struct Question {
    pub(crate) name: Vec<u8>,
    pub(crate) qtype: QType,
    pub(crate) qclass: QClass,
}

impl Question {
    pub(crate) fn new(name: Vec<u8>, qtype: QType, qclass: QClass) -> Self {
        Self {
            name,
            qtype,
            qclass,
        }
    }

    fn to_vec(&self) -> Vec<u8> {
        let mut inner = Vec::<u8>::new();
        inner.extend_from_slice(self.name.as_slice());
        inner.extend_from_slice(&self.qtype.to_byte().to_be_bytes());
        inner.extend_from_slice(&self.qclass.to_byte().to_be_bytes());
        inner
    }

    pub(crate) fn parse_new(buf: &[u8]) -> Vec<Question> {
        let header_len = 12;
        let qc = parse_questions_count_new(buf);
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

            // println!("currentindex {} {}", current_index, *current.unwrap());
            name.push(*current.unwrap());

            if current == Some(&0x00) {
                println!("name {:?}", String::from_utf8(name.to_vec()).unwrap());
                if base_domain_name.is_empty() {
                    base_domain_name = name.clone().as_slice()[..].to_vec();
                }

                // println!(
                //     "base_domain_name {:?}",
                //     String::from_utf8(base_domain_name.to_vec()).unwrap()
                // );
                let n = std::mem::take(&mut name);
                let question = Question::new(n, QType::A, QClass::IN);
                questions.push(question);
                current_index += 4;
                count += 1;
            } else if current == Some(&0xC0) {
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
        // println!("{:?}", questions);
        questions
    }
}

fn parse_questions_count_new(buf: &[u8]) -> u16 {
    let header = &buf[..12];
    let qc_byte = (header[4..=5]).as_array::<2>().unwrap();
    let count = u16::from_be_bytes(*qc_byte);

    count
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
    pub(crate) fn new(name: Vec<u8>, qtype: QType, qclass: QClass) -> Self {
        let ttl = 60;
        let rdata = match qtype {
            QType::A => {
                let localhost = Ipv4Addr::new(127, 0, 0, 1);
                localhost.to_bits()
            }
        };

        let rdlength = match qtype {
            QType::A => 4,
        };

        Self {
            name,
            qtype,
            qclass,
            ttl,
            rdata,
            rdlength,
        }
    }

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

#[derive(Debug, Clone)]
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

#[derive(Default)]
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

    pub(crate) fn set_rcode(&mut self) {
        self.inner[3] |= 0x04
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
    use crate::core::Question;

    #[test]
    fn parse_questions_from_request_payload() {
        let request_payload: Vec<u8> = vec![
            194, 154, 1, 0, 0, 3, 0, 0, 0, 0, 0, 0, 3, 97, 98, 99, 17, 108, 111, 110, 103, 97, 115,
            115, 100, 111, 109, 97, 105, 110, 110, 97, 109, 101, 3, 99, 111, 109, 0, 0, 1, 0, 1, 3,
            100, 101, 102, 192, 16, 0, 1, 0, 1, 3, 104, 101, 102, 192, 16, 0, 1, 0, 1, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];

        let questions = Question::parse_new(request_payload.as_slice());

        assert_eq!(questions.len(), 3);
    }
}
