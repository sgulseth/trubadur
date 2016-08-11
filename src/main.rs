#![allow(dead_code)]
#[macro_use] extern crate lazy_static;
extern crate regex;
extern crate psutil;
mod pid;

use std::time::Duration;
use std::thread::{sleep};

fn collect() {
    for p in psutil::process::all().unwrap().iter() {
        let pid = p.pid.to_string();

        pid::collect_process_network_data(&pid);
    }
}

fn main() {
    loop {
        collect();
        sleep(Duration::from_secs(10));
    }
}