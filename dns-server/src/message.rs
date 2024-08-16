use anyhow::anyhow;
use anyhow::Error;
use bytes::{BufMut, BytesMut};
use std::{fmt, path::Display};

#[derive(Debug)]
pub struct DnsMessage {
    header: Header,
    question: Question,
    answer: Option<Answer>,
}

impl DnsMessage {
    pub fn answer(request: DnsMessage, record: Option<&Box<dyn Record>>) -> DnsMessage {
        let mut whole_answer = request;
        whole_answer.header.ancount = 1; // Needed to add this - without it answer section doesn't appear, and it just ignores the last 26 bytes
        whole_answer.answer = Answer::new(record);
        println!("Answer: {:?}", whole_answer.answer);
        //whole_answer.answer = Answer::from_request(request, record);
        // = DnsMessage {
        //     header: Header::from_request(request),
        //     question: Question::from_request(request),
        //     answer: Answer::from_request(request), //TODO
        // }
        whole_answer
    }
    pub fn write(&self, buf: &mut BytesMut) {
        self.header.write(buf);
        self.question.write(buf);
        if let Some(answer) = &self.answer {
            answer.write(buf);
        }
    }
    pub fn get_question(&self) -> &Question {
        &self.question
    }
}

#[derive(Debug)]
pub struct Question {
    name: QName,
    qtype: u16,
    qclass: u16,
}

impl Question {
    fn write(&self, buf: &mut BytesMut) {
        for label in &self.name.labels {
            buf.put_u8(label.value.len() as u8);
            buf.put(label.value.as_bytes());
        }
        buf.put_u8(0); //Writes a zero length byte to indicate the end of the domain name
        buf.put_u16(self.qtype);
        buf.put_u16(self.qclass);
    }
    fn from_request(request: &[u8]) -> Question {
        let mut qname_index = 12;
        let mut labels = vec![];
        loop {
            let label_length = request[qname_index] as usize;
            if label_length == 0 {
                break;
            }
            let label = String::from_utf8(
                request[qname_index + 1..qname_index + 1 + label_length].to_vec(),
            )
            .unwrap();
            labels.push(Label { value: label });
            qname_index += label_length + 1;
        }
        let qtype: u16 = (request[qname_index + 1] as u16) << 8 | request[qname_index + 2] as u16;
        let qclass: u16 = (request[qname_index + 3] as u16) << 8 | request[qname_index + 4] as u16;
        Question {
            name: QName { labels },
            qtype,
            qclass, //TO DO
        }
    }
    pub fn get_qname(&self) -> &QName {
        &self.name
    }
}

#[derive(Debug)]
pub struct QName {
    labels: Vec<Label>,
}

impl QName {
    pub fn to_name(&self) -> Name {
        let mut name = String::new();
        for label in &self.labels {
            name.push_str(&label.value);
            name.push('.');
        }
        name.pop();
        Name(name)
    }
}
#[derive(Debug)]
struct Label {
    value: String,
}
impl From<&Header> for [u8; 12] {
    fn from(header: &Header) -> [u8; 12] {
        let mut bytes = [0; 12];
        bytes[0] = (header.id >> 8) as u8;
        bytes[1] = header.id as u8;
        bytes[2] = (header.qr as u8) << 7
            | header.opcode << 3
            | (header.aa as u8) << 2
            | (header.tc as u8) << 1
            | header.rd as u8;
        bytes[3] = (header.ra as u8) << 7 | header.z << 4 | header.rcode;
        bytes[4] = (header.qdcount >> 8) as u8;
        bytes[5] = header.qdcount as u8;
        bytes[6] = (header.ancount >> 8) as u8;
        bytes[7] = header.ancount as u8;
        bytes[8] = (header.nscount >> 8) as u8;
        bytes[9] = header.nscount as u8;
        bytes[10] = (header.arcount >> 8) as u8;
        bytes[11] = header.arcount as u8;
        bytes
    }
}
impl Into<DnsMessage> for [u8; 512] {
    fn into(self) -> DnsMessage {
        let mut qname_index = 12;
        let mut labels = vec![];
        loop {
            let label_length = self[qname_index] as usize;
            if label_length == 0 {
                break;
            }
            let label =
                String::from_utf8(self[qname_index + 1..qname_index + 1 + label_length].to_vec())
                    .unwrap();
            println!("Label: {}", label);
            labels.push(Label { value: label });
            qname_index += label_length + 1;
        }
        let qtype: u16 = (self[qname_index + 1] as u16) << 8 | self[qname_index + 2] as u16;
        let qclass: u16 = (self[qname_index + 3] as u16) << 8 | self[qname_index + 4] as u16;
        DnsMessage {
            header: Header {
                id: (self[0] as u16) << 8 | self[1] as u16,
                qr: (self[2] >> 7) == 1,
                opcode: (self[2] >> 3) & 0b1111,
                aa: ((self[2] >> 2) & 0b1) == 1,
                tc: ((self[2] >> 1) & 0b1) == 1,
                rd: (self[2] & 0b1) == 1,
                ra: (self[3] >> 7) == 1,
                z: (self[3] >> 4) & 0b111,
                rcode: self[3] & 0b1111,
                qdcount: (self[4] as u16) << 8 | self[5] as u16,
                ancount: (self[6] as u16) << 8 | self[7] as u16,
                nscount: (self[8] as u16) << 8 | self[9] as u16,
                arcount: (self[10] as u16) << 8 | self[11] as u16,
            },
            question: Question {
                name: QName { labels },
                qtype,
                qclass,
            },
            answer: None, //Maybe update since I guess we need to accept the creation of dns records at some point? And that would presumably use the answer field and perhaps have None for question field
        }
    }
}

impl fmt::Display for DnsMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "DnsMessage {{ header: {:?}, question: {:?} }}",
            self.header, self.question
        )
    }
}

#[derive(Debug)]
struct Header {
    id: u16,
    qr: bool,
    opcode: u8,
    aa: bool,
    tc: bool,
    rd: bool,
    ra: bool,
    z: u8,
    rcode: u8,
    qdcount: u16,
    ancount: u16,
    nscount: u16,
    arcount: u16,
}

impl Header {
    fn write(&self, buf: &mut BytesMut) {
        buf.put_u16(self.id);
        buf.put_u8(
            (self.qr as u8) << 7
                | self.opcode << 3
                | (self.aa as u8) << 2
                | (self.tc as u8) << 1
                | self.rd as u8,
        );
        buf.put_u8((self.ra as u8) << 7 | self.z << 4 | self.rcode);
        buf.put_u16(self.qdcount);
        buf.put_u16(self.ancount);
        buf.put_u16(self.nscount);
        buf.put_u16(self.arcount);
    }
    fn from_request(request: &[u8]) -> Header {
        Header {
            id: (request[0] as u16) << 8 | request[1] as u16,
            qr: true,
            opcode: 0,
            aa: false,
            tc: false,
            rd: false,
            ra: false,
            z: 0,
            rcode: 0,
            qdcount: 1,
            ancount: 1, // Need to make this one to indicate one answer in response , need update
            nscount: 0,
            arcount: 0,
        }
    }
}

enum QType {
    A = 1,
    // NS = 2,
    // MD = 3,
    // MF = 4,
    CNAME = 5,
    // SOA = 6,
    // MB = 7,
    // MG = 8,
    // MR = 9,
    // NULL = 10,
    // WKS = 11,
    // PTR = 12,
    // HINFO = 13,
    // MINFO = 14,
    // MX = 15,
    // TXT = 16,
}

impl TryFrom<u8> for QType {
    type Error = &'static str;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(QType::A),
            // 2 => Ok(QType::NS),
            // 3 => Ok(QType::MD),
            // 4 => Ok(QType::MF),
            5 => Ok(QType::CNAME),
            // 6 => Ok(QType::SOA),
            // 7 => Ok(QType::MB),
            // 8 => Ok(QType::MG),
            // 9 => Ok(QType::MR),
            // 10 => Ok(QType::NULL),
            // 11 => Ok(QType::WKS),
            // 12 => Ok(QType::PTR),
            // 13 => Ok(QType::HINFO),
            // 14 => Ok(QType::MINFO),
            // 15 => Ok(QType::MX),
            // 16 => Ok(QType::TXT),
            _ => Err("Invalid QType"),
        }
    }
}

#[derive(Debug)]
struct Answer {
    resource_records: Vec<ResourceRecord>,
}

impl Answer {
    fn new(record: Option<&Box<dyn Record>>) -> Option<Answer> {
        match record {
            Some(record) => Some(Answer {
                resource_records: vec![record.into_resource_record()],
            }),
            None => None,
        }
    }
    fn write(&self, buf: &mut BytesMut) {
        for record in &self.resource_records {
            record.write(buf);
        }
    }
}

#[derive(Debug)]
struct ResourceRecord {
    name: Name,
    r#type: RrType,
    class: Class, // Turn this into a type see section 3.2.4
    ttl: u32,
    rdlength: u16,
    rdata: Vec<u8>, //Make this an actual type when looked into more
}

impl ResourceRecord {
    fn write(&self, buf: &mut BytesMut) {
        for label in &self.name.to_qname().labels {
            buf.put_u8(label.value.len() as u8);
            buf.put(label.value.as_bytes());
        }
        buf.put_u8(0);
        buf.put_u16(1); // TO DO
        buf.put_u16(1); // TO DO
        buf.put_u32(self.ttl);
        buf.put_u16(self.rdlength);
        for i in 0..4 {
            buf.put_u8(self.rdata[i]);
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Class {
    IN = 1,
}

impl TryFrom<&str> for Class {
    type Error = &'static str;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "IN" => Ok(Class::IN),
            _ => Err("Invalid Class"),
        }
    }
}

pub trait Record {
    fn into_resource_record(&self) -> ResourceRecord;
    fn get_name(&self) -> &Name;
}

#[derive(Debug, Clone)]
pub struct ARecord {
    pub name: Name,
    class: Class,
    ttl: u32,
    address: Ipv4Address,
}

impl ARecord {
    pub fn new(name: Name, class: Class, ttl: u32, address: Ipv4Address) -> ARecord {
        ARecord {
            name,
            class,
            ttl,
            address,
        }
    }
}

impl Record for ARecord {
    fn into_resource_record(&self) -> ResourceRecord {
        ResourceRecord {
            name: self.name.clone(),
            r#type: RrType::A,
            class: self.class,
            ttl: self.ttl,
            rdlength: 4,
            rdata: self.address.0.to_vec(),
        }
    }
    fn get_name(&self) -> &Name {
        &self.name
    }
}
#[derive(Debug)]
enum RrType {
    A,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Name(String);

impl Name {
    pub fn new(name: &str) -> Name {
        Name(name.to_string())
    }

    pub fn to_qname(&self) -> QName {
        let mut labels = vec![];
        for label in self.0.split('.') {
            labels.push(Label {
                value: label.to_string(),
            });
        }
        QName { labels }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Ipv4Address([u8; 4]);

impl Ipv4Address {
    pub fn new(address: &str) -> Result<Ipv4Address, Error> {
        let octets: Vec<u8> = address
            .split('.')
            .map(|octet| octet.parse().unwrap())
            .collect();
        if octets.len() != 4 {
            return Err(anyhow!("Invalid IPv4 address"));
        }
        Ok(Ipv4Address([octets[0], octets[1], octets[2], octets[3]]))
    }
    pub fn octets(&self) -> [u8; 4] {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{convert::TryFrom, vec};

    #[test]
    fn test_write_answer() {
        let mut buf = BytesMut::with_capacity(512);
        let record = ARecord::new(
            Name::new("example.com"),
            Class::IN,
            1,
            Ipv4Address::new("127.0.0.1").unwrap(),
        );
        let record: Box<dyn Record> = Box::new(record);
        let answer = Answer::new(Some(&record)).unwrap();
        answer.write(&mut buf);
        assert_eq!(
            buf.to_vec(),
            vec![
                7, 101, 120, 97, 109, 112, 108, 101, 3, 99, 111, 109, 0, 0, 1, 0, 1, 0, 0, 0, 1, 0,
                4, 127, 0, 0, 1
            ]
        );
    }

    #[test]
    fn test_write_question() {
        let mut buf = BytesMut::with_capacity(512);
        let question = Question {
            name: QName {
                labels: vec![
                    Label {
                        value: "example".to_string(),
                    },
                    Label {
                        value: "com".to_string(),
                    },
                ],
            },
            qtype: 1,
            qclass: 1,
        };
        question.write(&mut buf);
        assert_eq!(
            buf.to_vec(),
            vec![7, 101, 120, 97, 109, 112, 108, 101, 3, 99, 111, 109, 0, 0, 1, 0, 1]
        );
    }
}
