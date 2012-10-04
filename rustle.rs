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

    io::println(fmt!("%?", data));

    // build query
    let query = query(vec::tail(args));

    // search!
    search(query, data);
}

// a query is a collection of definitions, ordered from most specific
// to most general. they should all match something the user could be 
// looking for.
struct Query { forms: ~[Definition] }

// a Definition is what we are trying to match against. Note that 
// definitions are not exactly unique, as they can be made more specific 
// (ie, A,B -> C can be A A -> B, etc)
struct Definition { name: ~str, path: ~str, anchor: ~str, desc: ~str, 
                    args: ~[~str], ret: ~str }

// A bucket holds a bunch of definitions, ordered by number of distinct 
// non-polymorphic types, then by number of distinct polymorphic types, 
// for completely polymorphic functions.
struct Bucket { np0: ~[Definition], np1: ~[Definition], np2: ~[Definition],
                npn: ~[Definition], p1: ~[Definition], p2: ~[Definition], 
                p3: ~[Definition], pn: ~[Definition] }

// Data stores all the definitions in buckets, based on function arity.
struct Data { ar0: Bucket, ar1: Bucket, ar2: Bucket, ar3: Bucket, 
              ar4: Bucket, ar5: Bucket, arn: Bucket }


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
                    let empty_bucket = Bucket { np0: ~[], np1: ~[], np2: ~[],
                                                npn: ~[], p1: ~[], p2: ~[], 
                                                p3: ~[], pn: ~[] };
                    let defs = vec::map(lst, load_obj);
                    Data { ar0: Bucket {np0: defs, .. empty_bucket}, ar1: empty_bucket, 
                           ar2: empty_bucket, ar3: empty_bucket, ar4: empty_bucket, 
                           ar5: empty_bucket, arn: empty_bucket}
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
    let arg_str = str::trim(str::split_char(str::split_char(s, 
                                                            '(')[1], 
                                                            ')')[0]);
    if str::len(arg_str) == 0 {
        return ~[];
    }
    let args = vec::map(str::split_char(arg_str, ','), 
                        |x| {
                            str::trim(str::split_char(*x, ':')[1])
                        });
    vec::map(args, |x| { str::trim_left_chars(*x, &[' ', '&', '~', '@'])})
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

// query builds a Query from whatever was passed in on the commandline
fn query(parts: ~[~str]) -> Query {
    fail;
}

// search looks for matches from the query in the data, and prints out
// what it finds
fn search(q: Query, d: Data) {
    fail;
}