use std::fs::File;
use std::io::{Write, Read};
use bincode::{serialize, deserialize, Infinite};

use nnet::*;

type Agent = (FlatNetwork, u32);
type Data = (usize, Vec<Agent>);

pub fn load() -> Option<Data> {
    File::open("last_gen.net").ok()
        .and_then(|mut file| {
            let mut buf = Vec::new();
            file.read_to_end(&mut buf).ok().map(|_| buf)
        })
        .and_then(|buf| {
            deserialize(buf.as_slice()).ok()
        })
}

pub fn save(generation: usize, agents: Vec<Agent>) {
    let encoded: Vec<u8> = serialize(&(generation, agents), Infinite).expect("serialize");
    let mut file = File::create("last_gen.net").expect("create");
    file.write_all(encoded.as_slice()).expect("write_all");
}
