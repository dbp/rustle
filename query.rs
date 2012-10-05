//! This file contains procedures related to searching data

use parse::*;

// query builds a Query from whatever was passed in on the commandline
pub fn query(q: ~str) -> ~[Query] {
    let parts = vec::map(str::split_str(q, "->"), |x| { str::trim(*x) });
    let rv = if vec::len(parts) < 2 {
        Arg { name: ~"()", inner: ~[] }
    } else {
        parse_arg(&parts[1], q)
    };
    let ars = vec::map(split_arguments(parts[0]),
                       |x| { parse_arg(&trim_sigils(*x), q) });
    // just one for now
    let (args, ret) = canonicalize_args(ars, rv);
    ~[Query {args: args, ret: ret}]
}

// search looks for matches from the query in the data, and prints out
// what it finds
pub fn search(qs: ~[Query], d: Data) {
    let q = qs[0];
    match vec::len(q.args) {
        0 => search_bucket(d.ar0, q),
        1 => search_bucket(d.ar1, q),
        2 => search_bucket(d.ar2, q),
        3 => search_bucket(d.ar3, q),
        4 => search_bucket(d.ar4, q),
        5 => search_bucket(d.ar5, q),
        _ => search_bucket(d.arn, q)
    }
}

// search_bucket looks for matches in a bucket
fn search_bucket(b: Bucket, q: Query) {
    for vec::each(b.np0) |d| {
        if d.args == q.args && d.ret == q.ret {
            io::println(fmt!("%s::%s: %s - %s", d.path, d.name,
                             d.signature, d.desc));
        }
    }
}
