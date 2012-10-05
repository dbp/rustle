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