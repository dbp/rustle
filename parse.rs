//! This file contains common functions used in parsing

use types::*;

// split arguments; note that because commas
// can appear in parametric types, ie Either<A,B>,
// we need to handle this a little more carefully
pub fn split_arguments(a: &~str) -> ~[~str] {
    let mut arg_strs : ~[~str] = ~[];
    let mut level = 0;
    let mut start = 0;
    // add a trailing comma so we pick up the last argument
    let s = str::append(copy *a, ~",");
    for str::each_chari(s) |i,c| {
        match c {
            ',' if level == 0 => {
                arg_strs.push(str::trim(str::slice(s,start,i)));
                start = i+1;
            }
            '<' => level += 1,
            '>' => if i == 0 || s.char_at(i-1) != '-' { level -= 1 },
            '(' => level += 1,
            ')' => level -= 1,
            _ => {}
        }
    }
    return arg_strs;
}

// parse_signature takes a string of a function and returns a list of the
// argument types, and the return type
fn parse_signature(arg_list: ~str, self: Option<~str>, canonicalize: bool)
        -> (~[@Arg], @Arg, uint) {
    let self_list = match self {
        None => ~[],
        Some(s) => ~[parse_arg(&trim_sigils(s))]
    };
    let args_ret = str::split_str(arg_list, "->");
    let mut arlen = vec::len(args_ret);
    let ret;
    if vec::len(args_ret) == 1 {
        arlen += 1;
        ret = @Basic(~"()");
    } else {
        ret = parse_arg(&str::trim(args_ret[arlen-1]));
    }
    let arg_str =
        trim_parens(str::connect(vec::view(args_ret, 0, arlen-1), "->"));
    let args;
    if str::len(arg_str) == 0 {
        args = ~[];
    } else {
        let arg_strs = vec::map(split_arguments(&arg_str), |a| {
            let t = str::splitn_char(*a, ':', 1);
            if t.len() > 1 {
                copy t[1]
            } else {
                copy t[0]
            }
        });
        args = vec::map(arg_strs, |x| {
                parse_arg(&trim_sigils(*x))
            } );
    }
    if canonicalize {
        return canonicalize_args(vec::append(self_list,args), ret);
    } else {
        return (vec::append(self_list,args), ret, 0);
    }

}

// parse_arg takes a string and turns it into an Arg
pub fn parse_arg(su: &~str) -> @Arg {
    let s = trim_sigils(*su);
    if str::len(s) == 0 {
        return @Basic(~"()");
    }
    let ps = str::splitn_char(str::trim(s), '<', 1);
    if vec::len(ps) == 1 {
        // non-parametrized type, see if it's a vec or a tuple
        match str::char_at(s, 0) {
            '[' => {
                // if there is not a ']', we want to fail
                let end = option::get(&str::rfind_char(s, ']'));
                let mut vs = str::trim(str::slice(s, 1, end));
                // we drop any modifiers: const, mut.
                vs = drop_modifiers(&vs);
                return @Vec(parse_arg(&vs));
            }
            // want tuples, but not unit
            '(' if str::len(s) > 2 => {
                // we want to fail if there is no matching paren
                let end = option::get(&str::rfind_char(s, ')'));
                let inn = str::trim(str::slice(s, 1, end));
                let inner = vec::map(split_arguments(&inn), |a| { parse_arg(a) });
                return @Tuple(inner);
            }
            // a function type
            'f' if str::char_at(s,1) == 'n'
                && !char::is_alphanumeric(str::char_at(s,2)) => {
                let (args, ret, _n) = parse_signature(s, None, false);
                return @Function(args, ret);
            }
            _ => {
                if s.len() == 1 {
                    // assume this is a constrained type without constraints.
                    // note that this is to allow shorthand, so you don't need to write
                    // <A:>Option<A> -> A
                    // to make it fit the pattern
                    // <A: Copy, B: Copy> Option<A> -> B
                    return @Constrained(copy s, ~[]);
                } else {
                    // basic type
                    return @Basic(copy s);
                }
            }
        }
    } else {
        let params = split_arguments(&str::split_char(ps[1], '>')[0]);
        return @Parametric(@Basic(ps[0]), vec::map(params, |a| {parse_arg(a)}));
    }
}

// canonicalize_args takes a list of arguments and a return type
// and replaces generic names consistently (alphabetically, single
// uppercase letters, in order of frequency)
pub fn canonicalize_args(args: ~[@Arg], ret: @Arg) -> (~[@Arg],@Arg,uint) {
    // The basic process is as follows:
    // 1. identify and count polymorphic params
    // 2. sort and assign new letters to them
    // 3. replace names

    let identifiers : HashMap<~str, uint> = HashMap();
    fn walk_ids(a: @Arg, m: &HashMap<~str, uint>) {
        traverse_constrained(a, |n| {
            match m.find(copy *n) {
                None => m.insert(copy *n, 1),
                Some(num) => m.insert(copy *n, num+1)
            };
        });
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
    let names : HashMap<~str,@~str> = HashMap();
    let mut n = 0;
    for vec::each(identifiers_sorted) |p| {
        names.insert(p.first(), letters(n));
        n += 1;
    }
    // now rename args
    fn rename_arg(a: @Arg, n: &HashMap<~str,@~str>) -> @Arg {
        map_constrained(a, |name, constraints| {
            let nm = copy *name;
            if n.contains_key(nm) {
                @Constrained(copy *n.get(move nm), copy *constraints)
            } else {
                @Constrained(move nm, copy *constraints)
            }
        })
    }
    return (vec::map(args, |a| { rename_arg(*a, &names) }),
            rename_arg(ret,&names), n);
}

// replace_arg replaces one argument with another
pub fn replace_arg(a: @Arg, old: @Arg, new: @Arg) -> @Arg {
    match *a {
        b if b == *old => {
            new
        }
        Vec(inner) => {
            @Vec(replace_arg(inner, old, new))
        }
        Tuple(inner) => {
            @Tuple(vec::map(inner, |i| {replace_arg(*i, old, new)}))
        }
        Parametric(a, inner) => {
            @Parametric(replace_arg(a, old, new),
                       vec::map(inner, |i| { replace_arg(*i, old, new)}))
        }
        Function(args, ret) => {
            @Function(vec::map(args, |i| { replace_arg(*i, old, new)}),
                     replace_arg(ret, old, new))
        }
        _ => a
    }
}

// drop_modifiers takes off const or mut. note that it will only take off one
fn drop_modifiers(s: &~str) -> ~str {
    let end = s.len();
    for [~"const ", ~"mut "].each |m| {
        if option::is_some(&str::find_str(*s, *m)) {
            return str::trim(str::slice(*s, str::len(*m), end));
        }
    }
    return copy *s;
}

// trim_sigils trims off the sigils off of types
pub fn trim_sigils(s: &str) -> ~str {
    drop_modifiers(&str::trim_left_chars(s, &[' ', '&', '~', '@', '+']))
}

pub fn trim_parens(s: ~str) -> ~str {
    let begin = option::get_default(&str::find_char(s,'('),-1)+1;
    let end = option::get_default(&str::rfind_char(s,')'), str::len(s));
    str::trim(str::slice(s, begin, end))
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_split_arguments() {
        assert split_arguments(&~"uint, ~str") == ~[~"uint", ~"~str"];
        assert split_arguments(&~"uint, ~str, int") ==
            ~[~"uint", ~"~str", ~"int"];
        assert split_arguments(&~"uint, Type<A,B>") ==
            ~[~"uint", ~"Type<A,B>"];
        assert split_arguments(&~"uint, Type<(C,D),B>") ==
            ~[~"uint", ~"Type<(C,D),B>"];
    }

    #[test]
    fn test_parse_arg() {
        assert parse_arg(&~"~str") == Basic(~"str");
        assert parse_arg(&~"@str") == Basic(~"str");
        assert parse_arg(&~"&@str") == Basic(~"str");
        assert parse_arg(&~"Option<~str>") ==
            Parametric(~"Option", ~[Basic(~"str")]);
        assert parse_arg(&~"~[uint]") ==
            Vec(Basic(~"uint"));
        assert parse_arg(&~"(uint, ~str)") ==
            Tuple(~[Basic(~"uint"), Basic(~"str")]);
    }

    #[test]
    fn test_canonicalize_args() {
        // assert canonicalize_args(~[Arg {name: ~"str", inner: ~[]},
        //                            Arg {name: ~"str", inner: ~[]}],
        //                          Arg {name: ~"str", inner: ~[]})
        //        == (~[Arg {name: ~"str", inner: ~[]},
        //              Arg {name: ~"str", inner: ~[]}],
        //            Arg {name: ~"str", inner: ~[]}, 0);
        // assert canonicalize_args(~[Arg {name: ~"T", inner: ~[]},
        //                            Arg {name: ~"str", inner: ~[]}],
        //                          Arg {name: ~"str", inner: ~[]})
        //        == (~[Arg {name: ~"A", inner: ~[]},
        //              Arg {name: ~"str", inner: ~[]}],
        //            Arg {name: ~"str", inner: ~[]}, 1);
        // assert canonicalize_args(~[Arg {name: ~"T", inner: ~[]},
        //                            Arg {name: ~"str", inner: ~[]}],
        //                          Arg {name: ~"T", inner: ~[]})
        //        == (~[Arg {name: ~"A", inner: ~[]},
        //              Arg {name: ~"str", inner: ~[]}],
        //            Arg {name: ~"A", inner: ~[]}, 1);

        // assert canonicalize_args(~[Arg {name: ~"T", inner: ~[]},
        //                            Arg {name: ~"str", inner: ~[]},
        //                            Arg {name: ~"T", inner: ~[]}],
        //                          Arg {name: ~"T", inner: ~[]})
        //        == (~[Arg {name: ~"A", inner: ~[]},
        //              Arg {name: ~"str", inner: ~[]},
        //              Arg {name: ~"A", inner: ~[]}],
        //            Arg {name: ~"A", inner: ~[]}, 1);

        // assert canonicalize_args(~[Arg {name: ~"T", inner: ~[]},
        //                            Arg {name: ~"U", inner: ~[]},
        //                            Arg {name: ~"U", inner: ~[]}],
        //                          Arg {name: ~"U", inner: ~[]})
        //        == (~[Arg {name: ~"B", inner: ~[]},
        //              Arg {name: ~"A", inner: ~[]},
        //              Arg {name: ~"A", inner: ~[]}],
        //            Arg {name: ~"A", inner: ~[]}, 2);
    }

    #[test]
    fn test_canonicalize_parameterized_args() {
        // assert canonicalize_args(~[Arg {name: ~"Option",
        //                                 inner: ~[Arg {name: ~"T", inner: ~[]}]}],
        //                          Arg {name: ~"T", inner: ~[]})
        //     == (~[Arg {name: ~"Option",
        //                inner: ~[Arg {name: ~"A", inner: ~[]}]}],
        //         Arg {name: ~"A", inner: ~[]}, 1);
    }

    #[test]
    fn test_canonicalize_vector_args() {
        // assert canonicalize_args(~[Arg {name: ~"[]",
        //                                 inner: ~[Arg {name: ~"T", inner: ~[]}]}],
        //                          Arg {name: ~"T", inner: ~[]})
        //     == (~[Arg {name: ~"[]",
        //                inner: ~[Arg {name: ~"A", inner: ~[]}]}],
        //         Arg {name: ~"A", inner: ~[]}, 1);
    }

    #[test]
    fn test_canonicalize_tuple_args() {
        // assert canonicalize_args(~[Arg {name: ~"()",
        //                                 inner: ~[Arg {name: ~"T", inner: ~[]}]}],
        //                          Arg {name: ~"T", inner: ~[]})
        //     == (~[Arg {name: ~"()",
        //                inner: ~[Arg {name: ~"A", inner: ~[]}]}],
        //         Arg {name: ~"A", inner: ~[]}, 1);
    }

    #[test]
    fn test_replace_arg_name() {
        // let a = Constrained(~"T", ~[]);
        // assert replace_arg_name(&a, @~"T", @~"U").name == ~"U";
        // assert replace_arg_name(&a, @~"V", @~"U").name == ~"T";

        // let b = Arg {name: ~"T", inner: ~[copy a]};
        // let b_rep = replace_arg_name(&b, @~"T", @~"U");
        // assert b_rep.name == ~"U";
        // assert b_rep.inner[0].name == ~"U";

        // let b = Arg {name: ~"V", inner: ~[copy a]};
        // let b_rep = replace_arg_name(&b, @~"T", @~"U");
        // assert b_rep.name == ~"V";
        // assert b_rep.inner[0].name == ~"U";
    }

    #[test]
    fn test_drop_modifiers() {
        assert drop_modifiers(&~"const hello") == ~"hello";
        assert drop_modifiers(&~"mut hello") == ~"hello";
        assert drop_modifiers(&~"hello") == ~"hello";
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