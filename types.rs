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

// a Definition is what we are trying to match against. Note that
// definitions are not exactly unique, as they can be made more specific
// (ie, A,B -> C can be A,A -> B, etc)
struct Definition { name: ~str, path: ~str, anchor: ~str, desc: ~str,
                    args: ~[Arg], ret: Arg, signature: ~str }

impl Definition : to_str::ToStr {
    fn to_str() -> ~str {
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
struct Trie { children: HashMap<~str,@Trie>, mut definitions: ~[@Definition] }

// Data stores all the definitions in buckets, based on function arity.
struct Data { mut ar0: Bucket, mut ar1: Bucket, mut ar2: Bucket,
              mut ar3: Bucket, mut ar4: Bucket, mut ar5: Bucket,
              mut arn: Bucket, mut names: @Trie }

fn empty_data() -> Data {
    let empty_bucket = Bucket { defs: ~[] };
    let empty_trie = Trie { children: HashMap(), definitions: ~[] };
    Data { ar0: empty_bucket, ar1: empty_bucket, ar2: empty_bucket,
           ar3: empty_bucket, ar4: empty_bucket, ar5: empty_bucket,
           arn: empty_bucket, names: @empty_trie}
}

fn letters() -> ~[~str] {
    vec::map(str::chars("ABCDEFGHIJKLMNOPQRSTUVWXYZ"),
            |c| { str::from_char(*c) })
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
}