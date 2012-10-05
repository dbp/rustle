extern mod std;
use std::json;
use std::json::*;

fn main() {
    let args = os::args();

    if args.contains(&~"-h") || args.contains(&~"--help") {
        // usage();
        return;
    }

    // load in data
    let data = load(path::from_str("rustle.data"));

    // build query
    let querys = query(args[1]);

    // search!
    search(querys, data);
}

// a query is a collection of definitions, ordered from most specific
// to most general. they should all match something the user could be 
// looking for.
struct Query { args: ~[~str], ret: ~str }

// a Definition is what we are trying to match against. Note that 
// definitions are not exactly unique, as they can be made more specific 
// (ie, A,B -> C can be A A -> B, etc)
struct Definition { name: ~str, path: ~str, anchor: ~str, desc: ~str, 
                    args: ~[~str], ret: ~str }

// A bucket holds a bunch of definitions, ordered by number of distinct 
// non-polymorphic types, then by number of distinct polymorphic types, 
// for completely polymorphic functions.
struct Bucket { mut np0: ~[Definition], mut np1: ~[Definition], 
                mut np2: ~[Definition], mut npn: ~[Definition], 
                mut p1: ~[Definition], mut p2: ~[Definition], 
                mut p3: ~[Definition], mut pn: ~[Definition] }

// Data stores all the definitions in buckets, based on function arity.
struct Data { mut ar0: Bucket, mut ar1: Bucket, mut ar2: Bucket, 
              mut ar3: Bucket, mut ar4: Bucket, mut ar5: Bucket, 
              mut arn: Bucket }

fn empty_data() -> Data {
    let empty_bucket = Bucket { np0: ~[], np1: ~[], np2: ~[],
                                npn: ~[], p1: ~[], p2: ~[], 
                                p3: ~[], pn: ~[] };
    Data { ar0: empty_bucket, ar1: empty_bucket, ar2: empty_bucket, 
           ar3: empty_bucket, ar4: empty_bucket, ar5: empty_bucket, 
           arn: empty_bucket}
}

// load parses a json file with all the data into the in-memory 
// representation above
fn load(path: path::Path) -> Data {
    let res;
    match io::file_reader(&path) {
        Err(msg) => {
            io::println(fmt!("file_reader err: %s", msg));
            libc::exit(1);
            fail;
        }
        Ok(file) => {
            res = json::from_reader(file);
        }
    }

    match res {
        Ok(json) => {
            match json {
                List(lst) => {
                    let defs = vec::map(lst, load_obj);
                    bucket_sort(defs)
                }
                _ => {
                    io::println("json not correctly formatted");
                    libc::exit(1);
                    fail;
                }
            }
        }
        Err(err) => {
            io::println(fmt!("parsing error in data on line %u, col %u",
                             err.line, err.col));
            libc::exit(1);
            fail;
        }
    }
}

// load_obj loads a single object into a Definition, or fails if the json 
// is not well formed
fn load_obj(obj: &Json) -> Definition {
    let str_cast : fn(Json) -> ~str = |j| { match j { String(s) => s, _ => fail ~"non-string" } };
    match *obj {
        Object(object) => {
            let ty = str_cast(object.get(&~"type"));
            Definition { name: str_cast(object.get(&~"name")),
                         path: str_cast(object.get(&~"path")),
                         anchor: str_cast(object.get(&~"anchor")),
                         desc: str_cast(object.get(&~"desc")),
                         args: load_args(ty),
                         ret: load_ret(ty) }
        }
        _ => {
            io::println("json definitions must be objects");
            libc::exit(1);
            fail;
        }
    }
}

// load_args takes a string of a function and returns a list of the
// argument types
fn load_args(s: ~str) -> ~[~str] {
    let arg_str = trim_parens(s);
    if str::len(arg_str) == 0 {
        return ~[];
    }
    let args = vec::map(str::split_char(arg_str, ','), 
                        |x| {
                            str::trim(str::split_char(*x, ':')[1])
                        });
    vec::map(args, |x| { trim_sigils(*x)} )
}

// load_ret takes a string of a function and returns the return value
fn load_ret(s: ~str) -> ~str {
    let st = str::split_str(s, "-> ");
    if vec::len(st) < 2 {
        ~"()"
    } else {
        str::trim(st[1])
    }
}

// bucket_sort takes defitions and builds the Data structure, by putting them
// into the appropriate buckets
fn bucket_sort(ds: ~[Definition]) -> Data {
    let data = empty_data();
    for vec::each(ds) |d| {
        match vec::len(d.args) {
            0 => bucket_drop(&data.ar0, d),
            1 => bucket_drop(&data.ar1, d),
            2 => bucket_drop(&data.ar2, d),
            3 => bucket_drop(&data.ar3, d),
            4 => bucket_drop(&data.ar4, d),
            5 => bucket_drop(&data.ar5, d),
            _ => bucket_drop(&data.arn, d)
        }
    }
    return data;
}

// bucket_drop places a definition into the right part of the bucket
fn bucket_drop(b: &Bucket, d: &Definition) {
    // for now, just put all in np0
    b.np0.push(copy *d);
}

// query builds a Query from whatever was passed in on the commandline
fn query(q: ~str) -> ~[Query] {
    let parts = vec::map(str::split_str(q, "->"), |x| { str::trim(*x) });
    let rv = if vec::len(parts) < 2 { ~"()" } else { parts[1] };
    let ars = vec::map(str::split_char(trim_parens(parts[0]), ','), |x| { trim_sigils(*x) });
    // just one for now
    ~[Query {args: ars, ret: rv}]
}

// search looks for matches from the query in the data, and prints out
// what it finds
fn search(qs: ~[Query], d: Data) {
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
            io::println(fmt!("%s::%s %s -> %s - %s", d.path, d.name, 
                             str::connect(d.args, ", "), d.ret, d.desc));
        }
    }
}

// trim_sigils trims off the sigils off of types
fn trim_sigils(s: ~str) -> ~str {
    str::trim_left_chars(s, &[' ', '&', '~', '@'])
}

fn trim_parens(s: ~str) -> ~str {
    str::trim(str::split_char(str::split_char(s, '(')[1], ')')[0])
}