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
                    let defs = vec::concat(vec::map(lst, load_obj));
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

// load_obj loads a single object into a set of Definitions, or fails if the json
// is not well formed
fn load_obj(obj: &Json) -> ~[(@Definition, bool)] {
    fn str_cast(j: Json) -> ~str {
        match j { String(s) => s,
                  _ => fail ~"non-string" }
    }
    let mut definitions;
    match *obj {
        Object(object) => {
            let ty = str_cast(object.get(&~"type"));
            let self = match str_cast(object.get(&~"self")) {
                ~"" => None, s => Some(s)
            };
            let (args, rv, l) = load_args(ty,self);
            let canonical =
                @Definition { name: str_cast(object.get(&~"name")),
                              path: str_cast(object.get(&~"path")),
                              anchor: str_cast(object.get(&~"anchor")),
                              desc: str_cast(object.get(&~"desc")),
                              args: args,
                              ret: rv,
                              signature: ty };
            definitions = ~[(canonical,true)];
            if l > 1 {
                // generate variants. for now, we just generate one where
                // all the type variables are the same. the general case has
                // exponential variations, and furthermore this type of
                // solution wouldn't make sense. This should cover most
                // of the cases without getting too crazy.
                let mut n = 1;
                let mut vargs = args;
                let mut ret = rv;
                while n < l {
                    let zl = letters()[0];
                    let nl = letters()[n];
                    vargs = vec::map(vargs, |a| {
                        replace_arg_name(*a, nl, zl)
                    });
                    ret = replace_arg_name(ret, nl, zl);
                    n += 1;
                }
                definitions.push((@Definition {args: vargs,
                                             ret: ret,
                                             ..*canonical}, false));
            }
        }
        _ => {
            io::println("json definitions must be objects");
            libc::exit(1);
            fail;
        }
    }
    return definitions;
}

// load_args takes a string of a function and returns a list of the
// argument types, and the return type
fn load_args(arg_list: ~str, self: Option<~str>) -> (~[Arg], Arg, uint) {
    let self_list = match self {
        None => ~[],
        Some(s) => ~[parse_arg(&trim_sigils(s), s)]
    };
    let args_ret = str::split_str(arg_list, "->");
    let mut arlen = vec::len(args_ret);
    let ret;
    if vec::len(args_ret) == 1 {
        arlen += 1;
        ret = Arg { name: ~"()", inner: ~[] };
    } else {
        ret = parse_arg(&str::trim(args_ret[arlen-1]), arg_list);
    }
    let arg_str =
        trim_parens(str::connect(vec::view(args_ret, 0, arlen-1), "->"));
    let args;
    if str::len(arg_str) == 0 {
        args = ~[];
    } else {
        let arg_strs = vec::map(split_arguments(arg_str), |a| {
            let t = str::splitn_char(*a, ':', 1);
            if vec::len(t) < 2 {
                error!("%s", arg_list);
                error!("%s", arg_str);
                error!("%?", t);
            }
            t[1]
        });
        args = vec::map(arg_strs, |x| {
                parse_arg(&trim_sigils(*x), arg_list)
            } );
    }
    return canonicalize_args(vec::append(self_list,args), ret);
}

// bucket_sort takes definitions and builds the Data structure, by putting
// them into the appropriate buckets
fn bucket_sort(ds: ~[(@Definition, bool)]) -> Data {
    let data = empty_data();
    for vec::each(ds) |dc| {
        let (d, canonical) = *dc;
        match vec::len(d.args) {
            0 => bucket_drop(&data.ar0, d),
            1 => bucket_drop(&data.ar1, d),
            2 => bucket_drop(&data.ar2, d),
            3 => bucket_drop(&data.ar3, d),
            4 => bucket_drop(&data.ar4, d),
            5 => bucket_drop(&data.ar5, d),
            _ => bucket_drop(&data.arn, d)
        }
        if canonical {
            let mut name = d.name;
            add_name(data.names, &mut name, d);
        }
    }
    return data;
}

// bucket_drop places a definition into the right part of the bucket
fn bucket_drop(b: &Bucket, d: @Definition) {
    // for now, just put all in np0
    b.defs.push(d);
}

// add_name adds a definition to the trie, for prefix searching
fn add_name(t: @Trie, n: &mut ~str, d: @Definition) {
    if n.len() == 0 {
        // found where to put it
        t.definitions.push(d);
    } else {
        // move down a level
        let c = str::from_char(str::shift_char(n));
        let mut v;
        match t.children.find(c) {
            None => {
                // add a new branch
                v = @Trie { children: HashMap(), definitions: ~[] };
                t.children.insert(c, v);
            }
            Some(child) => {
                v = child;
            }
        }
        add_name(v, n, d);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_load_args() {
        assert load_args(~"fn ne(other: & ~str) -> bool",
                         Some(~"& str")) ==
                (~[Arg {name: ~"str", inner: ~[]},
                   Arg {name: ~"str", inner: ~[]}],
                 Arg {name: ~"bool", inner: ~[]},
                 0);
    }
    fn test_method_args() {
        assert load_args(~"fn ne(other: & Option<T>) -> bool",
                         Some(~"& str")) ==
                (~[Arg {name: ~"str", inner: ~[]},
                   Arg {name: ~"str", inner: ~[]}],
                 Arg {name: ~"bool", inner: ~[]},
                 0);
    }
}
