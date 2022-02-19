use tonic::{transport::Server, Request, Response, Status};
use user_proto::{
    hello_server::{Hello, HelloServer},
    HelloResponse,
};

pub mod user_proto {
    tonic::include_proto!("hello");
}

#[derive(Debug, Default)]
struct MyUserService {}

#[tonic::async_trait]
impl Hello for MyUserService {
    async fn hello(&self, _request: Request<()>) -> Result<Response<HelloResponse>, Status> {
        println!("requested hello in three");
        Ok(Response::new(HelloResponse {
            msg: "hello from three service".to_string(),
        }))
    }
}

pub async fn start_server() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:5000".parse()?;
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
    tracing_subscriber::fmt::init();
    println!("start server");
    start_server().await?;
    Ok(())
}
