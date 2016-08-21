#![allow(unused_must_use)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![feature(ipv6_to_octets)]
#[macro_use]
extern crate lazy_static;
extern crate toml;
extern crate pnet;
extern crate influent;
mod network;
mod utils;

use std::process::exit;
use std::time::Duration;
use std::thread::{sleep};
use std::env;

use influent::create_client;
use influent::client::{Client, Credentials};
use influent::measurement::{Measurement, Value};
use influent::client::http::HttpClient;

fn save(packet: &network::Proto, influxdb_client: &HttpClient) {
    let protocol = packet.protocol.to_string();
    let source: String = utils::ip_to_string(packet.source);
    let dst: String = utils::ip_to_string(packet.destination);

    let source_port = packet.source_port.to_string();
    let destination_port = packet.destination_port.to_string();

    let mut measurement = Measurement::new("packets");
    measurement.add_tag("protocol", &protocol);
    measurement.add_tag("source", &source);
    measurement.add_tag("source_port", &source_port);
    measurement.add_tag("destination", &dst);
    measurement.add_tag("destination_port", &destination_port);
    measurement.add_field("size", Value::Integer(packet.packet_size as i64));

    influxdb_client.write_one(measurement, None);

    println!("Saving {} Packet: {}:{} > {}:{}; length: {}", packet.protocol, packet.source, packet.source_port, packet.destination, packet.destination_port, packet.packet_size);
}

fn main() {
    let mut args = env::args();
    if args.len() < 2 {
        println!("trubadur config.toml {}", args.len());
        exit(1);
    }

    let config = utils::load_config(args.nth(1).unwrap());

    // prepare influxdb config
    let database = config.lookup("influxdb.database").unwrap().as_str().unwrap();
    let username = config.lookup("influxdb.username").unwrap().as_str().unwrap();
    let password = config.lookup("influxdb.password").unwrap().as_str().unwrap();
    let hosts = config.lookup("influxdb.hosts").unwrap().as_slice().unwrap().iter().map(|val| val.as_str().unwrap()).collect();

    let credentials = Credentials {
        database: &database,
        username: &username,
        password: &password
    };
    let influxdb_client = create_client(credentials, hosts);

    loop {
        network::capture("eth0", &save, &influxdb_client);
        sleep(Duration::from_secs(1));
    }
}