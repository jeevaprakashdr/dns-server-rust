use zerocopy::{Immutable, IntoBytes};

#[derive(IntoBytes, Immutable)]
#[repr(C)]
pub(crate) struct DNSMessage {
    pub header: DNSHeader,
}

#[derive(IntoBytes, Immutable)]
#[repr(C)]
pub(crate) struct DNSHeader {
    pub id: u16,

    pub response: bool,
    pub opcode: u8,
    pub authoritative_answer: bool,
    pub truncation: bool,
    pub recursion_desired: bool,

    pub recursion_available: bool,
    pub reserved: bool,
    pub rescode: u8,
    pub questions_count: u16,
    pub answer_record_count: u16,
    pub authority_record_count: u16,
    pub additional_record_count: u16,
}

impl DNSHeader {
    pub(crate) fn new(id: u16) -> Self {
        Self {
            id,
            response: true,
            opcode: 0,
            authoritative_answer: false,
            truncation: false,
            recursion_desired: false,
            recursion_available: false,
            reserved: false,
            rescode: 0,
            questions_count: 0,
            answer_record_count: 0,
            authority_record_count: 0,
            additional_record_count: 0,
        }
    }
}
