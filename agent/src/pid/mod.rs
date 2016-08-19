#![allow(dead_code)]
use std::error::Error;
use std::str;
use std::io::prelude::*;
use std::fs::File;

use regex::Regex;

#[derive(Debug)]
pub enum PidState {
    Running,
    Sleeping,
    SleepingUninterruptibleWait,
    Zombie,
    Unknown
}

#[derive(Debug)]
pub struct PidStat {
    pid: String,
    executable_name: String,
    state: PidState,
    uid: i32,
    gid: i32
}

#[derive(Debug)]
pub enum PidError {
    ParserError
}

pub fn get_stat(pid: &str) -> Result<PidStat, PidError>  {
    let path = format!("/proc/{}/stat", pid);
    let mut f = match File::open(path) {
        Err(why) => panic!("couldn't open {} stat: {}", pid, why.description()),
        Ok(file) => file,
    };

    // read the whole file
    let mut s = String::new();
    match f.read_to_string(&mut s) {
        Err(why) => panic!("couldn't read {}: {}", pid, why.description()),
        Ok(content) => content
    };

    // Todo: Add more matches, https://www.redhat.com/archives/axp-list/2001-January/msg00355.html
    let re: Regex = Regex::new(r"^(?P<pid>\d+) \((?P<exec>.*)\) (?P<state>(S|R|D|Z)) (?P<user_id>\d+) (?P<group_id>\d+)").unwrap();

    let cap = re.captures(s.trim()).unwrap();

    let exec: &str = cap.name("exec").unwrap_or("");

    let state = match cap.name("state").unwrap() {
        "S" => PidState::Sleeping,
        "R" => PidState::Running,
        "D" => PidState::SleepingUninterruptibleWait,
        "Z" => PidState::Zombie,
        _ => PidState::Unknown
    };

    let user_id: i32 = match cap.name("user_id").unwrap_or("").parse::<i32>() {
        Ok(v) => v,
        Err(why) => panic!("Error: {}", why.description())
    };

    let group_id: i32 = match cap.name("group_id").unwrap_or("").parse::<i32>() {
        Ok(v) => v,
        Err(why) => panic!("Error: {}", why.description())
    };

    let pid_stats: PidStat = PidStat {
        pid: pid.to_string(),
        executable_name: exec.to_string(),
        state: state,
        uid: user_id,
        gid: group_id
    };

    return Ok(pid_stats);
}
