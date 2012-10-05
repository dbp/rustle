extern mod std;
use std::json;
use std::json::*;
use std::map;
use std::map::HashMap;
use std::sort;
use io::ReaderUtil;

use types::*;

fn main() {
    let args = os::args();

    if args.contains(&~"-h") || args.contains(&~"--help") {
        usage();
        return;
    }
    // load in data
    let data = load::load(path::from_str("rustle.data"));

    if args.len() == 1 {
        // start interactive loop
        io::println(~"Rustle, a Rust api search, by type signature, v. 0.1.");
        io::println(~"Type in a type signature, like \"Option<A> -> A\". Ctrl-D to quit");
        loop {
            let stdin = io::stdin();

            io::print("rustle> ");
            let raw = (stdin as ReaderUtil).read_line();
            if str::is_empty(raw) {
                if stdin.eof() {
                    io::println("");
                    break;
                }
                loop;
            }
            // build query
            let queries = query::query(str::trim(raw));

            // search
            query::search(queries, data);
        }
    } else {
        // single run
        let queries = query::query(args[1]);
        query::search(queries, data);
    }
}

fn usage() {
    io::println(~"Rustle, a Rust api search, by type signature, v. 0.1.");
    io::println(~"Usage: rustle -h | --help            -- this message");
    io::println(~"       rustle                        -- start interactive mode");
    io::println(~"       rustle \"[(A,B)] -> ([A],[B])\"     -- query directly");
}