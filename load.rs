//! This file contains code related to loading data from disk

use parse::*;

// load parses a json file with all the data into the in-memory
// representation above
pub fn load(path: path::Path) -> Data {
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
    let str_cast : fn(Json) -> ~str = |j| {
        match j { String(s) => s,
                  _ => fail ~"non-string" }
        };
    match *obj {
        Object(object) => {
            let ty = str_cast(object.get(&~"type"));
            let (args, rv) = load_args(ty);
            Definition { name: str_cast(object.get(&~"name")),
                         path: str_cast(object.get(&~"path")),
                         anchor: str_cast(object.get(&~"anchor")),
                         desc: str_cast(object.get(&~"desc")),
                         args: args,
                         ret: rv,
                         signature: ty }
        }
        _ => {
            io::println("json definitions must be objects");
            libc::exit(1);
            fail;
        }
    }
}

// load_args takes a string of a function and returns a list of the
// argument types, and the return type
fn load_args(s: ~str) -> (~[Arg], Arg) {
    let arg_str = trim_parens(s);
    let ret = load_ret(s);
    let args;
    if str::len(arg_str) == 0 {
        args = ~[];
    } else {
        let arg_strs = vec::map(split_arguments(arg_str), |a| {
            str::splitn_char(*a, ':', 1)[1]
        });
        args = vec::map(arg_strs, |x| { parse_arg(&trim_sigils(*x)) } );
    }
    return canonicalize_args(args, ret);
}

// load_ret takes a string of a function and returns the return value
fn load_ret(s: ~str) -> Arg {
    let st = str::split_str(s, "-> ");
    if vec::len(st) == 1 {
        Arg { name: ~"()", inner: ~[] }
    } else {
        parse_arg(&str::trim(st[1]))
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
