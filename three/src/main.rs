use playground_util::set_parent;

pub mod hello_proto {
    tonic::include_proto!("hello");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    playground_util::init_traicing("three");

    let addr = "0.0.0.0:5000".parse()?;
    tracing::info!("listening on {}", addr);
    tonic::transport::Server::builder()
        .trace_fn(set_parent)
        .add_service(hello_proto::hello_server::HelloServer::new(
            MyUserService::default(),
        ))
        .serve(addr)
        .await?;

    Ok(())
}

#[derive(Debug, Default)]
struct MyUserService {}

#[tonic::async_trait]
impl hello_proto::hello_server::Hello for MyUserService {
    #[tracing::instrument]
    async fn hello(
        &self,
        _: tonic::Request<()>,
    ) -> Result<tonic::Response<hello_proto::HelloResponse>, tonic::Status> {
        playground_util::log("start hello");

        playground_util::set_baggage("three:hello");

        let msg = f().await;
        playground_util::log("finish hello");

        Ok(tonic::Response::new(hello_proto::HelloResponse { msg }))
    }
}

#[tracing::instrument]
async fn f() -> String {
    playground_util::log("start f");
    playground_util::set_baggage("three:f");

    let response = g().await;

    playground_util::log("finish f");
    response
}

#[tracing::instrument]
async fn g() -> String {
    playground_util::log("start g");

    playground_util::set_baggage("three:g");

    playground_util::log("finish g");
    "hello from three service".to_string()
}
