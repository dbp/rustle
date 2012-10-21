//! This file contains type definitions

use std::treemap::*;
type Set<T: Copy Ord> = TreeMap<T,()>;

use core::cmp::{Eq, Ord};

// Note that we are using treemaps to get unordered comparison.
// because vecs are much more supported, we only convert to sets
// when we compare. this is inefficient, but intended to be temporary.
// once a more supported container is in the standard library, this will
// all be pulled out, and some sort of set will be used throughout.
fn Set<T: Copy Eq Ord>() -> Set<T> {
    TreeMap()
}
fn set_equals<K: Copy Eq Ord>(t1: &const Set<K>,
                              t2: &const Set<K>)
                            -> bool {
    let mut v1 = ~[];
    let mut v2 = ~[];
    traverse(*t1, |k,_v| { v1.push((copy *k))} );
    traverse(*t2, |k,_v| { v2.push((copy *k))} );
    return v1 == v2;
}
pub fn set_from_vec<T: Copy Ord Eq>(v: &~[T]) -> Set<T> {
    let mut s = Set();
    for v.each |e| {
        insert(s,*e,());
    }
    return s;
}

// an Arg is a name, like str or Option, and then an optional list
// of parameters. ex: Option<T> is "Option", ["T"] (roughly).
// struct Arg { name: ~str, inner: ~[Arg] }

enum Arg {
    Basic(~str),
    Parametric(@Arg, ~[@Arg]),
    // Tuple and Vecs are really special cases of Parametric types, but
    // are sufficiently special that it seems worthwhile to handle them
    // separately.
    Tuple(~[@Arg]),
    Vec(@Arg),
    Constrained(~str, ~[Constraint]),
    Function(~[@Arg],@Arg)
}

pub fn map_constrained(a: @Arg, f: fn(&~str, &~[Constraint]) -> @Arg) -> @Arg {
    match *a {
        Constrained(name, constraints) => f(&name, &constraints),
        Tuple(args) => @Tuple(vec::map(args, |a| { map_constrained(*a, f)})),
        Vec(arg) => @Vec(map_constrained(arg, f)),
        Parametric(arg,args) => {
            @Parametric(map_constrained(arg, f),
                       vec::map(args, |a| { map_constrained(*a, f)}))
        }
        Function(args, ret) => {
            @Function(vec::map(args, |a| { map_constrained(*a, f)}),
                      map_constrained(ret, f))
        }
        Basic(_) => a
    }
}

pub fn traverse_constrained(a: @Arg, f: fn(&~str)) {
    map_constrained(a, |n,_cs| { f(n); a } );
}


enum Constraint = ~str;

impl Constraint : Eq {
    pure fn eq(other: &Constraint) -> bool {
        *self == **other
    }
    pure fn ne(other: &Constraint) -> bool {
        *self != **other
    }
}

impl Constraint : Ord {
    pure fn ge(other: &Constraint) -> bool {
        *self >= **other
    }
    pure fn le(other: &Constraint) -> bool {
        *self <= **other
    }
    pure fn gt(other: &Constraint) -> bool {
        *self > **other
    }
    pure fn lt(other: &Constraint) -> bool {
        *self < **other
    }
}

pure fn arg_eq(s: &Arg, o: &Arg) -> bool {
    match (s, o) {
        (&Basic(ref s1), &Basic(ref s2)) => s1 == s2,
        (&Parametric(ref s1, ref a1), &Parametric(ref s2, ref a2)) =>
            (s1 == s2) && (a1 == a2),
        (&Tuple(a1),&Tuple(a2)) => a1 == a2,
        (&Vec(t1),&Vec(t2)) => t1 == t2,
        (&Constrained(s1,c1),&Constrained(s2,c2)) =>
            (s1 == s2) && (c1 == c2),
        (&Function(a1,r1),&Function(a2,r2)) => (a1 == a2) && (r1 == r2),
        _ => false
    }
}

impl Arg : Eq {
    pure fn eq(other: &Arg) -> bool {
        arg_eq(&self, other)
    }
    pure fn ne(other: &Arg) -> bool {
        !arg_eq(&self, other)
    }
}

pure fn arg_le(s: &Arg, o: &Arg) -> bool {
    match (s, o) {
        // we define ordering of variants, and then ordering within variants
        (&Basic(ref s1), &Basic(ref s2)) => s1 <= s2,
        (&Parametric(ref s1, ref a1), &Parametric(ref s2, ref a2)) =>
            if s1 == s2 { a1 <= a2 } else { s1 <= s2 },
        (&Tuple(a1),&Tuple(a2)) => a1 <= a2,
        (&Vec(t1),&Vec(t2)) => t1 <= t2,
        (&Constrained(s1,c1),&Constrained(s2,c2)) =>
            if s1 == s2 { c1 <= c2 } else { s1 <= s2 },
        (&Function(a1,r1),&Function(a2,r2)) =>
            if a1 == a2 { r1 <= r2 } else { a1 <= a2 },
        (&Basic(_), _) => true,
        (_, &Basic(_)) => false,
        (&Parametric(_,_), _) => true,
        (_, &Parametric(_,_)) => false,
        (&Tuple(_), _) => true,
        (_, &Tuple(_)) => false,
        (&Vec(_), _) => true,
        (_, &Vec(_)) => false,
        (&Constrained(_,_), _) => true,
        (_, &Constrained(_,_)) => false,
        // these are implicit in the above patterns
        //(&Function(_,_), _) => true,
        //(_, &Function(_,_)) => false,
    }
}

// we are only ordering on the name, at least for now
impl Arg : Ord {
    pure fn ge(other: &Arg) -> bool {
        !arg_le(&self, other) || arg_eq(&self, other)
    }
    pure fn le(other: &Arg) -> bool {
        arg_le(&self, other)
    }
    pure fn gt(other: &Arg) -> bool {
        !arg_le(&self, other)
    }
    pure fn lt(other: &Arg) -> bool {
        arg_le(&self, other) && !arg_eq(&self, other)
    }
}

// a query is a set of arguments and a return type
struct Query { args: ~[@Arg], ret: @Arg }

impl Query : Eq {
    pure fn eq(other: &Query) -> bool {
        (self.args == other.args) && (self.ret == other.ret)
    }
    pure fn ne(other: &Query) -> bool {
        (self.args != other.args) || (self.ret != other.ret)
    }
}

// a Definition is what we are trying to match against. Note that
// definitions are not exactly unique, as they can be made more specific
// (ie, A,B -> C can be A,A -> B, etc)
struct Definition { name: ~str, path: ~str, anchor: ~str, desc: ~str,
                    args: ~[@Arg], ret: @Arg, signature: ~str }

impl Definition : Eq {
    pure fn eq(other: &Definition) -> bool {
        (self.name == other.name) && (self.path == other.path) &&
        (self.anchor == other.anchor) && (self.desc == other.desc) &&
        (self.args == other.args) && (self.ret == other.ret) &&
        (self.signature == other.signature)
    }
    pure fn ne(other: &Definition) -> bool {
        (self.name != other.name) || (self.path != other.path) ||
        (self.anchor != other.anchor) || (self.desc != other.desc) ||
        (self.args != other.args) || (self.ret != other.ret) ||
        (self.signature != other.signature)
    }
}

// fn show_def returns a representation of the definition suitable for printing
impl Definition {
    fn show() -> ~str {
        fmt!("%s::%s - %s - %s", self.path,
             self.name, self.signature, self.desc)
    }
}

// A bucket holds a bunch of definitions
struct Bucket { defs: ~[@Definition] }

// A trie is used to look up names efficiently by prefix. We assume that
// most searches will be by the beginning of names, and can expand later.
// This allows a fast simple implementation (hashing used here for simplicity
// as well, though it is totally unnecessary).
struct Trie { children: HashMap<~str,@Trie>, mut defs: ~[@Definition] }

// Data stores all the definitions in buckets, based on function arity.
struct Data { ar0: Bucket, ar1: Bucket, ar2: Bucket,
              ar3: Bucket, ar4: Bucket, ar5: Bucket,
              arn: Bucket, names: @Trie }

fn empty_data() -> Data {
    let empty_bucket = Bucket { defs: ~[] };
    let empty_trie = Trie { children: HashMap(), defs: ~[] };
    Data { ar0: empty_bucket, ar1: empty_bucket, ar2: empty_bucket,
           ar3: empty_bucket, ar4: empty_bucket, ar5: empty_bucket,
           arn: empty_bucket, names: @empty_trie}
}

fn letters(n: uint) -> @~str {
    match n {
        0  => @~"A",
        1  => @~"B",
        2  => @~"C",
        3  => @~"D",
        4  => @~"E",
        5  => @~"F",
        6  => @~"G",
        7  => @~"H",
        8  => @~"I",
        9  => @~"J",
        10 => @~"K",
        11 => @~"L",
        12 => @~"M",
        13 => @~"N",
        14 => @~"O",
        15 => @~"P",
        16 => @~"Q",
        17 => @~"R",
        18 => @~"S",
        19 => @~"T",
        20 => @~"U",
        21 => @~"V",
        22 => @~"W",
        23 => @~"X",
        24 => @~"Y",
        25 => @~"Z",
        _  => fail ~"not enough letters"
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_set_equals() {
        let v1 : ~[uint] = ~[1,2,4];
        let v2 : ~[uint] = ~[2,1,4];
        assert set_equals(&set_from_vec(&v1), &set_from_vec(&v2));

        let u1 = ~[@Basic(~"uint"),
                   @Vec(@Basic(~"A")),
                   @Basic(~"A")];
        let u2 = ~[@Vec(@Basic(~"A")),
                   @Basic(~"A"),
                   @Basic(~"uint")];
        assert set_equals(&set_from_vec(&u1), &set_from_vec(&u2));
    }

    #[test]
    fn arg_eq() {
        assert @Basic(~"uint") == @Basic(~"uint");
        assert @Parametric(@Basic(~"uint"), ~[@Basic(~"A")]) != @Basic(~"A");
    }

    #[test]
    fn test_definition_show() {
        let d =
            Definition { name: ~"foo", path: ~"core::foo", anchor: ~"fun-foo",
                         desc: ~"foo does bar", args: ~[],
                         ret: @Basic(~"int"),
                         signature: ~"fn foo() -> int" };
        assert d.show() == ~"core::foo::foo - fn foo() -> int - foo does bar";
    }

    #[test]
    fn test_letters() {
        assert letters(1) == @~"B";
        assert letters(25) == @~"Z";
    }
}