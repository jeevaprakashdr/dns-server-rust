#[allow(unused_imports)]
use std::net::UdpSocket;

use crate::core::{Message, QClass, QType};

mod core;

fn main() {
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
    let mut buf = [0; 512];

    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                println!("Received {} bytes from {}", size, source);

                let id = &buf[..2].to_vec();
                let mut message = Message::new();
                message.header.set_id(id);
                message.header.set_qr();
                message.header.set_opcode(&buf[2]);
                message.header.set_rcode();
                message.set_question("codecrafters.io".to_string(), QType::A, QClass::IN);
                message.set_answer("codecrafters.io".to_string(), QType::A, QClass::IN);

                let message = message.to_vec();
                println!("{:?}", message);

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
