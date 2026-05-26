use std::net::Ipv4Addr;

use rand::RngExt;

pub(crate) struct Message {
    pub(crate) header: Header,
    question: Vec<Question>,
    answer: Answer,
}

impl Message {
    pub(crate) fn new() -> Self {
        Self {
            header: Header::default(),
            question: Vec::new(),
            answer: Answer::default(),
        }
    }

    pub(crate) fn to_vec(&mut self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&self.header.inner);
        for ele in &self.question {
            data.extend_from_slice(&ele.to_vec());
        }
        data.extend_from_slice(&self.answer.to_vec());

        data
    }

    pub(crate) fn set_question(&mut self, name: Vec<Vec<u8>>, qtype: QType, qclass: QClass) {
        self.header.set_question(name.len());

        for n in name {
            let question = Question::new(n);
            self.question.push(question);
        }
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
    pub(crate) fn new(name: Vec<u8>) -> Self {
        Self {
            name,
            qtype: QType::A,
            qclass: QClass::IN,
        }
    }

    fn to_vec(&self) -> Vec<u8> {
        let mut inner = Vec::<u8>::new();
        inner.extend_from_slice(self.name.as_slice());
        inner.extend_from_slice(&self.qtype.to_byte().to_be_bytes());
        inner.extend_from_slice(&self.qclass.to_byte().to_be_bytes());
        inner
    }

    pub(crate) fn parse_domain_name(buf: [u8; 512]) -> Vec<Vec<u8>> {
        let qc = parse_questions_count(buf);

        let mut names = Vec::new();
        let question_start_index = 12;
        let mut next_question_start_index = question_start_index;

        let mut count = 0;
        while (count < qc) {
            let filtere_header = &buf[next_question_start_index..];
            let mut termininator_index = find_null_terminator_index(filtere_header);
            termininator_index += 1; // including nul; terminator index
            let name = &filtere_header[..termininator_index].to_vec().clone();
            println!("parsed domain name {:?}", String::from_utf8(name.clone()));
            // println!("{:?}", &filtere_header[..termininator_index + 4]);

            let rtype = (filtere_header[termininator_index..termininator_index + 2])
                .as_array::<2>()
                .unwrap();
            // println!("rtype {:?}", rtype);
            let rclass = (filtere_header[termininator_index + 2..termininator_index + 4])
                .as_array::<2>()
                .unwrap();
            // println!("rclass {:?}", rclass);

            names.push(name.clone());
            count += 1;
            next_question_start_index = termininator_index + 5;
        }
        println!("{:?}", names);
        names
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

    println!("index {}", index);
    return index;
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

    fn set_question(&mut self, count: usize) {
        self.inner[4..6].copy_from_slice(&(count as u16).to_be_bytes())
    }

    fn set_answer(&mut self) {
        let count: u16 = 1;
        self.inner[6..8].copy_from_slice(&count.to_be_bytes())
    }
}
