/// This example demonstrates how to use grpc for communcating and "controlling" a
/// scheduler thread. This thread will be the one controlling the raspberry pi
/// according to the recieved plan / instructions.
///
/// The thread will act as a very simple polling scheduler. It will wake up each
/// second and set the state as per a global variable.
/// This solution is very simple as the thread has no need for any data aside from
/// the shared plan / instruction variable.
use std::{
    error::Error,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use grpc::counter_stopper_server::{CounterStopper, CounterStopperServer};
use tonic::{transport::Server, Code, Request, Response, Status};

pub mod grpc {
    tonic::include_proto!("grpc_worker");
}

#[derive(Debug)]
pub struct CounterData {
    current_count: u32,
    stopped: bool,
}

impl CounterData {
    pub fn new() -> Self {
        Self {
            current_count: 0,
            stopped: false,
        }
    }
}

#[derive(Debug)]
pub struct CounterImpl {
    data: Arc<Mutex<CounterData>>,
}

impl CounterImpl {
    pub fn new(data: Arc<Mutex<CounterData>>) -> Self {
        Self { data }
    }
}

#[tonic::async_trait]
impl CounterStopper for CounterImpl {
    async fn stop(&self, _: Request<()>) -> Result<Response<()>, Status> {
        {
            let mut data = self
                .data
                .lock()
                .map_err(|_| Status::new(Code::Unknown, "Mutex was poisoned"))?;
            data.stopped = true;
        }
        Ok(Response::new(()))
    }

    async fn resume(&self, _: Request<()>) -> Result<Response<()>, Status> {
        {
            let mut data = self
                .data
                .lock()
                .map_err(|_| Status::new(Code::Unknown, "Mutex was poisoned"))?;
            data.stopped = false;
        }
        Ok(Response::new(()))
    }
}

fn scheduler_proc(data: Arc<Mutex<CounterData>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    loop {
        let current_count = {
            let mut data = data.lock().map_err(|_| "Mutex was poisoned")?;
            if !data.stopped {
                data.current_count += 1;
            }
            data.current_count
        };
        println!("Count: {}", current_count);

        thread::sleep(Duration::from_secs(1));
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let data = Arc::new(Mutex::new(CounterData::new()));

    let addr = "127.0.0.1:42042".parse()?;
    let counter = CounterImpl::new(data.clone());

    let handle = thread::spawn(move || scheduler_proc(data));
    Server::builder()
        .add_service(CounterStopperServer::new(counter))
        .serve(addr)
        .await?;

    handle.join().unwrap()?;
    Ok(())
}
