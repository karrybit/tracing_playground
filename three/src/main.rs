use hello::{
    hello_server::{Hello, HelloServer},
    HelloResponse,
};
use tonic::{transport::Server, Request, Response, Status};
use tracing_subscriber::prelude::*;

pub mod hello {
    tonic::include_proto!("hello");
}

#[derive(Debug, Default)]
struct MyUserService {}

#[tonic::async_trait]
impl Hello for MyUserService {
    async fn hello(&self, _request: Request<()>) -> Result<Response<HelloResponse>, Status> {
        tracing::info!("requested hello in three");
        Ok(Response::new(HelloResponse {
            msg: "hello from three service".to_string(),
        }))
    }
}

pub async fn start_server() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:5000".parse()?;
    tracing::info!("listening on {}", addr);
    Server::builder()
        .add_service(HelloServer::new(MyUserService::default()))
        .serve(addr)
        .await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // initialize tracing
    let layer = tracing_subscriber::fmt::layer().with_writer(std::io::stdout);
    tracing_subscriber::Registry::default()
        .with(layer.pretty())
        .init();
    tracing::info!("start server");
    start_server().await?;
    Ok(())
}
