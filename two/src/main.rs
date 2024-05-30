use opentelemetry::{baggage::BaggageExt, trace::FutureExt, Context, KeyValue};
use tracing_opentelemetry::OpenTelemetrySpanExt;

mod hello {
    tonic::include_proto!("hello");
}

#[tokio::main]
async fn main() {
    playground_util::init_traicing("two");

    let app = axum::Router::new().route("/", axum::routing::get(f));
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 4000));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[tracing::instrument]
async fn f(header: axum::http::header::HeaderMap) -> String {
    let parent_context = playground_util::extract_context(&header);
    playground_util::log("start f");

    let span = tracing::span::Span::current();
    span.set_parent(parent_context);
    let context = span
        .context()
        .with_baggage(vec![KeyValue::new("two:f", true)]);
    let response = g().with_context(context).await;

    playground_util::log("finish f");
    response
}

#[tracing::instrument]
async fn g() -> String {
    playground_util::log("start g");

    let context =
        opentelemetry::Context::current().with_baggage(vec![KeyValue::new("two:g", true)]);
    async move {
        let response = request().await;
        playground_util::log("finish g");
        response
    }
    .with_context(context)
    .await
}

#[tracing::instrument]
async fn request() -> String {
    playground_util::log("start request");
    let context = Context::current().with_baggage(vec![KeyValue::new("two:request", true)]);

    async move {
        let mut client = hello::hello_client::HelloClient::connect("http://localhost:5000")
            .await
            .unwrap();
        let mut request = tonic::Request::new(());
        let mut header = request.metadata().clone().into_headers();

        // playground_util::inject_context(&moving_context, &mut header);
        playground_util::inject_context(&mut header);

        let metadata = tonic::metadata::MetadataMap::from_headers(header);
        *request.metadata_mut() = metadata;

        let response = client.hello(request).await.unwrap();

        playground_util::log("finish request");
        format!("hello from two service\n{}\n", response.get_ref().msg)
    }
    .with_context(context)
    .await
}
