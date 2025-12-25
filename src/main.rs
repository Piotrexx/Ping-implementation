use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};

use socket2::{Domain, Protocol, Socket, Type};


struct ICMPEchoRequestHeader {
    header_type: u8,
    code: u8,
    check_sum: u16,
    indentifier: u16,
    sequence_number: u16,
    payload: [u8;4],
}

impl ICMPEchoRequestHeader {
    fn check_sum(&mut self) -> u16{
        let header_and_code_word: u16 = ((self.header_type as u16) << 8) | self.code as u16;
        let check_sum_word: u16 = 0;
        let mut payload_words= [0u16; 2];
        payload_words[0] = ((self.payload[0] as u16) << 8) | self.payload[1] as u16;
        payload_words[1] = ((self.payload[2] as u16) << 8) | self.payload[3] as u16;

        let mut sum: u32 = 0;
        sum += header_and_code_word as u32;
        sum += check_sum_word as u32;
        sum += payload_words[0] as u32;
        sum += payload_words[1] as u32;
        sum += self.indentifier as u32;
        sum += self.sequence_number as u32;

        while (sum >> 16) != 0 {
            sum = (sum & 0xFFFF) + (sum >> 16);
        }

        !(sum as u16)
    }

    fn new(&mut self, sequence_number: u16) -> Self {
        Self { header_type: 8, code: 0, check_sum: self.check_sum(), indentifier: 12, sequence_number: sequence_number, payload: *b"1234" }
    }
}


fn main() {

    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::ICMPV4));
    
    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(8,8,8,8)), 0);

    socket.unwrap().bind(&address);
    socket.unwrap().send(&[8, 0, 0, 1]);
    
    println!("Hello, world!");
}
