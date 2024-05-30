use opentelemetry::{baggage::BaggageExt, trace::FutureExt, KeyValue};
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
        request: tonic::Request<()>,
    ) -> Result<tonic::Response<hello_proto::HelloResponse>, tonic::Status> {
        playground_util::log("start hello");
        let parent_context =
            playground_util::extract_context(&request.metadata().clone().into_headers());

        let span = tracing::span::Span::current();
        span.set_parent(parent_context);

        let context = span
            .context()
            .with_baggage(vec![KeyValue::new("three", "hello")]);

        async move {
            playground_util::log("finish hello");
            Ok(tonic::Response::new(hello_proto::HelloResponse {
                msg: "hello from three service".to_string(),
            }))
        }
        .with_context(context)
        .await
    }
}
