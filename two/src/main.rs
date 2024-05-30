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
    playground_util::set_baggage_with_context(parent_context, "two:f");

    let response = g().await;

    playground_util::log("finish f");
    response
}

#[tracing::instrument]
async fn g() -> String {
    playground_util::log("start g");
    playground_util::set_baggage("two:g");

    let response = request().await;
    playground_util::log("finish g");
    response
}

#[tracing::instrument]
async fn request() -> String {
    playground_util::set_baggage("two:request");

    let mut client = hello::hello_client::HelloClient::connect("http://localhost:5000")
        .await
        .unwrap();
    let mut request = tonic::Request::new(());
    let mut header = request.metadata().clone().into_headers();

    playground_util::inject_context(&mut header);

    let metadata = tonic::metadata::MetadataMap::from_headers(header);
    *request.metadata_mut() = metadata;

    let response = client.hello(request).await.unwrap();

    playground_util::log("finish request");
    format!("hello from two service\n{}\n", response.get_ref().msg)
}
