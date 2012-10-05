extern mod std;
use std::json;
use std::json::*;
use std::map;
use std::map::HashMap;
use std::sort;

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

// an Arg is a name, like str or Option, and then an optional list
// of parameters. ex: Option<T> is "Option", ["T"] (roughly).
struct Arg { name: ~str, inner: ~[Arg] }

impl Arg : cmp::Eq {
    pure fn eq(other: &Arg) -> bool { 
        (self.name == other.name) && (self.inner == other.inner)
    }
    pure fn ne(other: &Arg) -> bool {
        (self.name != other.name) || (self.inner != other.inner)
    }
}

// a query is a collection of definitions, ordered from most specific
// to most general. they should all match something the user could be 
// looking for.
struct Query { args: ~[Arg], ret: Arg }

// a Definition is what we are trying to match against. Note that 
// definitions are not exactly unique, as they can be made more specific 
// (ie, A,B -> C can be A A -> B, etc)
struct Definition { name: ~str, path: ~str, anchor: ~str, desc: ~str, 
                    args: ~[Arg], ret: Arg, signature: ~str }

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
        let argst = vec::map(str::split_char(arg_str, ','), 
                            |x| {
                                str::trim(str::split_char(*x, ':')[1])
                            });
        args = vec::map(argst, |x| { parse_arg(&trim_sigils(*x)) } );
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

// parse_arg takes a string and turns it into an Arg
fn parse_arg(s: &~str) -> Arg {
    let ps = str::split_char(str::trim(*s), '<');
    if vec::len(ps) == 1 {
        // non-parametrized type
        return Arg { name: copy *s, inner: ~[] };
    } else {
        let params = str::split_char(str::split_char(ps[1], '>')[0],',');
        return Arg { name: ps[0], inner: vec::map(params, parse_arg)};
    }
}

// canonicalize_args takes a list of arguments and a return type 
// and replaces generic names consistently (alphabetically, single 
// uppercase letters, in order of frequency)
fn canonicalize_args(args: ~[Arg], ret: Arg) -> (~[Arg],Arg) {
    // The basic process is as follows:
    // 1. identify and count polymorphic params
    // 2. sort and assign new letters to them
    // 3. replace names

    let identifiers : HashMap<~str, uint> = HashMap();
    fn walk_ids(a: Arg, m: &HashMap<~str, uint>) {
        match m.find(a.name) {
            None => {
                if str::len(a.name) == 1 {
                    m.insert(a.name, 1);
                } else {
                    // not counting other types currently
                }
            }
            Some(n) => { m.insert(a.name, n+1); }
        }
        vec::map(a.inner, |x| { walk_ids(*x, m) } );
    }
    // identify / count parameters
    vec::map(args, |a| { walk_ids(*a,&identifiers) } );
    walk_ids(ret, &identifiers);
    // put them in a vec
    let mut identifiers_vec : ~[(~str, uint)] = ~[];
    for identifiers.each |i,c| {
        identifiers_vec.push((i,c));
    }
    // sort the vec
    let identifiers_sorted = 
        sort::merge_sort(|x,y| { x.second() >= y.second() }, identifiers_vec);
    // new name assignments
    let names : HashMap<~str,~str> = HashMap();
    let letters = str::chars("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    let mut n = 0;
    for vec::each(identifiers_sorted) |p| {
        names.insert(p.first(), str::from_char(letters[n]));
        n += 1;
    }
    // now rename args
    fn rename_arg(a: Arg, n: &HashMap<~str,~str>) -> Arg {
        if n.contains_key(a.name) {
            Arg { name: n.get(a.name), 
                  inner: vec::map(a.inner, |x| { rename_arg(*x, n) })}
        } else {
            Arg { name: a.name, 
                  inner: vec::map(a.inner, |x| { rename_arg(*x, n) })}
        }
    }
    return (vec::map(args, |a| { rename_arg(*a, &names) }), 
            rename_arg(ret,&names));
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
    let rv = if vec::len(parts) < 2 { 
        Arg { name: ~"()", inner: ~[] } 
    } else { 
        parse_arg(&parts[1]) 
    };
    let ars = vec::map(str::split_char(trim_parens(parts[0]), ','), 
                       |x| { parse_arg(&trim_sigils(*x)) });
    // just one for now
    let (args, ret) = canonicalize_args(ars, rv);
    ~[Query {args: args, ret: ret}]
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
            io::println(fmt!("%s::%s: %s - %s", d.path, d.name, 
                             d.signature, d.desc));
        }
    }
}

// trim_sigils trims off the sigils off of types
fn trim_sigils(s: ~str) -> ~str {
    str::trim_left_chars(s, &[' ', '&', '~', '@', '+'])
}

fn trim_parens(s: ~str) -> ~str {
    str::trim(str::split_char(str::split_char(s, '(')[1], ')')[0])
}


#[cfg(test)]
mod tests {

    #[test]
    fn arg_eq() {
        assert Arg { name: ~"uint", inner: ~[] } == Arg { name: ~"uint", inner: ~[] };
        assert !(Arg { name: ~"uint", inner: ~[Arg { name: ~"A", inner: ~[] }] } 
                 == Arg { name: ~"uint", inner: ~[] });
        assert !(Arg { name: ~"uint", inner: ~[Arg { name: ~"A", inner: ~[] }] } 
                 == Arg { name: ~"uint", inner: ~[Arg { name: ~"B", inner: ~[] }] });
    }

    // #[test]
    // fn test_canonicalize_args() {
    //     assert canonicalize_args(~[~"str", ~"uint", ~"str"]) 
    //            == ~[~"str", ~"uint", ~"str"];
    //     assert canonicalize_args(~[~"str", ~"T", ~"str"]) 
    //            == ~[~"str", ~"A", ~"str"];
    //     assert canonicalize_args(~[~"str", ~"T", ~"T"]) 
    //            == ~[~"str", ~"A", ~"A"];
    //     assert canonicalize_args(~[~"U", ~"T", ~"T"]) 
    //            == ~[~"B", ~"A", ~"A"];
    //     assert canonicalize_args(~[~"U", ~"T", ~"V"]) 
    //            == ~[~"A", ~"B", ~"C"];
    // }

    // #[test]
    // fn test_parameterized_args() {
    //     assert canonicalize_args(~[~"Option<T>", ~"T"]) == ~[~"Option<A>", ~"A"];
    //     assert canonicalize_args(~[~"~[T]", ~"T"]) == ~[~"~[A]", ~"A"];
    // }

    #[test]
    fn test_trim_parens() {
        assert trim_parens(~"(hello)") == ~"hello";
        assert trim_parens(~"  (   hello)") == ~"hello";
        assert trim_parens(~"  (   hello )  ") == ~"hello";
    }

    #[test]
    fn test_trim_sigils() {
        assert trim_sigils(~"~str") == ~"str";
        assert trim_sigils(~"@str") == ~"str";
        assert trim_sigils(~"str") == ~"str";
        assert trim_sigils(~"& str") == ~"str";
    }
}