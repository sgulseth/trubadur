use std::error::Error;
use std::io::prelude::*;
use std::fs::File;
use std::net::{IpAddr};

use toml;
use toml::Value;

pub fn ip_to_string(ip: IpAddr) -> String {
    let ip_as_string = match ip {
        IpAddr::V4(address) => {
            let octets: Vec<u8> = address.octets().to_vec();
            let ip: Vec<String> = octets.iter().map(|int| { int.to_string() }).collect();
            ip.join(".")
        },
        IpAddr::V6(address) => String::from("Ipv6 not supported")
    };

    return ip_as_string;
}

pub fn load_config(path: String) -> Value {
    let mut f = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", path, why.description()),
        Ok(file) => file,
    };

    // read the whole file
    let mut config = String::new();
    match f.read_to_string(&mut config) {
        Err(why) => panic!("couldn't read file {}: {}", path, why.description()),
        Ok(content) => content
    };
    let mut parser = toml::Parser::new(&config);

    let toml = match parser.parse() {
        Some(toml) => toml,
        None => {
            for err in &parser.errors {
                let (loline, locol) = parser.to_linecol(err.lo);
                let (hiline, hicol) = parser.to_linecol(err.hi);
                println!("{}:{}:{}-{}:{} error: {}",
                         path, loline, locol, hiline, hicol, err.desc);
            }
            panic!("Error parsing config")
        }
    };

    Value::Table(toml)
}