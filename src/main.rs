#[allow(unused_imports)]
use std::net::UdpSocket;
use std::vec;

use clap::Parser;

use crate::core::{Message, Question};

mod core;

fn main() {
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
    let mut buf = [0; 512];

    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                println!("Received {} bytes from {}", size, source);
                println!("recieved {:?}", &buf[..100]);
                let args = ServerArguments::parse();
                println!("arguments {:?}", args);

                let mut message = Message::new();
                message.header.set_id(&buf[..2].to_vec());
                message.header.set_qr();
                message.header.set_opcode(&buf[2]);
                message.header.set_rcode();
                let questions = Question::parse(buf.as_slice());
                let mut forwarder_responses = Vec::new();
                for question in questions.clone() {
                    let mut forwarder_message = message.clone();
                    forwarder_message.header.reset_qr();
                    forwarder_message.header.reset_rcode();
                    forwarder_message.set_question(vec![question]);

                    let resolver_socket =
                        UdpSocket::bind("0.0.0.0:0").expect("Failed to bind addresss");
                    resolver_socket
                        .send_to(&forwarder_message.to_vec(), "127.0.0.1:5354")
                        .unwrap();
                    let mut buf = [0u8; 512];
                    let (number_of_bytes, _) = resolver_socket.recv_from(&mut buf).unwrap();
                    forwarder_responses.push(buf[..number_of_bytes].to_vec());
                }

                message.set_question(questions);
                let answers = forwarder_responses
                    .iter()
                    .map(|fr| {
                        let message = Message::parse(&fr);
                        message.get_answers()
                    })
                    .flat_map(|f| f)
                    .collect::<Vec<_>>();

                message.set_fowarder_answer(answers);

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

#[derive(clap::Parser, Debug)]
pub struct ServerArguments {
    #[arg(short, long)]
    pub resolver: String,
}
