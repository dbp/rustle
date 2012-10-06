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
    let (args, ret, l) = canonicalize_args(ars, rv);
    // now create more general variants
    let mut queries = ~[Query {args: args, ret: ret}];

    generalize_queries(args,ret,l,&mut queries);
    return queries;
}

// search_type looks for matches from the query in the data, and prints out
// what it finds
pub fn search_type(qs: ~[Query], d: &Data) {
    for qs.each |q| {
        match vec::len(q.args) {
            0 => search_bucket(d.ar0, *q),
            1 => search_bucket(d.ar1, *q),
            2 => search_bucket(d.ar2, *q),
            3 => search_bucket(d.ar3, *q),
            4 => search_bucket(d.ar4, *q),
            5 => search_bucket(d.ar5, *q),
            _ => search_bucket(d.arn, *q)
        }
    }
}

// search_name looks for a function by name, prefix only
pub fn search_name(q: ~str, d: &Data) {
    let mut name = q;
    search_trie(d.names, &mut name, q);
}

// search_bucket looks for matches in a bucket
fn search_bucket(b: Bucket, q: Query) {
    for vec::each(b.defs) |d| {
        if d.args == q.args && d.ret == q.ret {
            io::println(d.to_str());
        }
    }
}

// search_trie looks for matching defitions by name
fn search_trie(t: @Trie, n: &mut ~str, q: ~str) {
    fn find_defs(t: @Trie, q: ~str) {
        // go through everything at this level, and any deeper
        for vec::each(t.definitions) |d| {
            // to avoid borrow errors - eek! FIXME!
            unsafe {
                if d.name.contains(q) {
                    io::println(d.to_str());
                }
            }
        }
        for t.children.each_value |c| {
            find_defs(c, q);
        }
    }
    if n.len() == 0 {
        // at level, look for definition
        find_defs(t, q);
    } else {
        // look deeper, if we can
        let c = str::from_char(str::shift_char(n));
        match t.children.find(c) {
            None => return,
            Some(child) => search_trie(child, n, q)
        }
    }
}

// generalize_queries creates more general versions of queries
// by replacing contrete types with polymorphic variables
// note that how we are doing it now, it will generate (lots of) duplicate
// queries. l is the next available polymorphic variable letter
fn generalize_queries(args: ~[Arg], ret: Arg, l: uint, q: &mut ~[Query]) {
    let arg_names = HashMap();
    fn get_arg_names(a: Arg, n: &HashMap<~str,()>) {
        n.insert(a.name,());
        vec::map(a.inner, |a| {get_arg_names(*a, n)});
    }
    vec::map(args, |a| {get_arg_names(*a,&arg_names)});
    get_arg_names(ret,&arg_names);
    // now for all that aren't polymorphic, make them and
    // search recursively. note that t
    for arg_names.each_key |n| {
        if n.len() != 1 && n != ~"[]" && n != ~"()" {
            let nn = letters()[l];
            let nargs = vec::map(args, |a| { replace_arg_name(*a,n,nn) });
            let nret = replace_arg_name(ret,n,nn);
            q.push(Query {args: nargs, ret: nret});
            generalize_queries(nargs, nret, l+1, q);
        }
    }
}