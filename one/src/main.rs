use opentelemetry::{baggage::BaggageExt, trace::FutureExt, Context, KeyValue};

#[tokio::main]
async fn main() {
    playground_util::init_traicing("one");

    let app = axum::Router::new().route("/", axum::routing::get(f));
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 3000));

    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[tracing::instrument]
async fn f(header: axum::http::header::HeaderMap) -> String {
    playground_util::log("start f");
    let context = playground_util::extract_context(&header);
    let context = context.with_baggage(vec![KeyValue::new("one:f", true)]);
    let response = g().with_context(context).await;

    playground_util::log("finish f");
    response
}

#[tracing::instrument]
async fn g() -> String {
    playground_util::log("start g");
    let context =
        opentelemetry::Context::current().with_baggage(vec![KeyValue::new("one:g", true)]);
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
    let context = Context::current().with_baggage(vec![KeyValue::new("one:request", true)]);

    async move {
        let client = reqwest::Client::new();
        let mut request = client.get("http://localhost:4000/").build().unwrap();
        // playground_util::inject_context(&moving_context, request.headers_mut());
        playground_util::inject_context(request.headers_mut());

        let response = client.execute(request).await.unwrap().text().await.unwrap();

        playground_util::log("finish request");
        format!("hello from one service\n{}\n", response)
    }
    .with_context(context)
    .await
}
