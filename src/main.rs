#[allow(unused_imports)]
use std::net::UdpSocket;

use crate::core::DNSHeader;

mod core;

fn main() {
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
    let mut buf = [0; 512];

    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                println!("Received {} bytes from {}", size, source);
                let mut header = DNSHeader::new();
                header.set_id(u16::try_from(1234).unwrap());
                header.set_qr(true);

                println!("{:?}", &header.as_slice());

                udp_socket
                    .send_to(&header.as_slice(), source)
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
