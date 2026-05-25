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
                message.header.set_id(&buf[..2].to_vec());
                message.header.set_qr();
                message.header.set_opcode(&buf[2]);
                message.header.set_rcode();
                let domain_name = parse_domain_name(buf);
                message.set_question(domain_name.clone(), QType::A, QClass::IN);
                message.set_answer(domain_name, QType::A, QClass::IN);

                let message = message.to_vec();
                // println!("{:?}", message);

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

fn parse_domain_name(buf: [u8; 512]) -> Vec<u8> {
    // println!("{:?}", &buf[12..]);
    let filtere_header = &buf[12..];

    let split_filtered = filtere_header
        .split(|&pred| pred == 0x00)
        .collect::<Vec<_>>();

    let name = split_filtered.first().unwrap();
    println!("parsed domain name {:?}", String::from_utf8(name.to_vec()));

    let mut name = name.to_vec();
    name.push(0 as u8);
    name
}
