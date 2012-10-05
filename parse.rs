//! This file contains common functions used in parsing

use types::*;

// split arguments; note that because commas
// can appear in parametric types, ie Either<A,B>,
// we need to handle this a little more carefully
pub fn split_arguments(a: ~str) -> ~[~str] {
    let mut arg_strs : ~[~str] = ~[];
    let mut level = 0;
    let mut start = 0;
    // add a trailing comma so we pick up the last argument
    for str::each_chari(str::append(a, ~",")) |i,c| {
        match c {
            ',' if level == 0 => {
                arg_strs.push(str::trim(str::slice(a,start,i)));
                start = i+1;
            }
            '<' => level += 1,
            '>' => level -= 1,
            '(' => level += 1,
            ')' => level -= 1,
            _ => {}
        }
    }
    return arg_strs;
}

// parse_arg takes a string and turns it into an Arg
pub fn parse_arg(s: &~str, ctx: ~str) -> Arg {
    let ps = str::split_char(str::trim(*s), '<');
    if vec::len(ps) == 1 {
        // non-parametrized type, see if it's a vec
        let ps = str::split_char(str::trim(*s), '[');
        if vec::len(ps) == 1 {
            // normal type
            return Arg { name: copy *s, inner: ~[] };
        } else {
            // vector type
            // we drop any modifiers, like const, mut, etc.
            let va = str::words(str::trim(str::split_char(ps[1],']')[0]));
            if vec::len(va) == 0 {
                error!("%s", ctx);
            }
            return Arg { name: ~"[]",
                         inner: ~[parse_arg(&va[vec::len(va)-1], ctx)] };
        }
    } else {
        let params = split_arguments(str::split_char(ps[1], '>')[0]);
        return Arg { name: ps[0], inner: vec::map(params, |a| {parse_arg(a,ctx)})};
    }
}

// canonicalize_args takes a list of arguments and a return type
// and replaces generic names consistently (alphabetically, single
// uppercase letters, in order of frequency)
pub fn canonicalize_args(args: ~[Arg], ret: Arg) -> (~[Arg],Arg) {
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

// trim_sigils trims off the sigils off of types
pub fn trim_sigils(s: ~str) -> ~str {
    str::trim_left_chars(s, &[' ', '&', '~', '@', '+'])
}

pub fn trim_parens(s: ~str) -> ~str {
    let begin = option::get_default(&str::find_char(s,'('),-1)+1;
    let end = option::get_default(&str::rfind_char(s,')'), str::len(s));
    str::trim(str::slice(s, begin, end))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_canonicalize_args() {
        assert canonicalize_args(~[Arg {name: ~"str", inner: ~[]},
                                   Arg {name: ~"str", inner: ~[]}],
                                 Arg {name: ~"str", inner: ~[]})
               == (~[Arg {name: ~"str", inner: ~[]},
                     Arg {name: ~"str", inner: ~[]}],
                   Arg {name: ~"str", inner: ~[]});
        assert canonicalize_args(~[Arg {name: ~"T", inner: ~[]},
                                   Arg {name: ~"str", inner: ~[]}],
                                 Arg {name: ~"str", inner: ~[]})
               == (~[Arg {name: ~"A", inner: ~[]},
                     Arg {name: ~"str", inner: ~[]}],
                   Arg {name: ~"str", inner: ~[]});
        assert canonicalize_args(~[Arg {name: ~"T", inner: ~[]},
                                   Arg {name: ~"str", inner: ~[]}],
                                 Arg {name: ~"T", inner: ~[]})
               == (~[Arg {name: ~"A", inner: ~[]},
                     Arg {name: ~"str", inner: ~[]}],
                   Arg {name: ~"A", inner: ~[]});

        assert canonicalize_args(~[Arg {name: ~"T", inner: ~[]},
                                   Arg {name: ~"str", inner: ~[]},
                                   Arg {name: ~"T", inner: ~[]}],
                                 Arg {name: ~"T", inner: ~[]})
               == (~[Arg {name: ~"A", inner: ~[]},
                     Arg {name: ~"str", inner: ~[]},
                     Arg {name: ~"A", inner: ~[]}],
                   Arg {name: ~"A", inner: ~[]});

        assert canonicalize_args(~[Arg {name: ~"T", inner: ~[]},
                                   Arg {name: ~"U", inner: ~[]},
                                   Arg {name: ~"U", inner: ~[]}],
                                 Arg {name: ~"U", inner: ~[]})
               == (~[Arg {name: ~"B", inner: ~[]},
                     Arg {name: ~"A", inner: ~[]},
                     Arg {name: ~"A", inner: ~[]}],
                   Arg {name: ~"A", inner: ~[]});
    }

    #[test]
    fn test_parameterized_args() {
        assert canonicalize_args(~[Arg {name: ~"Option",
                                        inner: ~[Arg {name: ~"T", inner: ~[]}]}],
                                 Arg {name: ~"T", inner: ~[]})
            == (~[Arg {name: ~"Option",
                       inner: ~[Arg {name: ~"A", inner: ~[]}]}],
                Arg {name: ~"A", inner: ~[]});
    }

    #[test]
    fn test_vector_args() {
        assert canonicalize_args(~[Arg {name: ~"[]",
                                        inner: ~[Arg {name: ~"T", inner: ~[]}]}],
                                 Arg {name: ~"T", inner: ~[]})
            == (~[Arg {name: ~"[]",
                       inner: ~[Arg {name: ~"A", inner: ~[]}]}],
                Arg {name: ~"A", inner: ~[]});
    }

    #[test]
    fn test_trim_parens() {
        assert trim_parens(~"(hello)") == ~"hello";
        assert trim_parens(~"  (   hello)") == ~"hello";
        assert trim_parens(~"  (   hello )  ") == ~"hello";
    }

    #[test]
    fn test_trim_nested_parens() {
        assert trim_parens(~"(hello, (A,B))") == ~"hello, (A,B)";
    }

    #[test]
    fn test_trim_sigils() {
        assert trim_sigils(~"~str") == ~"str";
        assert trim_sigils(~"@str") == ~"str";
        assert trim_sigils(~"str") == ~"str";
        assert trim_sigils(~"& str") == ~"str";
    }
}