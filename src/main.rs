use std::env;
use isahc::prelude::*;
use std::fs::OpenOptions;
use chrono::DateTime;
use chrono::offset::Utc;
use std::io::Write;
use isahc::{config::SslOption, prelude::*, Request};
use std::{time::SystemTime};
use opentelemetry::global;
use opentelemetry::sdk::export::trace::stdout;
use opentelemetry::sdk::{
    propagation::TraceContextPropagator,
    trace::{self, Sampler},
};
use opentelemetry::{
    trace::{TraceContextExt, Tracer},
    Context, KeyValue,
};
use opentelemetry_http::HeaderInjector;

use isahc::{
    config::VersionNegotiation,
    HttpClient,
};

fn init_tracer() -> impl Tracer {
    global::set_text_map_propagator(TraceContextPropagator::new());
    // Install stdout exporter pipeline to be able to retrieve the collected spans.
    // For the demonstration, use `Sampler::AlwaysOn` sampler to sample all traces. In a production
    // application, use `Sampler::ParentBased` or `Sampler::TraceIdRatioBased` with a desired ratio.
    stdout::new_pipeline()
        .with_trace_config(trace::config().with_sampler(Sampler::AlwaysOn))
        .install_simple()
}

fn main() -> Result<(), isahc::Error> {
    // Send a GET request and wait for the response headers.
    // Must be `mut` so we can read the response body.
    // HTTP/2 with prior knowledge.
    // let args: Vec<String> = env::args().collect();
    // let url = &args[1];
    // let mut file = OpenOptions::new().write(true).read(true).append(true).create(true).open("time.txt").unwrap();
    // let start = SystemTime::now();
    // let datetime: DateTime<Utc> = start.into();
    // let content = String::from("HTTP3 page visit start time is: ");
    // file.write_all(content.as_bytes()).unwrap();
    // file.write_all(datetime.format("%m/%d/%Y %T%.3f\n").to_string().as_bytes()).unwrap();

    let _tracer = init_tracer();

    let http2_client = HttpClient::builder()
    .version_negotiation(VersionNegotiation::http2())
    .ssl_options(SslOption::DANGER_ACCEPT_INVALID_CERTS | SslOption::DANGER_ACCEPT_REVOKED_CERTS | SslOption::DANGER_ACCEPT_INVALID_HOSTS)
    .build()?;

    let span = global::tracer("example/client").start("say hello");
    let cx = Context::current_with_span(span);


    // let mut response = http2_client::get(url)
    // .ssl_options(SslOption::DANGER_ACCEPT_INVALID_CERTS | SslOption::DANGER_ACCEPT_REVOKED_CERTS | SslOption::DANGER_ACCEPT_INVALID_HOSTS)
    // .body(())?
    // .send()?;

    
    let mut response = http2_client.get(url)?;

    cx.span().add_event(
        "Got response!".to_string(),
        vec![KeyValue::new("status", response.status().to_string())],
    );

    // Print some basic info about the response to standard output.
    // println!("Status: {}", response.status());
    // println!("Headers: {:#?}", response.headers());

    // Read the response body as text into a string and print it.
    // print!("{}", response.text()?);
    // let end = SystemTime::now();
    // let datetime: DateTime<Utc> = end.into();
    // let content = String::from("HTTP3 page visit end time is: ");
    // file.write_all(content.as_bytes()).unwrap();
    // file.write_all(datetime.format("%m/%d/%Y %T%.3f\n").to_string().as_bytes()).unwrap();

    Ok(())
}