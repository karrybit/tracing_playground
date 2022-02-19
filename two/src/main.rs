pub mod hello {
    tonic::include_proto!("hello");
}

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = axum::Router::new()
        // `GET /` goes to `root`
        .route("/", axum::routing::get(hello));

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 4000));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// basic handler that responds with a static string
async fn hello() -> String {
    println!("requested hello in two");
    tracing::info!("requested hello in two");
    let mut client = hello::hello_client::HelloClient::connect("http://localhost:5000")
        .await
        .unwrap();
    let request = tonic::Request::new(());
    let response = client.hello(request).await.unwrap();
    format!("hello from two service\n{}\n", response.get_ref().msg)
}
