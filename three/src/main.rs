use opentelemetry::{baggage::BaggageExt, trace::FutureExt, KeyValue};
use playground_util::{inject_context, log, set_parent};
use tracing_opentelemetry::OpenTelemetrySpanExt;

pub mod hello_proto {
    tonic::include_proto!("hello");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    playground_util::init_traicing("three");

    let addr = "0.0.0.0:5000".parse()?;
    tracing::info!("listening on {}", addr);
    tonic::transport::Server::builder()
        .trace_fn(set_parent)
        .add_service(hello_proto::hello_server::HelloServer::new(
            MyUserService::default(),
        ))
        .serve(addr)
        .await?;

    Ok(())
}

#[derive(Debug, Default)]
struct MyUserService {}

#[tonic::async_trait]
impl hello_proto::hello_server::Hello for MyUserService {
    #[tracing::instrument]
    async fn hello(
        &self,
        _: tonic::Request<()>,
    ) -> Result<tonic::Response<hello_proto::HelloResponse>, tonic::Status> {
        playground_util::log("start hello");

        let context = tracing::Span::current()
            .context()
            .with_baggage(vec![KeyValue::new("three:hello", true)]);

        playground_util::log(&format!("before async block: {}", context.baggage()));
        async move {
            let context = tracing::Span::current().context();
            playground_util::log(&format!("in async block: {}", context.baggage()));

            let msg = f().await;
            playground_util::log("finish hello");
            Ok(tonic::Response::new(hello_proto::HelloResponse { msg }))
        }
        .with_context(context)
        .await
    }
}

#[tracing::instrument]
async fn f() -> String {
    playground_util::log("start f");
    let context = tracing::Span::current()
        .context()
        .with_baggage(vec![KeyValue::new("three:f", true)]);
    let response = g().with_context(context).await;

    playground_util::log("finish f");
    response
}

#[tracing::instrument]
async fn g() -> String {
    playground_util::log("start g");

    let mut map = http::HeaderMap::new();
    inject_context(&mut map);
    log(&format!("map={map:?}"));

    playground_util::log("finish g");
    "hello from three service".to_string()
}
