use std::env;
use opentelemetry::{global, trace::{Span, Tracer}, KeyValue};
use isahc::{
    prelude::*,
    config::{VersionNegotiation, SslOption},
    HttpClient,
    Metrics
};

fn main() -> Result<(), isahc::Error>   {
    let args: Vec<String> = env::args().collect();
    let url = &args[1];

    global::set_text_map_propagator(opentelemetry_jaeger::Propagator::new());
    let tracer = opentelemetry_jaeger::new_agent_pipeline().with_service_name("http2_client").install_simple().unwrap();

    // start a new span
    let mut span = tracer.start("http2_fetch_webpage");

    let http2_client = HttpClient::builder()
        .version_negotiation(VersionNegotiation::http2())
        .ssl_options(SslOption::DANGER_ACCEPT_INVALID_CERTS | SslOption::DANGER_ACCEPT_REVOKED_CERTS | SslOption::DANGER_ACCEPT_INVALID_HOSTS)
        .metrics(true)
        .build().unwrap();

    let req_start = std::time::SystemTime::now();
    let mut response = http2_client.get(url).unwrap();
    span.add_event("request_completed".to_owned(), vec![]);

    span.add_event_with_timestamp("request_start", req_start, vec![]);
    //Get the total time from the start of the request until DNS name resolving was completed
    span.add_event_with_timestamp(
        "name_lookup",
        req_start + Metrics::name_lookup_time(response.metrics().unwrap()),
        vec![KeyValue::new(
            "name_lookup_time",
            format_duration(Metrics::name_lookup_time(response.metrics().unwrap())),
        )],
    );
    //Get the amount of time spent on TLS handshakes
    span.add_event_with_timestamp(
        "secure_connect",
        req_start + Metrics::secure_connect_time(response.metrics().unwrap()),
        vec![KeyValue::new(
            "secure_connect_time",
            format_duration(Metrics::secure_connect_time(response.metrics().unwrap())),
        )],
    );
    //Get the amount of time taken to establish a connection to the server (not including TLS connection time)
    span.add_event_with_timestamp(
        "connect",
        req_start + Metrics::connect_time(response.metrics().unwrap()),
        vec![KeyValue::new(
            "connect_time",
            format_duration(Metrics::connect_time(response.metrics().unwrap())),
        )],
    );
    //Get the time it took from the start of the request until the first byte is either sent or received
    span.add_event_with_timestamp(
        "transfer_start",
        req_start + Metrics::transfer_start_time(response.metrics().unwrap()),
        vec![KeyValue::new(
            "transfer_start_time",
            format_duration(Metrics::transfer_start_time(response.metrics().unwrap())),
        )],
    );
    //transfer time: Get the amount of time spent performing the actual request transfer. The “transfer” includes both sending the request and receiving the response
    //total time: Get the total time for the entire request. This will continuously increase until the entire response body is consumed and completed.
    span.add_event_with_timestamp(
        "transfer_end",
        req_start + Metrics::total_time(response.metrics().unwrap()),
        vec![
            KeyValue::new(
                "total_time",
                format_duration(Metrics::total_time(response.metrics().unwrap()))
            ),
            KeyValue::new(
                "transfer_time",
                format_duration(Metrics::transfer_time(response.metrics().unwrap()))
            )
        ]);
    //If automatic redirect following is enabled, gets the total time taken for all redirection steps including name lookup, connect, pretransfer and transfer before final transaction was started.
    span.add_event_with_timestamp(
        "redirect",
        req_start + Metrics::redirect_time(response.metrics().unwrap()),
        vec![KeyValue::new(
            "redirect_time",
            format_duration(Metrics::redirect_time(response.metrics().unwrap())),
        )],
    );

    print!("{}", response.text().unwrap());
    // end or drop the span to export
    span.end();
    // export remaining spans
    global::shutdown_tracer_provider(); 
    Ok(())
}

fn format_duration(duration: std::time::Duration) -> String {
    let ns = duration.as_nanos();
    if ns >= 1_000_000_000 {
        // seconds
        format!(
            "{}.{:03}s",
            ns / 1_000_000_000,
            ns.rem_euclid(1_000_000_000) / 1_000_000
        )
    } else if ns >= 1_000_000 {
        // ms
        format!("{}.{:03}ms", ns / 1_000_000, ns.rem_euclid(1_000_000) / 1_000)
    } else if ns >= 1_000 {
        // us
        format!("{}.{:03}us", ns / 1_000, ns.rem_euclid(1_000))
    } else {
        // ns
        format!("{}ns", ns)
    }
}