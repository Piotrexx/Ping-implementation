use std::time::Instant;

use windows_sys::Win32::Networking::WinSock::{
    AF_INET, IN_ADDR, INVALID_SOCKET, IPPROTO_ICMP, SO_RCVTIMEO, SOCK_RAW, SOCKADDR, SOCKADDR_IN,
    SOCKADDR_STORAGE, SOCKET, SOCKET_ERROR, SOL_SOCKET, WSACleanup, WSADATA, WSAETIMEDOUT,
    WSAGetLastError, WSAStartup, closesocket, htons, inet_pton, recvfrom, sendto, setsockopt,
    socket,
};

use crate::{Args, protocol::ICMPEchoRequestHeader};

// struct ResponseData<'a> {
//     data: &'a [u8],
//     time_elapsed: u128
// }

fn ipv4_win_socket_address(ip: String, port: u16) -> SOCKADDR_IN {
    let mut addr: SOCKADDR_IN = unsafe { std::mem::zeroed() };

    addr.sin_family = AF_INET as u16;
    addr.sin_port = unsafe { htons(port) };

    let ip_cstr = std::ffi::CString::new(ip).unwrap();

    let res = unsafe {
        inet_pton(
            AF_INET.into(),
            ip_cstr.as_bytes().as_ptr(),
            &mut addr.sin_addr as *mut IN_ADDR as *mut _,
        )
    };

    if res != 1 {
        panic!("Invalid IPv4 address");
    }

    addr
}

pub fn send_icmp_packets(args: Args) {
    // -> Vec<ResponseData<'static>> {
    let mut wsa_init: WSADATA = unsafe { std::mem::zeroed() };
    let init_result = unsafe { WSAStartup(0x202, &mut wsa_init) };

    if init_result != 0 {
        panic!("winsock initialization failed: {}", unsafe {
            WSAGetLastError()
        });
    };

    let socket: SOCKET = unsafe { socket(AF_INET as i32, SOCK_RAW, IPPROTO_ICMP) };
    if socket == INVALID_SOCKET {
        panic!("socket init failed: {}", unsafe { WSAGetLastError() });
    }

    let timeout_ms = 5000;

    let ret = unsafe {
        setsockopt(
            socket,
            SOL_SOCKET,
            SO_RCVTIMEO,
            &timeout_ms as *const _ as *const u8,
            std::mem::size_of_val(&timeout_ms) as i32,
        )
    };

    if ret == SOCKET_ERROR {
        panic!("setsockopt (timeout init) failed: {}", unsafe {
            WSAGetLastError()
        })
    }

    let address = ipv4_win_socket_address(args.ip.clone(), 0);

    let mut succeded: u8 = 0;
    let mut failed: u8 = 0;

    // let mut data_collector: Vec<ResponseData> = Vec::with_capacity(args.packet_num as usize);

    for i in 0..args.packet_num {
        let data = ICMPEchoRequestHeader::new(i);
        let buf = data.to_buf();
        let send_time = Instant::now();
        let msg = unsafe {
            sendto(
                socket,
                buf.as_ptr(),
                buf.len() as i32,
                0,
                &address as *const _ as *const SOCKADDR,
                std::mem::size_of::<SOCKADDR_IN>() as i32,
            )
        };

        if msg == SOCKET_ERROR {
            panic!("sendto failed: {}", unsafe { WSAGetLastError() });
        }
        let mut buf = [0u8; 1024];
        let mut from: SOCKADDR_STORAGE = unsafe { std::mem::zeroed() };
        let mut fromlen: i32 = std::mem::size_of::<SOCKADDR_STORAGE>() as i32;
        let response = unsafe {
            recvfrom(
                socket,
                buf.as_mut_ptr(),
                buf.len() as i32,
                0,
                &mut from as *mut _ as *mut SOCKADDR,
                &mut fromlen,
            )
        };
        if response == SOCKET_ERROR {
            let err = unsafe { WSAGetLastError() };
            if err == WSAETIMEDOUT {
                failed += 1;
                println!("Request timed out.");
                continue;
            }
            panic!("recvfrom failed: {}", unsafe { WSAGetLastError() });
        }

        let rtt = send_time.elapsed();

        let data = &buf[..response as usize];
        // data_collector.push(ResponseData { data: data, time_elapsed: rtt.as_millis() });
        let ip_header_len = (data[0] & 0x0F) * 4;
        let icmp = &data[(ip_header_len as usize)..];
        let ip_header = &data[..(ip_header_len as usize)];
        if icmp[0] == 0 && icmp[1] == 0 {
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
        closesocket(socket);
        WSACleanup();
    };
    println!(
        "\nStatistics: \n \t Packets: Sent={}, Received={}, Lost={} ({}% loss)",
        args.packet_num,
        succeded,
        failed,
        ((failed / args.packet_num as u8) * 100)
    )
}
