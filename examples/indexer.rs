use std::io::Result;
use std::collections::HashMap;

extern crate nntp;
#[macro_use] extern crate log;
use nntp::client::Client;

pub fn main() -> Result<()> {
    simple_logger::init().expect("simple logger");
    let mut client = Client::connect_tls("us.newsgroupdirect.com", 563, 32*1024)?;
//    let mut client = Client::connect("nntp.aioe.org", 119)?;

    let response = client.read_response()?;
    assert!(response.expected("200"));

//    client.reader()?;

    use std::env;
    let envmap : HashMap<String,String> = env::vars().collect();
    client.authinfo_user(envmap.get("NEWSGROUP_USER").expect("newsgroup user"))?;
    client.authinfo_pass(envmap.get("NEWSGROUP_PASS").expect("newsgroup pass"))?;

    client.discovery_capabilities()?;
    info!("client: {:#?}", client);

    let group_response = client.group("comp.sys.raspberry-pi")?;
    let (_num_articles, low_water_mark, high_water_mark) = group_response.parse_group_stats()?;
    info!("num articles: {}, low water mark: {}, high water mark: {}", _num_articles, low_water_mark, high_water_mark);

    let compression = client.xfeature_compress_gzip().expect("compression");
    println!("compression: {:#?}", compression);

    client.discovery_capabilities()?;
    info!("client: {:#?}", client);

    let chunk_size = 1;
    let chunks1 = &(low_water_mark .. high_water_mark).chunks(chunk_size);
    let chunks2 = &(low_water_mark .. high_water_mark).chunks(chunk_size);

    use itertools::Itertools;
    for (chunk1,chunk2) in chunks1.into_iter().zip(chunks2.into_iter()) {
        for i in chunk1 {
            client.head_by_range_pipeline_write(low_water_mark .. 100)?;
        }
        client.flush_pipeline()?;

        let mut counter= 0;

        for _ in 0..10 {
            trace!("reading!");

            if let Ok(headers) = client.head_by_id_read_pipeline() {
                if !headers.success() {
                    error!("just a little failure: {}", &headers.response_line[0..3]);
                } else {
                    trace!("headers response: {:#?}", std::str::from_utf8(headers.rest.as_ref().expect("rest")));
                    counter += 1;
                }
            } else {
                panic!("what")
            }
        }
//        println!("we read {} articles (estimates had it at {})", counter, high_water_mark-low_water_mark);
        println!("our stream: {:#?}", client.stream);
        panic!("bing");
    }

    client.quit().map(|_| ())
}