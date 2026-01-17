use std::{
    ffi::c_void, mem::zeroed, net::{Ipv4Addr, SocketAddr}, time::Instant
};

use libc::{
    __errno_location, AF_INET, ETIMEDOUT, IPPROTO_ICMP, SOCK_DGRAM, SOCK_RAW, close, recvfrom,
    sendto, sockaddr, sockaddr_in, socket, socklen_t,
};

use crate::{Args, protocol::ICMPEchoRequestHeader};

fn address_from_string(ip: String) -> (*const sockaddr, socklen_t) {
    let sock_addr: Ipv4Addr = ip.parse().unwrap();

    match SocketAddr::new(std::net::IpAddr::V4(sock_addr), 0) {
        SocketAddr::V4(addr) => {
            let sockaddr_in = sockaddr_in {
                sin_family: AF_INET as u16,
                sin_port: addr.port().to_be(),
                sin_addr: libc::in_addr {
                    s_addr: u32::from_ne_bytes(addr.ip().octets()).to_be(),
                },
                sin_zero: [0; 8],
            };

            let ptr = &sockaddr_in as *const sockaddr_in as *const sockaddr;
            let len = std::mem::size_of::<sockaddr_in>() as libc::socklen_t;

            (ptr, len)
        }
        _ => {
            // SocketAddr::V6(addr) => {
            unimplemented!("Not implemented yet")
        }
    }
}

pub fn send_icmp_packets(args: Args) {
    let socket = unsafe { socket(AF_INET, SOCK_DGRAM, IPPROTO_ICMP) };
    if socket < 0 {
        let err = unsafe { *__errno_location() };
        panic!("Socket init failed: {}", err);
    }

    let mut succeded: u8 = 0;
    let mut failed: u8 = 0;

    for i in 0..args.packet_num {
        let data = ICMPEchoRequestHeader::new(i);
        let buf = data.to_buf();
        let send_time = Instant::now();
        let (addres_pointer, addr_len) = address_from_string(args.ip.clone());
        let msg = unsafe {
            sendto(
                socket,
                buf.as_ptr() as *const c_void,
                buf.len(),
                0,
                addres_pointer,
                addr_len,
            )
        };

        if msg < 0 {
            let err = unsafe { *__errno_location() };
            panic!("sendto failed: {}", err)
        }
        let mut buf = [0u8; 1024];
        let mut sockaddr_src: sockaddr_in = unsafe {std::mem::zeroed()};
        let mut sockaddr_src_size: socklen_t = std::mem::size_of::<sockaddr_in>() as socklen_t;
        let response = unsafe {
            recvfrom(
                socket,
                buf.as_mut_ptr() as *mut c_void,
                buf.len(),
                0,
                &mut sockaddr_src as *mut _ as *mut sockaddr,
                &mut sockaddr_src_size as *mut socklen_t
            )
        };

        if response < 0 {
            let err = unsafe { *__errno_location() };
            if err == ETIMEDOUT {
                failed += 1;
                println!("Request timed out.");
                continue;
            }
            panic!("recvfrom failed: {}", err)
        }
        let rtt = send_time.elapsed();

        let data = &buf[..response as usize];
        let ip_header_len = (data[0] & 0x0F) * 4;
        let icmp = &data[(ip_header_len as usize)..];
        let ip_header = &data[..(ip_header_len as usize)];
        if icmp[0] == 0 && icmp[0] == 0 {
            succeded += 1;
            println!(
                "Received {} bytes from {:?}, time={}ms, TTL={}",
                response,
                std::net::Ipv4Addr::new(ip_header[12], ip_header[13], ip_header[14], ip_header[15]),
                rtt.as_millis(),
                ip_header[8]
            );
        } else {
            failed += 1;
            println!("Returned packet is not a valid response");
        }
        succeded += 1;
    }

    unsafe {
        close(socket);
    }

    println!(
        "\nStatistics: \n \t Packets: Sent={}, Received={}, Lost={} ({}% loss)",
        args.packet_num,
        succeded,
        failed,
        ((failed / args.packet_num as u8) * 100)
    )
}
