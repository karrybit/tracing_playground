use opentelemetry::{baggage::BaggageExt, trace::TraceContextExt};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_subscriber::prelude::*;

static HOST: once_cell::sync::Lazy<&'static str> = once_cell::sync::Lazy::new(|| {
    let host = option_env!("HOST");
    match host {
        Some("docker-compose") => "two",
        _ => "localhost",
    }
});
static THREE_URL: once_cell::sync::Lazy<String> =
    once_cell::sync::Lazy::new(|| format!("http://{}:5000/", *HOST));

mod hello {
    tonic::include_proto!("hello");
}

#[tracing::instrument]
async fn _hello() -> String {
    let span = tracing::span::Span::current();
    tracing::info!("context in _hello: {:?}", span.context());
    tracing::info!("context in _hello: {:?}", span.context().baggage());
    tracing::info!(
        "span context in _hello: {:?}",
        span.context().span().span_context()
    );
    tracing::info!("span in _hello: {:?}", span.context().span());

    tracing::info!("start _hello");
    std::thread::sleep(std::time::Duration::from_millis(100));

    let mut client = hello::hello_client::HelloClient::connect(THREE_URL.as_str())
        .await
        .unwrap();
    let mut request = tonic::Request::new(());
    let mut header = request.metadata().clone().into_headers();

    tracing::info!("before injected request header _hello: {:?}", header);
    opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.inject_context(
            &span.context(),
            &mut opentelemetry_http::HeaderInjector(&mut header),
        )
    });
    tracing::info!("after injected request header _hello: {:?}", header);

    let metadata = tonic::metadata::MetadataMap::from_headers(header);
    *request.metadata_mut() = metadata;

    tracing::info!("request _hello: {:?}", request);
    tracing::info!("metadata _hello: {:?}", request.metadata());
    tracing::info!(
        "request header _hello: {:?}",
        request.metadata().clone().into_headers()
    );

    let response = client.hello(request).await.unwrap();

    std::thread::sleep(std::time::Duration::from_millis(100));
    tracing::info!("finish _hello");

    format!("hello from two service\n{}\n", response.get_ref().msg)
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

    tracing::info!("start hello_inner");
    std::thread::sleep(std::time::Duration::from_millis(100));
    tracing::info!("finish hello_inner");
}

#[tracing::instrument]
async fn hello(header: axum::http::header::HeaderMap) -> String {
    tracing::info!("header: {:?}", header);
    let parent_ctx = opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.extract(&opentelemetry_http::HeaderExtractor(&header))
    });

    tracing::info!("parent context in hello: {:?}", parent_ctx);
    tracing::info!("parent baggage in hello: {:?}", parent_ctx.baggage());
    tracing::info!(
        "parent span context in hello: {:?}",
        parent_ctx.span().span_context()
    );
    tracing::info!("parent span in hello: {:?}", parent_ctx.span());

    let span = tracing::span::Span::current();
    span.set_parent(parent_ctx);
    tracing::info!("context in hello: {:?}", span.context());
    tracing::info!("context in hello: {:?}", span.context().baggage());
    tracing::info!(
        "span context in hello: {:?}",
        span.context().span().span_context()
    );
    tracing::info!("span in hello: {:?}", span.context().span());

    tracing::info!("start hello");

    span.in_scope(|| hello_inner());

    std::thread::sleep(std::time::Duration::from_millis(100));
    let response = span.in_scope(|| _hello()).await;
    std::thread::sleep(std::time::Duration::from_millis(100));

    tracing::info!("finish hello");
    response
}

#[tokio::main]
async fn main() {
    opentelemetry::global::set_text_map_propagator(
        opentelemetry::sdk::propagation::TextMapCompositePropagator::new(vec![
            Box::new(opentelemetry::sdk::propagation::BaggagePropagator::new()),
            Box::new(opentelemetry::sdk::propagation::TraceContextPropagator::new()),
        ]),
    );
    let tracing_layer = tracing_opentelemetry::layer().with_tracer(
        opentelemetry_jaeger::new_pipeline()
            .with_service_name("two")
            .install_simple()
            .unwrap(),
    );
    let format_layer = tracing_subscriber::fmt::layer().with_writer(std::io::stdout);
    tracing_subscriber::Registry::default()
        .with(tracing_layer)
        .with(format_layer.json().flatten_event(true))
        .with(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();
    tracing::info!("start server");

    let app = axum::Router::new().route("/", axum::routing::get(hello));
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 4000));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
