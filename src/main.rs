use crate::dns_server::DnsServer;

mod core;
mod dns_server;

fn main() {
    DnsServer::new().start();
}
