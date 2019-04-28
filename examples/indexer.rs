use std::io::Result;
use std::collections::HashMap;

extern crate nntp;
use nntp::client::Client;

pub fn main() -> Result<()> {
//    let mut client = Client::connect_tls("us.newsgroupdirect.com", 563)?;
    let mut client = Client::connect("nntp.aioe.org", 119)?;

    let response = client.read_response()?;
    assert!(response.expected("200"));

//    client.reader()?;

//    use std::env;
//    let envmap : HashMap<String,String> = env::vars().collect();
//    client.authinfo_user(envmap.get("NEWSGROUP_USER").expect("newsgroup user"))?;
//    client.authinfo_pass(envmap.get("NEWSGROUP_PASS").expect("newsgroup pass"))?;

    client.discovery_capabilities()?;

    let group_response = client.group("comp.sys.raspberry-pi")?;
    let (_num_articles, low_water_mark, high_water_mark) = group_response.parse_group_stats()?;
    println!("num articles: {}, low water mark: {}, high water mark: {}", _num_articles, low_water_mark, high_water_mark);

    for i in low_water_mark .. high_water_mark {
        client.article_by_id_pipeline(i)?;
    }
    client.flush_pipeline()?;

    let mut counter= 0;
    for _ in low_water_mark .. high_water_mark {
        if let Ok(article) = client.article_by_id_pipeline_read() {
            counter += 1;
        }
    }
    println!("we read {} articles", counter);

    client.quit().map(|_| ())
}