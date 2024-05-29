use opentelemetry::{baggage::BaggageExt, trace::FutureExt, Context, KeyValue};

#[tokio::main]
async fn main() {
    playground_util::init_traicing("one");

    let app = axum::Router::new().route("/", axum::routing::get(hello));
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 3000));

    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[tracing::instrument]
async fn hello(header: axum::http::header::HeaderMap) -> String {
    let context = playground_util::extract_context(&header);
    playground_util::log("start in hello");

    let context = context.with_baggage(vec![KeyValue::new("one:hello", 1)]);
    let response = request_two().with_context(context).await;

    playground_util::log("finish in hello");
    response
}

#[tracing::instrument]
async fn request_two() -> String {
    let context = Context::current().with_baggage(vec![KeyValue::new("one:request_two", 2)]);
    playground_util::log("start in request_two");

    let moving_context = context.clone();
    async move {
        let client = reqwest::Client::new();
        let mut request = client.get("http://localhost:4000/").build().unwrap();
        playground_util::inject_context(&moving_context, request.headers_mut());

        let response = client.execute(request).await.unwrap().text().await.unwrap();

        playground_util::log("finish in request_two");
        format!("hello from one service\n{}\n", response)
    }
    .with_context(context)
    .await
}
