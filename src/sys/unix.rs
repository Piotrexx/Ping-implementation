use std::{
    ffi::c_void,
    net::{Ipv4Addr, SocketAddr},
    time::Instant,
};

use libc::{
    __errno_location, AF_INET, CMSG_DATA, CMSG_FIRSTHDR, CMSG_NXTHDR, ETIMEDOUT, IP_RECVTTL,
    IP_TTL, IPPROTO_ICMP, IPPROTO_IP, SOCK_DGRAM, c_int, close, iovec, msghdr, recvmsg, sendto,
    setsockopt, sockaddr, sockaddr_in, socket, socklen_t,
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

    let enable: c_int = 1;

    unsafe {
        setsockopt(
            socket,
            IPPROTO_IP,
            IP_RECVTTL,
            &enable as *const _ as *const c_void,
            std::mem::size_of::<c_int>() as _,
        )
    };

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
        let mut cmsg_buf = [0u8; 128];

        let mut iov = iovec {
            iov_base: buf.as_mut_ptr() as *mut c_void,
            iov_len: buf.len(),
        };
        let mut sockaddr_src: sockaddr_in = unsafe { std::mem::zeroed() };
        let sockaddr_src_size: socklen_t = std::mem::size_of::<sockaddr_in>() as socklen_t;

        let mut message = msghdr {
            msg_name: &mut sockaddr_src as *mut _ as *mut _,
            msg_namelen: sockaddr_src_size,
            msg_iov: &mut iov as *mut _,
            msg_iovlen: 1,
            msg_control: cmsg_buf.as_mut_ptr() as *mut _,
            msg_controllen: cmsg_buf.len(),
            msg_flags: 0,
        };

        let msg_response = unsafe { recvmsg(socket, &mut message as *mut _, 0) };

        if msg_response < 0 {
            let err = unsafe { *__errno_location() };
            if err == ETIMEDOUT {
                failed += 1;
                println!("Request timed out.");
                continue;
            }
            panic!("recvmsg failed: {}", err)
        }
        let rtt = send_time.elapsed();
        let mut ttl: u8 = 0;

        unsafe {
            let mut cmsg = CMSG_FIRSTHDR(&message);

            while !cmsg.is_null() {
                match ((*cmsg).cmsg_level, (*cmsg).cmsg_type) {
                    (IPPROTO_IP, IP_TTL) => {
                        ttl = *(CMSG_DATA(cmsg));
                    }
                    _ => {}
                }
                cmsg = CMSG_NXTHDR(&message, cmsg);
            }
        }
        let icmp_data = &buf[..msg_response as usize];

        if icmp_data[0] == 0 && icmp_data[1] == 0 {
            succeded += 1;
            println!(
                "Received {} bytes from {:?}, time={}ms, TTL={}",
                msg_response,
                args.ip,
                rtt.as_millis(),
                ttl
            );
        } else {
            failed += 1;
            println!("Returned packet is not a valid response");
        }
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
