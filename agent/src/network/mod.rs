use std::net::IpAddr;
use std::fmt;
use std::error;

use pnet::packet::PacketSize;
use pnet::packet::Packet;
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ip::{IpNextHeaderProtocol, IpNextHeaderProtocols};
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::ipv6::Ipv6Packet;
use pnet::packet::udp::UdpPacket;
use pnet::packet::tcp::TcpPacket;
use pnet::datalink::{self, NetworkInterface};

use influent::client::http::HttpClient;

pub struct Proto {
    pub protocol: String,
    pub packet_size: usize,
    pub source: IpAddr,
    pub source_port: u16,
    pub destination: IpAddr,
    pub destination_port: u16
}

#[derive(Debug)]
pub enum SnifferError {
    HandlePacketError,
    UnknownPacketProtocol,
    Ipv4PacketError,
    Ipv6PacketError
}

impl fmt::Display for SnifferError {
    // TODO: Provide information about the parameter format expected when
    // displaying malformed parameter errors
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::SnifferError::*;

        match *self {
            HandlePacketError => write!(f, "Error: Couldnt handle packet "),
            UnknownPacketProtocol => write!(f, "Error: Unkown packet protocol "),
            Ipv4PacketError => write!(f, "Error: Couldnt handle ipv4 packet "),
            Ipv6PacketError => write!(f, "Error: Couldnt handle ipv6 packet "),
        }
    }
}

impl error::Error for SnifferError {
    fn description(&self) -> &str {
        use self::SnifferError::*;

        match *self {
            HandlePacketError => "Error: Couldnt handle packet ",
            UnknownPacketProtocol => "Error: Unkown packet protocol",
            Ipv4PacketError => "Error: Couldnt handle ipv4 packet ",
            Ipv6PacketError => "Error: Couldnt handle ipv6 packet ",
        }
    }
}


fn handle_transport_protocol(source: IpAddr, destination: IpAddr, protocol: IpNextHeaderProtocol, packet: &[u8]) -> Option<Proto> {
    let proto = match protocol {
        IpNextHeaderProtocols::Udp  => {
            let udp = UdpPacket::new(packet);
            if let Some(udp) = udp {
                let proto = Proto {
                    protocol: "udp".to_string(),
                    packet_size: udp.packet_size(),
                    source: source,
                    source_port: udp.get_source(),
                    destination: destination,
                    destination_port: udp.get_destination()
                };

                return Some(proto);
            }

            return None;
        },
        IpNextHeaderProtocols::Tcp  => {
            let tcp = TcpPacket::new(packet);
            if let Some(tcp) = tcp {
                let proto = Proto {
                    protocol: "tcp".to_string(),
                    packet_size: tcp.packet_size(),
                    source: source,
                    source_port: tcp.get_source(),
                    destination: destination,
                    destination_port: tcp.get_destination()
                };

                return Some(proto);
            }

            return None;
        },
        _ => None
    };

    return proto;
}

fn handle_ipv4_packet(ethernet: &EthernetPacket) -> Option<Proto> {
    let header = Ipv4Packet::new(ethernet.payload());
    if let Some(header) = header {
        return handle_transport_protocol(IpAddr::V4(header.get_source()),
                                  IpAddr::V4(header.get_destination()),
                                  header.get_next_level_protocol(),
                                  header.payload());
    }

    return None;
}

fn handle_ipv6_packet(ethernet: &EthernetPacket) -> Option<Proto> {
    let header = Ipv6Packet::new(ethernet.payload());
    if let Some(header) = header {
        return handle_transport_protocol(IpAddr::V6(header.get_source()),
                                  IpAddr::V6(header.get_destination()),
                                  header.get_next_header(),
                                  header.payload());
    }

    return None;
}


fn handle_packet(ethernet: &EthernetPacket) -> Option<Proto> {
    match ethernet.get_ethertype() {
        EtherTypes::Ipv4 => handle_ipv4_packet(ethernet),
        EtherTypes::Ipv6 => handle_ipv6_packet(ethernet),
        _  => None
    }
}

pub fn capture(iface_name: &str, save: &Fn(&Proto, &HttpClient), influxdb_client: &HttpClient) {
    use pnet::datalink::Channel::Ethernet;

    let interface_names_match = |iface: &NetworkInterface| iface.name == iface_name;

    // Find the network interface with the provided name
    let interfaces = datalink::interfaces();
    let interface = interfaces.into_iter()
                              .filter(interface_names_match)
                              .next()
                              .unwrap();

    // Create a channel to receive on
    let (_, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("packetdump: unhandled channel type: {}"),
        Err(e) => panic!("packetdump: unable to create channel: {}", e),
    };

    let mut iter = rx.iter();

    loop {
        let packet_data = match iter.next() {
            Ok(packet) => packet,
            _ => panic!("packetdump: unable to iterage packet")
        };

        let packet = handle_packet(&packet_data);

        if let Some(packet) = packet {
            save(&packet, influxdb_client);
        }
    }
}
