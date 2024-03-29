use opentelemetry::{baggage::BaggageExt, trace::TraceContextExt};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_subscriber::prelude::*;

static TWO_URL: once_cell::sync::Lazy<String> =
    once_cell::sync::Lazy::new(|| std::env::var("TWO_URL").unwrap());

#[tokio::main]
async fn main() {
    opentelemetry::global::set_text_map_propagator(
        opentelemetry::sdk::propagation::TextMapCompositePropagator::new(vec![
            Box::new(opentelemetry::sdk::propagation::BaggagePropagator::new()),
            Box::new(opentelemetry::sdk::propagation::TraceContextPropagator::new()),
        ]),
    );

    let tracing_layer = tracing_opentelemetry::layer().with_tracer(
        opentelemetry_jaeger::new_agent_pipeline()
            .with_service_name("one")
            .install_simple()
            .unwrap(),
    );
    let format_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stdout)
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_thread_names(true);
    tracing_subscriber::Registry::default()
        .with(tracing_layer)
        .with(format_layer.json().flatten_event(true))
        .with(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();

    let app = axum::Router::new().route("/", axum::routing::get(hello));
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[tracing::instrument]
async fn hello(_header: axum::http::header::HeaderMap) -> String {
    let span = tracing::span::Span::current();
    tracing::info!("context in hello: {:?}", span.context());
    tracing::info!("baggage in hello: {:?}", span.context().baggage());
    tracing::info!(
        "span context in hello: {:?}",
        span.context().span().span_context()
    );
    tracing::info!("context in hello: {:?}", span.context());

    tracing::info!("start in hello");

    hello_inner();
    tokio::spawn(async_hello(span.context().span().span_context().clone()));

    std::thread::sleep(std::time::Duration::from_millis(100));
    let response = request_two().await;
    std::thread::sleep(std::time::Duration::from_millis(100));

    tracing::info!("finish in hello");
    response
}

#[tracing::instrument]
fn hello_inner() {
    let span = tracing::span::Span::current();
    tracing::info!("context in hello_inner: {:?}", span.context());
    tracing::info!("baggage in hello_inner: {:?}", span.context().baggage());
    tracing::info!(
        "span context in hello_inner: {:?}",
        span.context().span().span_context()
    );
    tracing::info!("span in hello_inner: {:?}", span.context().span());

    tracing::info!("start in hello_inner");
    std::thread::sleep(std::time::Duration::from_millis(100));
    tracing::info!("finish in hello_inner");
}

#[tracing::instrument]
async fn async_hello(span_context: opentelemetry::trace::SpanContext) {
    tracing::info!("linked span context in async_hello: {:?}", span_context);
    let span = tracing::span::Span::current();
    span.add_link(span_context);

    std::thread::sleep(std::time::Duration::from_millis(2000));

    tracing::info!("context in async_hello: {:?}", span.context());
    tracing::info!("baggage in async_hello: {:?}", span.context().baggage());
    tracing::info!(
        "span context in async_hello: {:?}",
        span.context().span().span_context()
    );
    tracing::info!("span in async_hello: {:?}", span.context().span());

    std::thread::sleep(std::time::Duration::from_millis(500));
}

#[tracing::instrument]
async fn request_two() -> String {
    let span = tracing::span::Span::current();
    tracing::info!("context in request_two: {:?}", span.context());
    tracing::info!("baggage in request_two: {:?}", span.context().baggage());
    tracing::info!(
        "span context in request_two: {:?}",
        span.context().span().span_context()
    );
    tracing::info!("span in request_two: {:?}", span.context().span());

    let ctx = span
        .context()
        .with_baggage(vec![opentelemetry::KeyValue::new("my-name", "my-value")]);

    tracing::info!("start request_two");
    std::thread::sleep(std::time::Duration::from_millis(100));

    let client = reqwest::Client::new();
    let mut request = client.get(TWO_URL.as_str()).build().unwrap();

    opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.inject_context(
            &ctx,
            &mut opentelemetry_http::HeaderInjector(request.headers_mut()),
        )
    });
    tracing::info!("header in request_two: {:?}", request.headers());

    let response = match client.execute(request).await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("{:?}", e);
            panic!()
        }
    };
    let response = match response.text().await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("{:?}", e);
            panic!()
        }
    };
    std::thread::sleep(std::time::Duration::from_millis(100));
    tracing::info!("finish request_two");

    format!("hello from one service\n{}\n", response)
}
