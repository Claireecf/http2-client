use std::env;
use isahc::prelude::*;
use std::fs::OpenOptions;
use chrono::DateTime;
use chrono::offset::Utc;
use std::io::Write;
use std::{time::SystemTime};
use isahc::{
    config::VersionNegotiation,
    HttpClient,
};

fn main() -> Result<(), isahc::Error> {
    // Send a GET request and wait for the response headers.
    // Must be `mut` so we can read the response body.
    // HTTP/2 with prior knowledge.
    let args: Vec<String> = env::args().collect();
    let url = &args[1];
    let mut file = OpenOptions::new().write(true).read(true).append(true).create(true).open("time.txt").unwrap();
    let start = SystemTime::now();
    let datetime: DateTime<Utc> = start.into();
    let content = String::from("HTTP3 page visit start time is: ");
    file.write_all(content.as_bytes()).unwrap();
    file.write_all(datetime.format("%m/%d/%Y %T%.3f\n").to_string().as_bytes()).unwrap();
    let http2_client = HttpClient::builder()
    .version_negotiation(VersionNegotiation::http2())
    .build()?;
    
    let mut response = http2_client.get(url)?;

    // Print some basic info about the response to standard output.
    // println!("Status: {}", response.status());
    // println!("Headers: {:#?}", response.headers());

    // Read the response body as text into a string and print it.
    print!("{}", response.text()?);
    let end = SystemTime::now();
    let datetime: DateTime<Utc> = end.into();
    let content = String::from("HTTP3 page visit end time is: ");
    file.write_all(content.as_bytes()).unwrap();
    file.write_all(datetime.format("%m/%d/%Y %T%.3f\n").to_string().as_bytes()).unwrap();

    Ok(())
}