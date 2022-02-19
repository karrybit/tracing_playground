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
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn hello() -> String {
    println!("requested hello in one");
    tracing::info!("requested hello in one");
    let response = match reqwest::get("http://localhost:4000/").await {
        Ok(r) => r,
        Err(e) => {
            println!("{:?}", e);
            panic!()
        }
    };
    let response = match response.text().await {
        Ok(r) => r,
        Err(e) => {
            println!("{:?}", e);
            panic!()
        }
    };
    format!("hello from one service\n{}\n", response)
}
