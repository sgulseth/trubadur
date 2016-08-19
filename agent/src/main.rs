#![allow(unused_must_use)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![feature(ipv6_to_octets)]
extern crate time;
extern crate pnet;
extern crate influent;
mod network;
mod utils;
use std::time::Duration;
use std::thread::{sleep};

use influent::create_client;
use influent::client::{Client, Credentials};
use influent::measurement::{Measurement, Value};

use std::net::{IpAddr};

// prepare client

fn collect(packets: Vec<network::Proto>) {
    println!("Got {} packets", packets.len());
    let credentials = Credentials {
        database: "trubadur",
        username: "root",
        password: ""
    };
    let hosts = vec!["http://influxdb:8086"];
    let client = create_client(credentials, hosts);
    for packet in &packets {
        println!("{} Packet: {}:{} > {}:{}; length: {}", packet.protocol, packet.source, packet.source_port, packet.destination, packet.destination_port, packet.packet_size);

        let protocol = packet.protocol.to_string();
        let source: String = match packet.source {
            IpAddr::V4(address) => address.octets().iter().map(|int| { int as char }).collect().join("."),
            IpAddr::V6(address) => address.octets().iter().map(|int| { int as char }).collect().join(".")
        };

        let dst = match packet.destination {
            IpAddr::V4(address) => String::from("0.0.0.0"),
            IpAddr::V6(address) => String::from("0.0.0.0"),
        };

        let source_port = packet.source_port.to_string();
        let destination_port = packet.destination_port.to_string();

        let mut measurement = Measurement::new("packets");
        measurement.add_tag("protocol", &protocol);
        measurement.add_tag("source", &source);
        measurement.add_tag("source_port", &source_port);
        measurement.add_tag("destination", &dst);
        measurement.add_tag("destination_port", &destination_port);
        measurement.add_field("size", Value::Integer(packet.packet_size as i64));

        client.write_one(measurement, None);
    }
}

fn main() {
    let capture_duration: i64 = 2;
    loop {
        println!("Collecting packets for {} seconds", capture_duration);
        let packets: Vec<network::Proto> = network::start("eth0", capture_duration);
        collect(packets);
        sleep(Duration::from_secs(1));
    }
}