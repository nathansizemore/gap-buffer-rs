// Copyright 2017 Nathan Sizemore <nathanrsizemore@gmail.com>
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, you can
// obtain one at http://mozilla.org/MPL/2.0/.


extern crate gap_buffer;
extern crate gapbuffer;
extern crate rand;
extern crate scribe;


use std::time::Instant;

use rand::Rng;
use scribe::buffer::Position;


const INSERTS: usize = 10000;


struct BenchResult {
    _type: String,
    method: String,
    secs: u32,
    nanos: u32
}


fn main() {
    let data = "jncxjnfcsuho4t398u7r43duhmfrcmcsflkjmncfjmnlkrgcd7hioje";
    let mut results = Vec::<BenchResult>::new();

    insert_gapbuffer(data, &mut results);
    insert_scribe(data, &mut results);
    insert_gap_buffer(data, &mut results);

    print_results(results);
}

fn insert_gapbuffer(data: &str, results: &mut Vec<BenchResult>) {
    let mut rng = rand::thread_rng();
    let mut buf = gapbuffer::GapBuffer::<u8>::new();

    let mut max_index: usize = 0;

    let start = Instant::now();
    for _ in 0..INSERTS {
        let index = rng.gen_range::<usize>(0, max_index + 1);
        let bytes = data.as_bytes();

        for x in 0..bytes.len() {
            buf.insert(index + x, bytes[x]);
        }

        max_index += data.len() - 1;
    }
    let duration = start.elapsed();

    results.push(BenchResult {
        _type: "gapbuffer::GapBuffer".to_owned(),
        method: "insert".to_owned(),
        secs: duration.as_secs() as u32,
        nanos: duration.subsec_nanos()
    });
}

fn insert_scribe(data: &str, results: &mut Vec<BenchResult>) {
    let mut rng = rand::thread_rng();
    let mut buf = scribe::Buffer::new();

    let mut max_index: usize = 0;

    let start = Instant::now();
    for _ in 0..INSERTS {
        let index = rng.gen_range::<usize>(0, max_index + 1);
        buf.cursor.move_to(Position { line: 0, offset: index });
        buf.insert(data);
        max_index += data.len() - 1;
    }
    let duration = start.elapsed();

    results.push(BenchResult {
        _type: "scribe::Buffer".to_owned(),
        method: "insert".to_owned(),
        secs: duration.as_secs() as u32,
        nanos: duration.subsec_nanos()
    });
}

fn insert_gap_buffer(data: &str, results: &mut Vec<BenchResult>) {
    let mut rng = rand::thread_rng();
    let mut buf = gap_buffer::GapBuffer::with_capacity(1);

    let mut max_index: usize = 0;

    let start = Instant::now();
    for _ in 0..INSERTS {
        let index = rng.gen_range::<usize>(0, max_index + 1);
        buf.insert_str(index, data);
        max_index += data.len() - 1;
    }
    let duration = start.elapsed();

    results.push(BenchResult {
        _type: "gap_buffer::GapBuffer".to_owned(),
        method: "insert_str".to_owned(),
        secs: duration.as_secs() as u32,
        nanos: duration.subsec_nanos()
    });
}

fn print_results(results: Vec<BenchResult>) {
    let (type_len, method_len) = get_lens(&results);

    for result in results {
        let mut line = String::new();
        line.push_str(&format!("{:1$}    ", result._type, type_len));
        line.push_str(&format!("{:1$}    ", result.method, method_len));
        line.push_str(&format!("{:02}.", result.secs));
        line.push_str(&format!("{:010}", result.nanos));
        println!("{}", line);
    }
}

fn get_lens(results: &Vec<BenchResult>) -> (usize, usize) {
    let mut type_len = 0;
    let mut method_len = 0;

    for result in results {
        if result._type.len() > type_len {
            type_len = result._type.len();
        }

        if result.method.len() > method_len {
            method_len = result.method.len();
        }
    }

    (type_len, method_len)
}
