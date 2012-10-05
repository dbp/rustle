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