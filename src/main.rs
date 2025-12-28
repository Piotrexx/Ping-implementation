use clap::Parser;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::io::ErrorKind;
use std::mem::MaybeUninit;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::{Duration, Instant};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t=String::from("8.8.8.8"))]
    ip: String,

    #[arg(short, long, default_value_t = 4)]
    packet_num: u16,
}

struct ICMPEchoRequestHeader {
    header_type: u8,
    code: u8,
    check_sum: u16,
    indentifier: u16,
    sequence_number: u16,
    payload: [u8; 4],
}

impl ICMPEchoRequestHeader {
    fn check_sum(&mut self) -> u16 {
        let header_and_code_word: u16 = ((self.header_type as u16) << 8) | self.code as u16;
        let check_sum_word: u16 = 0;
        let mut payload_words = [0u16; 2];
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
        let mut packet = Self {
            header_type: 8,
            code: 0,
            check_sum: 0,
            indentifier: 12,
            sequence_number: sequence_number,
            payload: *b"1234",
        };
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
    let args = Args::parse();

    let ip: Ipv4Addr = args.ip.parse().unwrap();

    let socket = Socket::new(Domain::IPV4, Type::RAW, Some(Protocol::ICMPV4)).unwrap();

    let addr = SockAddr::from(SocketAddr::new(IpAddr::V4(ip), 0));

    let mut failed: u8 = 0;
    let mut succeded: u8 = 0;

    socket
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    println!("Pinging {} with 4 bytes of data:", args.ip);
    for i in 0..args.packet_num {
        let send_time = Instant::now();
        let packet = ICMPEchoRequestHeader::new(i);
        socket.send_to(&packet.to_buf(), &addr).unwrap();
        let mut buf = [MaybeUninit::<u8>::uninit(); 1024];

        match socket.recv_from(&mut buf) {
            Ok((n, address)) => {
                let rtt = send_time.elapsed();
                let data = unsafe { std::slice::from_raw_parts(buf.as_ptr() as *const u8, n) };
                let ip_header_len = (data[0] & 0x0F) * 4;
                let icmp = &data[(ip_header_len as usize)..];
                let ip_header = &data[..(ip_header_len as usize)];
                if icmp[0] == 0 && icmp[0] == 0 {
                    succeded+=1;
                    println!(
                        "Received {} bytes from {:?}, time={}ms, TTL={}",
                        n,
                        address.as_socket_ipv4().unwrap(),
                        rtt.as_millis(),
                        ip_header[8]
                    );
                }
                else {
                    failed+=1;
                    println!("Returned packet is not a valid response");
                }
            }
            Err(err) => {
                failed+=1;
                if err.kind() == ErrorKind::TimedOut {
                    println!("Request timed out")
                } else {
                    return;
                }
            }
        }
    }

    println!("\nStatistics: \n \t Packets: Sent={}, Received={}, Lost={} ({}% loss)", args.packet_num, succeded, failed, ((failed/args.packet_num as u8)*100))
}
