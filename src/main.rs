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
                let mut message = Message::new();
                message.header.set_id(u16::try_from(1234).unwrap());
                message.header.set_qr(true);
                message.set_question("codecrafter.com", QType::A, QClass::IN);

                println!("{:?}", &message.to_vec());

                udp_socket
                    .send_to(&message.to_vec(), source)
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
