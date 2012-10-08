//! This file contains type definitions

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

// a query is a ...
struct Query { args: ~[Arg], ret: Arg }

impl Query : cmp::Eq {
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
                    args: ~[Arg], ret: Arg, signature: ~str }

impl Definition : cmp::Eq {
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
struct Bucket { mut defs: ~[@Definition] }

// A trie is used to look up names efficiently by prefix. We assume that
// most searches will be by the beginning of names, and can expand later.
// This allows a fast simple implementation (hashing used here for simplicity
// as well, though it is totally unnecessary).
struct Trie { children: HashMap<~str,@Trie>, mut defs: ~[@Definition] }

// Data stores all the definitions in buckets, based on function arity.
struct Data { mut ar0: Bucket, mut ar1: Bucket, mut ar2: Bucket,
              mut ar3: Bucket, mut ar4: Bucket, mut ar5: Bucket,
              mut arn: Bucket, mut names: @Trie }

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
    fn arg_eq() {
        assert Arg { name: ~"uint", inner: ~[] } == Arg { name: ~"uint", inner: ~[] };
        assert !(Arg { name: ~"uint", inner: ~[Arg { name: ~"A", inner: ~[] }] }
                 == Arg { name: ~"uint", inner: ~[] });
        assert !(Arg { name: ~"uint", inner: ~[Arg { name: ~"A", inner: ~[] }] }
                 == Arg { name: ~"uint", inner: ~[Arg { name: ~"B", inner: ~[] }] });
    }

    #[test]
    fn test_definition_show() {
        let d =
            Definition { name: ~"foo", path: ~"core::foo", anchor: ~"fun-foo",
                         desc: ~"foo does bar", args: ~[],
                         ret: Arg {name: ~"int", inner: ~[]},
                         signature: ~"fn foo() -> int" };
        assert d.show() == ~"core::foo::foo - fn foo() -> int - foo does bar";
    }

    #[test]
    fn test_letters() {
        assert letters(1) == @~"B";
        assert letters(25) == @~"Z";
    }
}