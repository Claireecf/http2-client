// use std::fs::OpenOptions;
// use chrono::DateTime;
// use chrono::offset::Utc;
// use std::io::Write;
// use std::{time::SystemTime};
use std::env;
use isahc::prelude::*;
use isahc::{config::SslOption};

use opentelemetry::{global, trace::{Span, Tracer}, KeyValue};
use tracing::{error, span};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

use isahc::{
    config::VersionNegotiation,
    HttpClient,
};

fn main() -> Result<(), isahc::Error>   {
    // Send a GET request and wait for the response headers.
    // Must be `mut` so we can read the response body.
    // HTTP/2 with prior knowledge.
    global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());
    let tracer = opentelemetry_jaeger::new_agent_pipeline().with_service_name("http2_client").install_simple().unwrap();

    // Create a tracing layer with the configured tracer
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer.clone());
    // Use the tracing subscriber `Registry`, or any other subscriber
    // that impls `LookupSpan`
    let subscriber = Registry::default().with(telemetry);

    let args: Vec<String> = env::args().collect();
    let url = &args[1];
    // let mut file = OpenOptions::new().write(true).read(true).append(true).create(true).open("time.txt").unwrap();
    // let start = SystemTime::now();
    // let datetime: DateTime<Utc> = start.into();
    // let content = String::from("HTTP3 page visit start time is: ");
    // file.write_all(content.as_bytes()).unwrap();
    // file.write_all(datetime.format("%m/%d/%Y %T%.3f\n").to_string().as_bytes()).unwrap();

    // start a new span
    let mut span = tracer.start("http2_fetch_webpage");

    // Trace executed code
    tracing::subscriber::with_default(subscriber, || {
        // Spans will be sent to the configured OpenTelemetry exporter
        let root = span!(tracing::Level::TRACE, "app_start", work_units = 2);
        let _enter = root.enter();
        let http2_client = HttpClient::builder()
        .version_negotiation(VersionNegotiation::http2())
        .ssl_options(SslOption::DANGER_ACCEPT_INVALID_CERTS | SslOption::DANGER_ACCEPT_REVOKED_CERTS | SslOption::DANGER_ACCEPT_INVALID_HOSTS)
        .build().unwrap();


        let mut response = http2_client.get(url).unwrap();
        // Print some basic info about the response to standard output.
        println!("Status: {}", response.status());
        println!("Headers: {:#?}", response.headers());
        // Read the response body as text into a string and print it.
        print!("{}", response.text().unwrap());
        error!("This event will be logged in the root span.");
    });
    // end or drop the span to export
    span.end();
    // export remaining spans
    global::shutdown_tracer_provider(); 

    // let end = SystemTime::now();
    // let datetime: DateTime<Utc> = end.into();
    // let content = String::from("HTTP3 page visit end time is: ");
    // file.write_all(content.as_bytes()).unwrap();
    // file.write_all(datetime.format("%m/%d/%Y %T%.3f\n").to_string().as_bytes()).unwrap();
    Ok(())
}