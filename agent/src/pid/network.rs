#![allow(dead_code)]
use std::net::{SocketAddr,IpAddr};

use std::error::Error;
use std::str;
use std::str::FromStr;
use std::path::Path;
use std::io::prelude::*;
use std::fs::File;

use regex::Regex;

#[derive(Debug)]
enum NetworkProtocol {
    Unknown,
    Ipv4,
    Ipv6
}

#[derive(Debug)]
enum Family { // We only care about TCP and UDP for now
    Unknown,
    Tcp,
    Udp
}

#[derive(Debug)]
enum ConntrackState {
    Establised,
    SynSent,
    SynRecv,
    FinWait1,
    FinWait2,
    TimeWait,
    Close,
    CloseWait,
    LastAck,
    Listen,
    Closing,
    Unknown
}

#[derive(Debug)]
enum NetworkError {
    ParserError
}

#[derive(Debug)]
struct Conntrack {
    network_protocol: self::NetworkProtocol,
    network_protocol_number: i32,
    protocol: self::Family,
    protocol_number: i32,
    timeout: i32,
    src: SocketAddr,
    dst: SocketAddr,
    state: self::ConntrackState
}

fn get_conntrack() -> Result<Vec<Conntrack>, NetworkError> {
    let mut f = match File::open("/proc/net/nf_conntrack") {
        Err(why) => panic!("couldn't open /proc/net/nf_conntrack conntrack: {}", why.description()),
        Ok(file) => file,
    };

    // read the whole file
    let mut s = String::new();
    match f.read_to_string(&mut s) {
        Err(why) => panic!("couldn't read /proc/net/nf_conntrack: {}", why.description()),
        Ok(content) => content
    };

    let re: Regex = Regex::new(r"(ipv4|ipv6)     (\d+) (tcp|udp)      (\d+) (\d+) (ESTABLISHED|SYN_SENT|SYN_RECV|FIN_WAIT1|FIN_WAIT2|TIME_WAIT|CLOSE|CLOSE_WAIT|LAST_ACK|LISTEN|CLOSING|UNKNOWN) src=(.*) dst=(.*) sport=(\d+) dport=(\d+) src=(.*) dst=(.*) sport=(\d+) dport=(\d+) \[(ASSURED|UNREPLIED)\] mark=(\d) zone=(\d) use=(\d)").unwrap();

    let mut conntracks = Vec::new();
    for cap in re.captures_iter(s.trim()) {
        if cap.len() == 0 {
            return Err(NetworkError::ParserError)
        }

        let network_protocol = match cap.at(1).unwrap() {
            "ipv4" => NetworkProtocol::Ipv4,
            "ipv6" => NetworkProtocol::Ipv6,
            _ => NetworkProtocol::Unknown
        };

        let protocol = match cap.at(3).unwrap() {
            "tcp" => Family::Tcp,
            "udp" => Family::Udp,
            _ => Family::Unknown
        };

        let state = match cap.at(6).unwrap() {
            "ESTABLISHED" => ConntrackState::Establised,
            "SYN_SENT" => ConntrackState::SynSent,
            "SYN_RECV" => ConntrackState::SynRecv,
            "FIN_WAIT1" => ConntrackState::FinWait1,
            "FIN_WAIT2" => ConntrackState::FinWait2,
            "TIME_WAIT" => ConntrackState::TimeWait,
            "CLOSE" => ConntrackState::Close,
            "CLOSE_WAIT" => ConntrackState::CloseWait,
            "LAST_ACK" => ConntrackState::LastAck,
            "LISTEN" => ConntrackState::Listen,
            "CLOSING" => ConntrackState::Closing,
            _ => ConntrackState::Unknown,
        };

        let src_ip_string = cap.at(7).unwrap_or("");

        let dst_ip_string = cap.at(8).unwrap_or("");

        let src_ip: IpAddr = match FromStr::from_str(&src_ip_string) {
            Ok(v) => v,
            Err(why) => panic!("Error: {}", why.description())
        };
        let dst_ip: IpAddr = match FromStr::from_str(&dst_ip_string) {
            Ok(v) => v,
            Err(why) => panic!("Error: {}", why.description())
        };

        let src_port: u16 = match cap.at(9).unwrap_or("").parse::<u16>() {
            Ok(v) => v,
            Err(why) => panic!("Error: {}", why.description())
        };

        let dst_port: u16 = match cap.at(10).unwrap_or("").parse::<u16>() {
            Ok(v) => v,
            Err(why) => panic!("Error: {}", why.description())
        };

        let src = SocketAddr::new(src_ip, src_port);
        let dst = SocketAddr::new(dst_ip, dst_port);

        let network_protocol_number: i32 = match cap.at(2).unwrap().parse::<i32>() {
            Ok(v) => v,
            Err(why) => panic!("Error: {}", why.description())
        };
        let protocol_number: i32 = match cap.at(4).unwrap().parse::<i32>() {
            Ok(v) => v,
            Err(why) => panic!("Error: {}", why.description())
        };
        let timeout: i32 = match cap.at(5).unwrap().parse::<i32>() {
            Ok(v) => v,
            Err(why) => panic!("Error: {}", why.description())
        };

        let conntrack: Conntrack = Conntrack {
            network_protocol: network_protocol,
            network_protocol_number: network_protocol_number,
            protocol: protocol,
            protocol_number: protocol_number,
            timeout: timeout,
            src: src,
            dst: dst,
            state: state
        };
        conntracks.push(conntrack);
    }
    return Ok(conntracks);
}

pub fn collect_network_data() {
    let path = Path::new("/proc/net");
    if path.exists() {
        let conntracks = match self::get_conntrack() {
            Err(v) => panic!("Error :("),
            Ok(v) => v
        };

        println!("Found {} connections", conntracks.len());

        for conntrack in &conntracks {
            println!("\t{:?}", conntrack);
        }

        println!("");
    }
}
