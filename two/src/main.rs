use opentelemetry::{baggage::BaggageExt, trace::TraceContextExt};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_subscriber::prelude::*;

static THREE_URL: once_cell::sync::Lazy<String> =
    once_cell::sync::Lazy::new(|| std::env::var("THREE_URL").unwrap());

mod hello {
    tonic::include_proto!("hello");
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
        opentelemetry_jaeger::new_agent_pipeline()
            .with_service_name("two")
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
    tracing::info!("start server");

    let app = axum::Router::new().route("/", axum::routing::get(hello));
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 4000));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
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

    hello_inner();

    std::thread::sleep(std::time::Duration::from_millis(100));
    let response = request_three().await;
    std::thread::sleep(std::time::Duration::from_millis(100));

    tracing::info!("finish hello");
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

    tracing::info!("start hello_inner");
    std::thread::sleep(std::time::Duration::from_millis(100));
    tracing::info!("finish hello_inner");
}

#[tracing::instrument]
async fn request_three() -> String {
    let span = tracing::span::Span::current();
    tracing::info!("context in request_three: {:?}", span.context());
    tracing::info!("context in request_three: {:?}", span.context().baggage());
    tracing::info!(
        "span context in request_three: {:?}",
        span.context().span().span_context()
    );
    tracing::info!("span in request_three: {:?}", span.context().span());

    tracing::info!("start request_three");
    std::thread::sleep(std::time::Duration::from_millis(100));

    let mut client = hello::hello_client::HelloClient::connect(THREE_URL.as_str())
        .await
        .unwrap();
    let mut request = tonic::Request::new(());
    let mut header = request.metadata().clone().into_headers();

    tracing::info!("before injected request header request_three: {:?}", header);
    opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.inject_context(
            &span.context(),
            &mut opentelemetry_http::HeaderInjector(&mut header),
        )
    });
    tracing::info!("after injected request header request_three: {:?}", header);

    let metadata = tonic::metadata::MetadataMap::from_headers(header);
    *request.metadata_mut() = metadata;

    tracing::info!("request request_three: {:?}", request);
    tracing::info!("metadata request_three: {:?}", request.metadata());
    tracing::info!(
        "request header request_three: {:?}",
        request.metadata().clone().into_headers()
    );

    let response = client.hello(request).await.unwrap();

    std::thread::sleep(std::time::Duration::from_millis(100));
    tracing::info!("finish request_three");

    format!("hello from two service\n{}\n", response.get_ref().msg)
}
