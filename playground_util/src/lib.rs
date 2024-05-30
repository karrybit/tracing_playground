use opentelemetry::{baggage::BaggageExt, trace::TraceContextExt, KeyValue};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_subscriber::{layer::SubscriberExt as _, util::SubscriberInitExt as _};

pub fn init_traicing(service_name: &str) {
    opentelemetry::global::set_text_map_propagator(
        opentelemetry_sdk::propagation::TextMapCompositePropagator::new(vec![
            Box::new(opentelemetry_sdk::propagation::BaggagePropagator::new()),
            Box::new(opentelemetry_sdk::propagation::TraceContextPropagator::new()),
        ]),
    );

    let env_filter_layer =
        tracing_subscriber::EnvFilter::try_new("info").expect("failed to create EnvFilter");

    if true {
        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_batch_config(
                opentelemetry_sdk::trace::BatchConfig::default()
                    .with_max_queue_size(2_048 * 4)
                    .with_max_export_batch_size(512 * 4),
            )
            .with_exporter(opentelemetry_otlp::new_exporter().tonic())
            .with_trace_config(opentelemetry_sdk::trace::config().with_resource(
                opentelemetry_sdk::Resource::new(vec![opentelemetry::KeyValue::new(
                    opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                    service_name.to_string(),
                )]),
            ))
            .install_batch(opentelemetry_sdk::runtime::Tokio)
            .expect("failed to install batch");

        let opentelemetry = tracing_opentelemetry::layer().with_tracer(tracer);
        let fmt_layer = tracing_subscriber::fmt::layer().with_writer(std::io::stdout);
        // .with_file(true)
        // .with_line_number(true)
        // .with_thread_names(true)
        // .with_thread_ids(true)
        // .with_target(true)
        // .with_timer(tracing_subscriber::fmt::time::UtcTime::rfc_3339())
        // .with_level(true);
        tracing_subscriber::registry()
            .with(opentelemetry)
            .with(env_filter_layer)
            .with(fmt_layer.json())
            .init();
    } else {
        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_writer(std::io::stdout)
            .with_file(true)
            .with_line_number(true)
            .with_thread_names(true)
            .with_thread_ids(true)
            .with_target(true)
            .with_timer(tracing_subscriber::fmt::time::UtcTime::rfc_3339())
            .with_level(true);
        tracing_subscriber::Registry::default()
            .with(env_filter_layer)
            .with(fmt_layer.pretty())
            .init()
    }
}

pub fn set_parent(request: &http::Request<()>) -> tracing::Span {
    let context = opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.extract(&opentelemetry_http::HeaderExtractor(request.headers()))
    });

    let root_span = tracing::info_span!("root span",);

    root_span.set_parent(context);
    root_span
}

pub fn inject_context(header: &mut http::HeaderMap) {
    let context = tracing::span::Span::current().context();
    opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.inject_context(&context, &mut opentelemetry_http::HeaderInjector(header))
    });
}

pub fn extract_context(header: &http::HeaderMap) -> opentelemetry::Context {
    opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.extract(&opentelemetry_http::HeaderExtractor(header))
    })
}

pub fn set_baggage(key: &str) {
    let span = tracing::span::Span::current();
    let context = span
        .context()
        .with_baggage(vec![KeyValue::new(key.to_string(), true)]);
    span.set_parent(context);
}

pub fn set_baggage_with_context(context: opentelemetry::Context, key: &str) {
    let context = context.with_baggage(vec![KeyValue::new(key.to_string(), true)]);
    let span = tracing::span::Span::current();
    span.set_parent(context);
}

pub fn log(message: &str) {
    let span = tracing::span::Span::current();
    let context = span.context();
    let span_ref = context.span();
    let span_context = span_ref.span_context();
    let trace_id = span_context.trace_id();
    let span_id = span_context.span_id();
    let baggage = context.baggage();
    tracing::info!("[{trace_id}:{span_id}] 👺 {message} 😈 {context:?} 👜 {baggage}");
}
