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
async fn f() -> String {
    playground_util::log("start f");
    playground_util::set_baggage("one:f");

    let response = g().await;

    playground_util::log("finish f");
    response
}

#[tracing::instrument]
async fn g() -> String {
    playground_util::log("start g");
    playground_util::set_baggage("one:g");

    let response = request().await;

    playground_util::log("finish g");
    response
}

#[tracing::instrument]
async fn request() -> String {
    playground_util::log("start request");
    playground_util::set_baggage("one:request");

    let client = reqwest::Client::new();
    let mut request = client.get("http://localhost:4000/").build().unwrap();
    playground_util::inject_context(request.headers_mut());

    let response = client.execute(request).await.unwrap().text().await.unwrap();

    playground_util::log("finish request");
    format!("hello from one service\n{}\n", response)
}
