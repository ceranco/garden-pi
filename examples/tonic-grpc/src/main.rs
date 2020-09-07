/// This examples demonstrates how to use the tonic crate to easily use
/// gRPC and Protobuf.
///
/// It creates two different tokio runtimes, one for the server and one for the client.
/// The runtimes run on their own threads, with the client signaling the server when to shutdown
/// (using a oneshot channel).
use futures::{channel::oneshot, FutureExt};
use hello_tonic::{
    greeter_client::GreeterClient,
    greeter_server::{Greeter, GreeterServer},
    HelloReply, HelloRequest,
};
use std::{error::Error, thread, time::Duration};
use tonic::{transport::Server, Request, Response, Status};

pub mod hello_tonic {
    tonic::include_proto!("tonic_grpc");
}

#[derive(Debug, Default)]
pub struct GreeterImpl {}

#[tonic::async_trait]
impl Greeter for GreeterImpl {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        println!("Got request: {:?}", request);

        let reply = HelloReply {
            message: format!("Hello {}!", request.into_inner().name),
        };

        Ok(Response::new(reply))
    }
}

async fn server_main(receiver: oneshot::Receiver<()>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let addr = "[::1]:50051".parse()?;
    let greeter = GreeterImpl::default();

    Server::builder()
        .add_service(GreeterServer::new(greeter))
        .serve_with_shutdown(addr, receiver.map(|_| ()))
        .await?;

    Ok(())
}

async fn client_main(sender: oneshot::Sender<()>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut client = GreeterClient::connect("http://[::1]:50051").await?;

    let names = ["Joe", "John", "Tommy", "Sandra", "Rita", "Carla", "Sammy"];
    for name in &names {
        let request = tonic::Request::new(HelloRequest {
            name: (*name).to_owned(),
        });
        let response = client.say_hello(request).await?;

        println!("RESPONSE={:?}", response);
        std::thread::sleep(Duration::from_millis(500));
    }

    sender.send(()).expect("Failed to send shutdown signal");
    Ok(())
}

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let (sender, receiver) = oneshot::channel::<()>();

    let server_handle = thread::spawn(move || {
        let mut scheduler = tokio::runtime::Builder::new()
            .threaded_scheduler()
            .enable_all()
            .build()
            .unwrap();

        scheduler.block_on(server_main(receiver))
    });
    let client_handle = thread::spawn(|| {
        let mut scheduler = tokio::runtime::Builder::new()
            .basic_scheduler()
            .threaded_scheduler()
            .enable_all()
            .build()
            .unwrap();

        scheduler.block_on(client_main(sender))
    });

    server_handle.join().unwrap()?;
    client_handle.join().unwrap()?;

    Ok(())
}
