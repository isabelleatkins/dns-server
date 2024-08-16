// Uncomment this block to pass the first stage
use bytes::{buf, BufMut, Bytes, BytesMut};
use message::{ARecord, Class, DnsMessage, Ipv4Address, Name, Record};
use record_database::RecordDatabase;
use std::{
    fs::File,
    io::{BufRead, BufReader},
    net::UdpSocket,
};

pub mod message;
pub mod record_database;
fn main() {
    let mut server = Server::new();
    server.run();
}

struct Server {
    socket: UdpSocket,
    records: RecordDatabase,
}

impl Server {
    fn new() -> Server {
        let records = populate_database();
        let socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
        Server { socket, records }
    }

    fn run(&mut self) {
        let mut buf = [0; 512];
        loop {
            match self.socket.recv_from(&mut buf) {
                Ok((size, source)) => {
                    let request: DnsMessage = buf.into();
                    println!("Request: {:?}", request);
                    for i in 0..2 {
                        println!("{:02x} ", buf[i]);
                    }
                    let name = request.get_question().get_qname().to_name();
                    println!("Name: {:?}", name);
                    let record = self.records.get_record(&name);
                    if record.is_none() {
                        eprintln!("Record not found for name: {:?}", name);
                    }
                    let dns_answer = DnsMessage::response(request, record);
                    let mut buffer = BytesMut::with_capacity(512);
                    dns_answer.write(&mut buffer);
                    debug_bytes(&buffer);
                    self.socket
                        .send_to(&buffer, source)
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

fn handle_request(request: DnsMessage) -> DnsMessage {
    todo!();
}

fn populate_database() -> RecordDatabase {
    let mut records: Vec<Box<dyn Record>> = vec![];
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() != 2 {
        println!(
            "Incorrect number of arguments passed in, expected 1 but got {}",
            args.len() - 1
        );
        std::process::exit(1);
    }
    let file = File::open(&args[1]).expect("Failed to open file");
    let file = BufReader::new(file);
    for line in file.lines() {
        let line = line.expect("Failed to read line");
        if line.is_empty() || line.starts_with(';') {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() != 4 {
            println!(
                "Expected 4 parts to entry in dns records file but got {}",
                parts.len()
            );
            continue;
        }
        println!("Parts: {:?}", parts);
        match parts[2] {
            "A" => {
                let a_record = ARecord::new(
                    Name::new(parts[0]),
                    Class::try_from(parts[1]).unwrap(),
                    1, // TO DO
                    Ipv4Address::new(parts[3]).unwrap(),
                );
                println!("A RECORD {:?}", a_record);
                records.push(Box::new(a_record));
            }
            _ => {
                println!("Unsupported record type: {}", parts[2]);
            }
        }
    }
    RecordDatabase::new(records)
}

fn debug_bytes(bytes: &[u8]) {
    for (i, byte) in bytes.iter().enumerate() {
        print!("{:02x} ", byte);
        if (i + 1) % 16 == 0 {
            println!();
        }
    }
    println!();
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setup() {
        populate_database();
    }
}
