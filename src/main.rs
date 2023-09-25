use std::{
    env,
    io::{BufReader, Read},
};

use reqwest::header::USER_AGENT;
use xml::reader::{EventReader, XmlEvent};

const HEADER: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/117.0.0.0 Safari/537.36";

fn fetch_sitemap(url: &String) -> String {
    let client = reqwest::blocking::Client::new();

    let mut body = String::new();
    let _res = client
        .get(url)
        .header(USER_AGENT, HEADER)
        .send()
        .expect("XML response")
        .read_to_string(&mut body);

    return body;
}

fn parse_xml(content: String) -> Vec<String> {
    let file = BufReader::new(content.as_bytes());
    let parser = EventReader::new(file);

    let mut inside_loc_element = false;
    let mut vec = Vec::new();

    for e in parser {
        match e {
            Ok(XmlEvent::StartElement { name, .. }) => {
                if name.local_name == "loc" {
                    inside_loc_element = true;
                }
            }
            Ok(XmlEvent::EndElement { name, .. }) => {
                if name.local_name == "loc" {
                    inside_loc_element = false;
                }
            }
            Ok(XmlEvent::Characters(text)) => {
                if inside_loc_element {
                    vec.push(text);
                }
            }
            Err(e) => {
                eprintln!("Error: {e}");
                break;
            }
            _ => {}
        }
    }

    return vec;
}

fn check_url(url: &String) {
    let client = reqwest::blocking::Client::new();

    let res = client
        .get(url)
        .header(USER_AGENT, HEADER)
        .send()
        .expect("HTTP response");

    if !res.status().is_success() {
        println!(":: Error loading: {}", url);
        println!("   -> Status code: {}", res.status());
    }
}

fn main() {
    let now = std::time::Instant::now();

    let args: Vec<String> = env::args().collect();
    let url = &args[1];
    let body = fetch_sitemap(&url);

    if body.len() == 0 {
        return;
    }

    println!("> Sitemap seems to be present");
    println!("> Checking links...");

    let urls = parse_xml(body);

    for url in urls {
        check_url(&url);
    }

    println!("> Check complete!");
    println!("Elapsed: {:.2?}", now.elapsed());
}
