use time;

use std::net::IpAddr;

use pnet::packet::PacketSize;
use pnet::packet::Packet;
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ip::{IpNextHeaderProtocol, IpNextHeaderProtocols};
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::ipv6::Ipv6Packet;
use pnet::packet::udp::UdpPacket;
use pnet::packet::tcp::TcpPacket;

use pnet::datalink::{self, NetworkInterface};

pub struct Proto {
    pub protocol: String,
    pub packet_size: usize,
    pub source: IpAddr,
    pub source_port: u16,
    pub destination: IpAddr,
    pub destination_port: u16
}

pub enum SnifferError {
    HandlePacketError,
    Ipv4PacketError,
    Ipv6PacketError
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


fn handle_packet(ethernet: &EthernetPacket) -> Result<Option<Proto> , SnifferError> {
    let packet = match ethernet.get_ethertype() {
        EtherTypes::Ipv4 => Ok(handle_ipv4_packet(ethernet)),
        EtherTypes::Ipv6 => Ok(handle_ipv6_packet(ethernet)),
        _  => Err(SnifferError::HandlePacketError)
    };

    return packet;
}

pub fn start(iface_name: &str, capture_duration: i64) -> Vec<Proto> {
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

    let mut data = Vec::new();
    let mut iter = rx.iter();
    let mut done = false;
    let end = time::get_time() + time::Duration::seconds(capture_duration);

    while !done {
        println!("Collecting...");
        let packet_data = match iter.next() {
            Ok(packet) => {
                match handle_packet(&packet) {
                    Ok(v) => v,
                    Err(e) => panic!("packetdump: unable to receive packet")
                }
            },
            _ => panic!("packetdump: unable to receive packet")
        };
        println!("Got packet");
        if let Some(packet_data) = packet_data {
            data.push(packet_data);
        }

        let now = time::get_time();
        if now.gt(&end) {
            done = true;
        }
    }

    return data;
}
