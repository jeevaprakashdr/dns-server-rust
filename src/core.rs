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

        for ele in &self.answer {
            data.extend_from_slice(&ele.to_vec());
        }

        data
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

#[derive(Default, Clone)]
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
        inner.push(0 as u8);
        inner.extend_from_slice(&self.qtype.to_byte().to_be_bytes());
        inner.extend_from_slice(&self.qclass.to_byte().to_be_bytes());
        inner
    }

    pub(crate) fn parse(buf: [u8; 512]) -> Vec<Question> {
        let qc = parse_questions_count(buf);

        let mut questions = Vec::new();
        let question_start_index = 12;
        let mut next_question_start_index = question_start_index;

        let mut count = 0;
        while count < qc {
            let filtere_header = &buf[next_question_start_index..];
            let termininator_index = find_null_terminator_index(filtere_header);
            let name = &filtere_header[..termininator_index].to_vec().clone();

            // termininator_index + 1 to ignore the null terminator after the domain name.
            let qtype = (filtere_header[termininator_index + 1..termininator_index + 3])
                .as_array::<2>()
                .unwrap();

            let qclass = (filtere_header[termininator_index + 3..termininator_index + 5])
                .as_array::<2>()
                .unwrap();

            let question = Question::new(
                name.clone(),
                QType::try_from(u16::from_be_bytes(*qtype)).unwrap(),
                QClass::try_from(u16::from_be_bytes(*qclass)).unwrap(),
            );
            questions.push(question);
            next_question_start_index = termininator_index + 5;

            count += 1;
        }

        questions
    }

    pub(crate) fn len(questions: Vec<Question>) -> usize {
        questions
            .iter()
            .map(|f| f.to_vec())
            .flat_map(|f| f)
            .collect::<Vec<_>>()
            .len()
    }
}

fn find_null_terminator_index(filtere_header: &[u8]) -> usize {
    let mut index = 0;
    for (i, ele) in filtere_header.iter().enumerate() {
        if ele == &0x00 {
            index = i;
            break;
        }
    }

    index
}

fn parse_questions_count(buf: [u8; 512]) -> u16 {
    let header = &buf[..12];
    println!("{:?}", header);

    let qc_byte = (header[4..=5]).as_array::<2>().unwrap();
    let count = u16::from_be_bytes(*qc_byte);
    println!("questions_count {:?}", count);
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
        inner.push(0 as u8);
        inner.extend_from_slice(&self.qtype.to_byte().to_be_bytes());
        inner.extend_from_slice(&self.qclass.to_byte().to_be_bytes());
        inner.extend_from_slice(&self.ttl.to_be_bytes());
        inner.extend_from_slice(&[0, 4]);
        inner.extend_from_slice(&[127, 0, 0, 1]);

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

    pub(crate) fn set_qr(&mut self, qr: &u8) {
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
