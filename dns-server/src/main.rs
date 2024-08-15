// Uncomment this block to pass the first stage
use bytes::{buf, BufMut, Bytes, BytesMut};
use message::DnsMessage;
use std::net::UdpSocket;

pub mod message;
fn main() {
    println!("Starting DNS server on port 2053");
    let udp_socket = UdpSocket::bind("127.0.0.1:2053").expect("Failed to bind to address");
    let mut buf = [0; 512];

    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                let request: DnsMessage = buf.into();
                println!("Request: {:?}", request);
                let mut response = vec![];
                for i in 0..2 {
                    println!("{:02x} ", buf[i]);
                }
                response = [response, vec![buf[0], buf[1]]].concat();
                response = [
                    response,
                    vec![0x80, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00],
                ]
                .concat();
                response.extend_from_slice(&b"\x0cgoogle.com\x02io"[..]); //QNAME
                response.put_u8(0u8); // null byte to end the label sequence that is QNAME
                response.put_u16(1u16); // QTYPE for A record
                response.put_u16(1u16); // QCLASS for IN

                /////ATTEMPT USING DNS MESSAGE
                let dns_answer = DnsMessage::new(&buf);
                let mut buf_2 = BytesMut::with_capacity(512);
                dns_answer.write(&mut buf_2);
                udp_socket
                    .send_to(&buf_2, source)
                    .expect("Failed to send response");
            }
            Err(e) => {
                eprintln!("Error receiving data: {}", e);
                break;
            }
        }
    }
}
