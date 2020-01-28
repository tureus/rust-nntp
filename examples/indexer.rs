use std::collections::HashMap;

#[macro_use]
extern crate log;
use nntp::capabilities::{Capability, Compression};

use nntp::prelude::*;

pub fn main() -> Result<(), NNTPError> {
    simple_logger::init().expect("simple logger");
    //            let mut client = Client::connect_tls("us.newsgroupdirect.com", 563, 32 * 1024)?;
    let mut client = Client::connect("nntp.aioe.org", 119)?;

    let response = client.read_response_line()?;
    assert!(response.starts_with("200"));

    //    client.reader()?;

    let auth = false;
    if auth {
        use std::env;
        let envmap: HashMap<String, String> = env::vars().collect();
        client.authinfo_user(envmap.get("NEWSGROUP_USER").expect("newsgroup user"))?;
        client.authinfo_pass(envmap.get("NEWSGROUP_PASS").expect("newsgroup pass"))?;
    }

    client.discovery_capabilities()?;
    info!("client: {:#?}", client);

    let group_response = client.group("comp.sys.raspberry-pi")?;
    let (_num_articles, low_water_mark, high_water_mark) = group_response.parse_group_stats()?;
    info!(
        "num articles: {}, low water mark: {}, high water mark: {}",
        _num_articles, low_water_mark, high_water_mark
    );

    if client.can(Capability::XFEATURE_COMPRESS(vec![Compression::GZIP])) {
        let compression = client.xfeature_compress_gzip().expect("compression");
        println!("compression: {:#?}", compression);
    }

    debug!("gzip: {}", client.stream.gzip());
    client.discovery_capabilities()?;
    info!("client: {:#?}", client);

    let chunk_size = 10;
    use itertools::Itertools;
    let chunks = &(low_water_mark..high_water_mark).chunks(chunk_size);

    for chunk in chunks {
        let mut iter = chunk.into_iter();
        let first = iter.next().unwrap();
        let last = iter.last().unwrap();
        info!("{:?}-{:?}", first, last);

        for id in first..=last {
            client.head_by_id_pipeline_write(id)?;
        }

        let mut results = vec![];
        for _ in first..=last {
            let res = client.head_by_id_read_pipeline()?;

            if let (Some(headers), Some(raw_headers)) = (res.headers(), res.raw_headers()) {
                info!("raw headers: {:?}\nheaders: {:#?}", raw_headers, headers)
            }

            results.push(res);
        }

        info!("{:#?}", results);
    }

    client.quit().map(|_| ())
}
