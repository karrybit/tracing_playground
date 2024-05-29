use opentelemetry::{baggage::BaggageExt, trace::FutureExt, Context, KeyValue};
use tracing_opentelemetry::OpenTelemetrySpanExt;

mod hello {
    tonic::include_proto!("hello");
}

#[tokio::main]
async fn main() {
    playground_util::init_traicing("two");

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
    let parent_context = playground_util::extract_context(&header);
    playground_util::log("start hello");

    let span = tracing::span::Span::current();
    span.set_parent(parent_context);
    let context = span
        .context()
        .with_baggage(vec![KeyValue::new("two:hello", 1)]);
    let response = request_three().with_context(context).await;

    playground_util::log("finish hello");
    response
}

#[tracing::instrument]
async fn request_three() -> String {
    let context = Context::current().with_baggage(vec![KeyValue::new("two::request_three", 2)]);
    playground_util::log("start request_three");

    let moving_context = context.clone();
    async move {
        let mut client = hello::hello_client::HelloClient::connect("http://localhost:5000")
            .await
            .unwrap();
        let mut request = tonic::Request::new(());
        let mut header = request.metadata().clone().into_headers();

        playground_util::inject_context(&moving_context, &mut header);

        let metadata = tonic::metadata::MetadataMap::from_headers(header);
        *request.metadata_mut() = metadata;

        let response = client.hello(request).await.unwrap();

        playground_util::log("finish request_three");
        format!("hello from two service\n{}\n", response.get_ref().msg)
    }
    .with_context(context)
    .await
}
