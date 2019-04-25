extern crate nntp;
#[allow(unused_imports)]
#[macro_use] extern crate prettytable;
extern crate bufstream;

#[allow(unused_imports)]
use std::collections::HashMap;

use nntp::{ParsedArticle, NNTPStream, NewsGroup};
#[allow(unused_imports)]
use prettytable::Table;

use bufstream::BufStream;

fn main() -> Result<(), std::io::Error> {
    let mut tcp_stream = std::net::TcpStream::connect(("us.newsgroupdirect.com", 563))?;

    let connector = native_tls::TlsConnector::new().unwrap();
    let stream = connector.connect("us.newsgroupdirect.com", tcp_stream).map_err(|x| {
        std::io::Error::new(std::io::ErrorKind::Other, "tls failed")
    })?;
    let stream = BufStream::new(stream);
    let mut nntp_stream = NNTPStream::connect(stream)?;

//    let GROUP = "comp.sys.raspberry-pi";
    let GROUP = "alt.binaries.warez";

    let capabilities = nntp_stream.capabilities()?;
    println!("{:#?}", capabilities);

    use std::env;
    let envmap : HashMap<String,String> = env::vars().collect();
    nntp_stream.auth_info(envmap.get("NEWSGROUP_USER").expect("newsgroup user"), envmap.get("NEWSGROUP_PASS").expect("newsgroup pass"))?;

    let groups = nntp_stream.list()?;
    panic!("{:#?}", groups);
//    let groups_by_name : HashMap<&str, &NewsGroup> = groups.iter().map(|x| (&x.name[..], x)).collect();
//
//    let mut t = Table::new();
//    for (name,_) in groups_by_name.iter() {
//        t.add_row(row![name]);
//    }
//    t.printstd();
//
    nntp_stream.group(GROUP)?;

    while nntp_stream.next().is_ok() {
        let headers = nntp_stream.head().unwrap();
        // has it gone through more than one resize?
        if headers.size() > 1024*2*2 {
            // nothing
        }
        let parsed = headers.parse()?;
        println!("{:?}", parsed);
        println!("code: {}, message: {}", parsed.code, parsed.message);
        println!("bleep: {:?}", parsed.headers[0])
    }

    let last_response = nntp_stream.next()?;
    println!("response: {}", last_response);
    let _ = nntp_stream.head()?;
//    panic!("got whatever\n{:#?}", whatever);

    println!("COMMAND: quit");
    nntp_stream.quit()
}
