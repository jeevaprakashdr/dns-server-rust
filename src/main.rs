#[allow(unused_imports)]
use std::net::UdpSocket;

use crate::core::{Answer, Message, Question};

mod core;

fn main() {
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
    let mut buf = [0; 512];

    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                println!("Received {} bytes from {}", size, source);
                println!("recieved {:?}", &buf[..100]);

                let mut message = Message::new();
                message.header.set_id(&buf[..2].to_vec());
                message.header.set_qr();
                message.header.set_opcode(&buf[2]);
                message.header.set_rcode();
                let questions = Question::parse_new(buf.as_slice());
                message.set_question(questions.clone());

                let mut answers = Vec::new();
                for question in questions.clone() {
                    let answer = Answer::new(question.name, question.qtype, question.qclass);
                    answers.push(answer);
                }
                message.set_answer(answers);

                let message = message.to_vec();
                println!("sent message {:?}", &message);

                udp_socket
                    .send_to(&message, source)
                    .inspect(|f| println!("passed {}", f))
                    .expect("Failed to send response");
            }
            Err(e) => {
                eprintln!("Error receiving data: {}", e);
                break;
            }
        }
    }
}
