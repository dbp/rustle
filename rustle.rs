extern mod std;
use std::json;
use std::json::*;
use std::map;
use std::map::HashMap;
use std::sort;

use types::*;

fn main() {
    let args = os::args();

    if args.contains(&~"-h") || args.contains(&~"--help") {
        // usage();
        return;
    }

    // load in data
    let data = load::load(path::from_str("rustle.data"));

    // build query
    let querys = query::query(args[1]);

    // search!
    query::search(querys, data);
}