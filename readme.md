about
-----
rustle is an api search tool inspired by [Hoogle](http://www.haskell.org/hoogle/). It allows you to search the core api (and hopefully std soon) by function signatures. An example would be if you were wondering how to get the value out of an Either, you could search
`Either<A,B> -> A` and you would get the following result: `core::either::unwrap_left: fn unwrap_left<T, U>(eith: Either<T, U>) -> T - Retrieves the value in the left branch.`
`Fails if the either is Right.` Similarly, `([A], fn(A)->B) -> [B]` returns `vec::map` (and a few variants).

usage
-----

1. `rustc rustle.rc` build rustle
2. `runghc scrape.hs /path/to/rust/doc` scrape documentation, creates rustle.data file. note that this is optional and requires the Haskell GHC compiler (as well as tagsoup and json from Hackage), as the repository includes prescraped data (but if the docs change, the data will be out of date).
3. `./rustle` start up rustle. Note that it expects rustle.data to be in the current directory.
4. type query!

(Alternatively, you can run it with single searches, like `./rustle "Option<A> -> bool`, but it will have to load in the data for each query, so the interactive mode is a lot faster. Also - using a readline wrapper like `rlwrap` is recommended, so you get line editing and history. `rlwrap ./rustle` will work.).

web
---

I've written a very minimal web frontend, using mongrel2 and @erickt's libraries to connect with it. It lives in web.rc/web.rs, and it depends upon `mongrel2`, `zmq`, and `tnetstring` being installed from cargo. It also obviously requires zeromq and mongrel2 (using web.conf) to run.

how
---
Right now the data is all scraped out of the documentation that rustdoc creates. We then parse out the arguments and return types (and self types for methods), discarding pointer types and some other stuff (like mut/const inside vector types). We then replace polymorphic type variables (single uppercase letters, by our assumption) in a way that is consistent (so, for example, you can search for `Option<A> -> A` and match against `Option<T> -> T`), and finally store all of this based on the number of arguments that a function has (stored this way to make searching faster). We also create some variants in the case of polymorphic functions - so for example, `Either<A,B> -> A` will also be recorded as `Either<A,A> -> A`.

To query, we parse the query into the same form, and now expand it to more general forms. So for example, `Either<int,uint> -> int` will also create `Either<A,uint> -> A` and `Either<A, B> -> A`. We then search against all of those (from most specific to most general), returning anything that matches. The comparisons are done without regard for the order of arguments (but only the top level - ie, `Either<A,B>` will not match `Either<B,A>` - though hopefully the combination of consistent ordering of polymorphic types will help alleviate problems here.)

We will also search by function name if the query does not have a `->` or `,` - the search is prefix only, for now. ie, to find `each_char`, `each` will work, not `char`.

limitations
-----------
There are two main problems:

1. Lack of heuristics in general for search - there is no weighting, and the only way that results are weighted is by the order they are added to the data structures, which is based on the order of files processed by the scraper.

2. No understanding of traits. All polymorphic parameters (identified by being a single capital letter) are treated as the same. This is a major limitation, and because of this (and the lack of heuristics), we currently don't include trait implementations in the search results, because they added a lot of results that weren't incredibly useful. Note that we do include methods defined on specific types (so it isn't `impl` that we ignore, but trait impls).


future
------
High priorities:
 * adding awareness of traits
 * adding some heuristics for results
 * creating a web frontend (and rustbot frontend?)

Lower priorities:
 * use rustdoc to extract the information that we need (currenly we scrape the html that rustdoc generates) - blocking that is that rustdoc is not a library, so it's functionality can't be used in other applications.

Longer term:
 * match partial function signatures

changelog
---------
0.4.2 - adding basic web frontend, putting online at http://lab.dbpmail.net/rustle/

0.4.1 - adding support for higher order functions, and refactoring argument types to support traits in the future.

0.4 - initial version. major version numbers will be tied to rust version.

authors
-------
Daniel Patterson (dbp)