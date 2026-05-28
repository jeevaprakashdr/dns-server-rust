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
                println!("recieved {:?}", args);
                let forward_address = args.resolver.split(|p| p == ':').collect::<Vec<_>>();

                let mut message = Message::new();
                message.header.set_id(&buf[..2].to_vec());
                message.header.set_qr();
                message.header.set_opcode(&buf[2]);
                message.header.set_rcode();

                let questions = Question::parse(buf.as_slice());
                // let mut forwarder_answers = Vec::new();
                // if !forward_address.is_empty() && forward_address.len() == 2 {
                //     for q in questions.clone() {
                //         let mut m = Message::Create(message.header.clone(), q);
                //         udp_socket
                //             .send_to(&m.to_vec(), &args.resolver)
                //             .expect("Failed to send request to forward server");

                //         let mut response = [0u8; 512];
                //         let (count, _) = udp_socket.recv_from(&mut response).unwrap();
                //         println!("forward answeres {:?}", &response[..count]);
                //         forwarder_answers.extend_from_slice(&response[..count]);
                //     }
                // }

                message.set_question(questions);
                message.set_answer();

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
