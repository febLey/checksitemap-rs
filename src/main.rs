use quick_xml::events::Event;
use quick_xml::Reader;
use rayon::prelude::*;
use reqwest::header::USER_AGENT;
use std::{env, io::Read};

const HEADER: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/117.0.0.0 Safari/537.36";

fn fetch_sitemap(client: &reqwest::blocking::Client, url: &String) -> String {
    let mut body = String::new();
    let _res = client
        .get(url)
        .header(USER_AGENT, HEADER)
        .send()
        .expect("XML response")
        .read_to_string(&mut body);

    return body;
}

fn parse_xml(content: &str) -> Vec<String> {
    // let mut inside_loc_element = false;

    let mut reader = Reader::from_str(content);
    // reader.trim_text(true);
    let mut buf = Vec::new();

    let mut urls = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            // error
            Err(e) => panic!("XML error at position {}: {:?}", reader.error_position(), e),
            // end of file
            Ok(Event::Eof) => break,

            Ok(Event::Start(e)) if e.name().as_ref() == b"loc" => {
                if let Ok(Event::Text(t)) = reader.read_event_into(&mut buf) {
                    urls.push(t.decode().unwrap().to_string());
                }
            }
            _ => {}
        }
    }

    return urls;
}

fn check_url(client: &reqwest::blocking::Client, url: &String) {
    let res = client
        .get(url)
        .header(USER_AGENT, HEADER)
        .send()
        .expect("HTTP response");

    let status = res.status();
    let _ = res.bytes();

    if !status.is_success() {
        println!(":: Error loading: {}", url);
        println!("   -> Status code: {}", status);
    }
}

fn main() {
    let now = std::time::Instant::now();

    let args: Vec<String> = env::args().collect();
    let client = reqwest::blocking::Client::new();
    let url = &args[1];
    let body = fetch_sitemap(&client, &url);

    if body.len() == 0 {
        return;
    }

    println!("> Sitemap seems to be present");
    println!("> Checking links...");

    let urls = parse_xml(&body);

    urls.par_iter().for_each(|url| {
        check_url(&client, url);
    });

    println!("> Check complete!");
    println!("Elapsed: {:.2?}", now.elapsed());
}
