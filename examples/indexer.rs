use std::collections::HashMap;

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate tokio;

use nntp::capabilities::{Capability, Compression};

use elasticsearch::http::request::JsonBody;
use elasticsearch::BulkParts;
use nntp::prelude::*;
use tokio::prelude::*;

#[tokio::main]
pub async fn main() -> Result<(), NNTPError> {
    env_logger::init();
    let mut elastic_client = elasticsearch::Elasticsearch::default();

    let mut client = Client::connect_tls("us.newsgroupdirect.com", 563, 32 * 1024)?;
    //    let mut client = Client::connect("nntp.aioe.org", 119)?;

    let response = client.read_response_line()?;
    assert!(response.starts_with("200"));

    let auth = true;
    if auth {
        use std::env;
        let envmap: HashMap<String, String> = env::vars().collect();
        client.authinfo_user(envmap.get("NEWSGROUP_USER").expect("newsgroup user"))?;
        client.authinfo_pass(envmap.get("NEWSGROUP_PASS").expect("newsgroup pass"))?;
    }

    //    let groups = client.list()?;
    //    info!("{:#?}", groups);

    client.discovery_capabilities()?;
    info!("client: {:#?}", client);

    //    let group_response = client.group("comp.sys.raspberry-pi")?;
    let group_name = "alt.binaries.frogs";
    let group_response = client.group(group_name)?;
    let (_num_articles, low_water_mark, high_water_mark) = group_response.parse_group_stats()?;
    info!(
        "num articles: {}, low water mark: {}, high water mark: {}",
        _num_articles, low_water_mark, high_water_mark
    );

    if client.can(Capability::XFEATURE_COMPRESS(vec![Compression::GZIP])) && false {
        let compression = client.xfeature_compress_gzip().expect("compression");
        println!("compression: {:#?}", compression);
    }

    debug!("gzip: {}", client.stream.gzip());
    client.discovery_capabilities()?;
    info!("client: {:#?}", client);

    let chunk_size = 150;
    use itertools::Itertools;
    let chunks = &(low_water_mark..high_water_mark).chunks(chunk_size);

    for chunk in chunks {
        let mut docs: Vec<JsonBody<_>> = Vec::with_capacity(chunk_size * 2);

        let mut iter = chunk.into_iter();
        let first = iter.next().unwrap();
        let last = iter.last().unwrap();
        info!("{:?}-{:?}", first, last);

        for id in first..=last {
            client.head_by_id_pipeline_write(id)?;
        }
        client.flush();

        for id in first..=last {
            let res = client.head_by_id_read_pipeline()?;
            let doc_id = format!("{}-{}", group_name, id);
            docs.push(json!({"index": {"_index": "index-00001", "_id": doc_id}}).into());

            let headers = match res.headers() {
                Some(h) => h,
                None => {
                    info!("skipping a doc for {}", id);
                    continue;
                }
            };
            let json_value = serde_json::to_value(
                headers
                    .0
                    .iter()
                    .map(|(k, v)| {
                        (
                            std::str::from_utf8(k).unwrap(),
                            v.map(|x| std::str::from_utf8(x).unwrap()),
                        )
                    })
                    .collect::<HashMap<_, _>>(),
            );
            let doc = match json_value {
                Err(e) => panic!(
                    "could not serialize {:#?}\n{:?}",
                    res.headers().unwrap().0,
                    e
                ),
                Ok(d) => d,
            };
            docs.push(doc.into());
        }

        let inst = std::time::Instant::now();
        let res = elastic_client
            .bulk(BulkParts::Index("usenet"))
            .body(docs)
            .send()
            .await;
        if res.is_err() {
            error!("bulk index failed {}", res.err().unwrap())
        }
        info!("bulk index done {:?}", inst.elapsed());
    }

    client.quit().map(|_| ())
}
