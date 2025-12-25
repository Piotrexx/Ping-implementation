use std::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};
use std::mem::MaybeUninit;
use std::time::Duration;
use socket2::{Domain, Protocol, Socket, Type, SockAddr};


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

    fn new(sequence_number: u16) -> Self {
        let mut packet = Self { header_type: 8, code: 0, check_sum: 0, indentifier: 12, sequence_number: sequence_number, payload: *b"1234" };
        packet.check_sum = packet.check_sum();
        packet
    }

    fn to_buf(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(12);
        buf.push(self.header_type);
        buf.push(self.code);
        buf.extend_from_slice(&self.check_sum.to_be_bytes());
        buf.extend_from_slice(&self.indentifier.to_be_bytes());
        buf.extend_from_slice(&self.sequence_number.to_be_bytes());
        buf.extend_from_slice(&self.payload);
        buf

    }
}


fn main() {

    let socket = Socket::new(Domain::IPV4, Type::RAW, Some(Protocol::ICMPV4)).unwrap();
    
    let addr = SockAddr::from(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(8, 22, 122, 111)),
        0,
    ));

    socket.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
    for i in 1..4 {
        let packet = ICMPEchoRequestHeader::new(i);
        println!("Bytes sent: {}", socket.send_to(&packet.to_buf(), &addr).unwrap());

        let mut buf = [MaybeUninit::<u8>::uninit(); 1024];
        let (n, from) = socket.recv_from(&mut buf).unwrap();

        let data = unsafe {
    std::slice::from_raw_parts(buf.as_ptr() as *const u8, n)
};

        println!("Received {} bytes from {:?}", n, from);
        println!("Payload: {:?}", &buf[..n]);
        println!("Data: {:?}", data);
        let icmp_payload = &data[28..];
        println!("String Data: {:?}", str::from_utf8(icmp_payload).unwrap());
    }
    
    println!("Hello, world!");
}
