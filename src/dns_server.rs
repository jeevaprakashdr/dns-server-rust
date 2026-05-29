use std::net::UdpSocket;

use crate::core::{Message, Question};

pub(crate) struct DnsServer {
    host: String,
    port: u16,
}

impl DnsServer {
    pub(crate) fn new() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 2053,
        }
    }

    fn get_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    pub(crate) fn start(&self) {
        let udp_socket = UdpSocket::bind(self.get_address()).expect("Failed to bind to address");
        let mut buf = [0; 512];

        loop {
            match udp_socket.recv_from(&mut buf) {
                Ok((size, source)) => {
                    println!("Received {} bytes from {}", size, source);

                    let mut message = Message::new();
                    message
                        .header
                        .set_id(&buf[..2].to_vec())
                        .set_qr()
                        .set_opcode(&buf[2])
                        .set_rcode();

                    let questions = Question::parse(buf.as_slice());
                    let forwarder_responses =
                        forward_questions(&udp_socket, message.clone(), questions.clone());
                    let answers = forwarder_responses
                        .iter()
                        .map(|fresponse| {
                            let message = Message::parse(&fresponse);
                            message.get_answers()
                        })
                        .flat_map(|answeres| answeres)
                        .collect::<Vec<_>>();

                    message.set_question(questions);
                    message.set_fowarder_answer(answers);

                    udp_socket
                        .send_to(&message.to_vec(), source)
                        .inspect(|f| println!("Passed {}", f))
                        .expect("Failed to send response");
                }
                Err(e) => {
                    eprintln!("Error receiving data: {}", e);
                    break;
                }
            }
        }
    }
}

fn forward_questions(
    udp_socket: &UdpSocket,
    message: Message,
    questions: Vec<Question>,
) -> Vec<Vec<u8>> {
    let mut forwarder_responses = Vec::new();

    for question in questions.clone() {
        let mut forwarder_message = message.clone();
        forwarder_message.header.reset_qr();
        forwarder_message.header.reset_rcode();
        forwarder_message.set_question(vec![question]);

        udp_socket
            .send_to(&forwarder_message.to_vec(), "localhost:5354")
            .unwrap();
        let mut buf = [0u8; 512];
        let (number_of_bytes, _) = udp_socket.recv_from(&mut buf).unwrap();
        forwarder_responses.push(buf[..number_of_bytes].to_vec());
    }

    forwarder_responses
}
