use opentelemetry::{baggage::BaggageExt, trace::TraceContextExt};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_subscriber::prelude::*;

pub mod hello_proto {
    tonic::include_proto!("hello");
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

#[derive(Debug, Default)]
struct MyUserService {}

#[tonic::async_trait]
impl hello_proto::hello_server::Hello for MyUserService {
    #[tracing::instrument]
    async fn hello(
        &self,
        request: tonic::Request<()>,
    ) -> Result<tonic::Response<hello_proto::HelloResponse>, tonic::Status> {
        tracing::info!("request in hello: {:?}", request);
        tracing::info!("metadata in hello: {:?}", request.metadata());

        let parent_ctx = opentelemetry::global::get_text_map_propagator(|propagator| {
            propagator.extract(&opentelemetry_http::HeaderExtractor(
                &request.metadata().clone().into_headers(),
            ))
        });
        tracing::info!("parent ctx in call: {:?}", parent_ctx);
        tracing::info!("parent baggage in call: {:?}", parent_ctx.baggage());
        tracing::info!(
            "span context in hello: {:?}",
            parent_ctx.span().span_context()
        );
        tracing::info!("parent span ctx in call: {:?}", parent_ctx.span());

        let span = tracing::span::Span::current();
        span.set_parent(parent_ctx);

        tracing::info!("context in hello: {:?}", span.context());
        tracing::info!("baggage in hello: {:?}", span.context().baggage());
        tracing::info!(
            "span context in hello: {:?}",
            span.context().span().span_context()
        );
        tracing::info!("span in hello: {:?}", span);

        hello_inner();

        tracing::info!("start hello");
        std::thread::sleep(std::time::Duration::from_millis(100));
        tracing::info!("finish hello");

        Ok(tonic::Response::new(hello_proto::HelloResponse {
            msg: "hello from three service".to_string(),
        }))
    }
}

async fn start_server() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:5000".parse()?;
    tracing::info!("listening on {}", addr);
    tonic::transport::Server::builder()
        // .add_service(tracing_interceptor(
        //     hello_proto::hello_server::HelloServer::new(MyUserService::default()),
        // ))
        .add_service(hello_proto::hello_server::HelloServer::new(
            MyUserService::default(),
        ))
        .serve(addr)
        .await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    opentelemetry::global::set_text_map_propagator(
        opentelemetry::sdk::propagation::TextMapCompositePropagator::new(vec![
            Box::new(opentelemetry::sdk::propagation::BaggagePropagator::new()),
            Box::new(opentelemetry::sdk::propagation::TraceContextPropagator::new()),
        ]),
    );
    let tracing_layer = tracing_opentelemetry::layer().with_tracer(
        opentelemetry_jaeger::new_pipeline()
            .with_service_name("three")
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
    start_server().await?;

    Ok(())
}
