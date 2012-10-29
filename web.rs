use std::net::url;

use io::WriterUtil;

use types::Definition;

fn strip_brackets(s:~str) -> ~str {
    str::replace(str::replace(s, ~"<", ~"&lt;"), ~">", ~"&gt;")
}

fn format_def(d: @Definition) -> ~str {
    fmt!("<a href='http://dl.rust-lang.org/doc/%s.html#%s'\
        target='blank'>%s::%s</a> - %s - %s",
        str::replace(d.path, ~"::", ~"/"),d.anchor, d.path,
             d.name, strip_brackets(d.signature), strip_brackets(d.desc))
}

fn main() {
    let ctx = match zmq::init(1) {
        Ok(ctx) => ctx,
        Err(e) => fail e.to_str(),
    };

    let conn = mongrel2::connect(ctx,
        Some(~"d5e521f5-c1bf-481c-a815-bc66905526ae"),
        ~[~"tcp://127.0.0.1:9998"],
        ~[~"tcp://127.0.0.1:9999"]);

    let data = load::load(path::from_str("rustle.data"));

    loop {
        let request = result::unwrap(conn.recv());
        let query_raw = match request.headers.find_ref(&~"QUERY") {
            Some(qs) => Some(str::to_bytes(qs[0])),
            None => None
        };
        let mq = option::chain(query_raw, |que| {
            url::decode_form_urlencoded(que).find(~"q").map(|qs| { *qs[0] })
        });

        let resp = match mq {
            Some(q) => {
                do io::with_str_writer |w| {
                    // do search
                    if q.contains(~"->") || q.contains(~",") {
                        // this is a search by type, for functions
                        // build query
                        let queries = query::query(copy q);
                        // search
                        for query::search_type(queries, &data).each |d| {
                            w.write_line("<pre><code>");
                            w.write_line(format_def(*d));
                            w.write_line("</code></pre>")
                        }
                    } else {
                        // this is a search by name
                        for query::search_name(q, &data).each |d| {
                            w.write_line("<pre><code/>");
                            w.write_line(format_def(*d));
                            w.write_line("</code></pre>");
                        }
                    }
                }
            },
            None => ~""
        };


        // fancy templating :)
        let page =
            fmt!("<html><body><p>This is rustle. Check the code at <a \
                href='http://github.com/dbp/rustle'>github.com/dbp/rustle</a>\
                .</p><p>Query form: (arg1,arg2) -> ret.</p><pre><code>Examples: ([A]) -> A, \
                 (Option&lt;A&gt;) -> A, ([A], fn(A)->B) -> [B]</code></pre>\
                 <form><input type='text' name='q' size='50'/><input \
                type='submit' value='Rustle Up'/></form>%s<hr/><div>%s</div></body></html>", if resp.len() > 0
                {~"query: " + mq.get()} else { ~"" },  resp);

        conn.reply_http(&request,
            200u,
            "OK",
            mongrel2::Headers(),
            str::to_bytes(page));
    }
}