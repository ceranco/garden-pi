use std::{collections::HashMap, error::Error, sync::Mutex};

use chrono::Local;
use garden_rpc::{
    garden_pi_server::{GardenPi, GardenPiServer},
    GetValveStateRequest, GetValveStateResponse, GetValveStatesResponse, SetValveStateRequest,
    Valve, ValveState,
};
use tonic::{transport::Server, Request, Response, Status};

/// A mock implementation of the garden server.
/// This is to be used for testing.
struct GardenPiMock {
    valves: Mutex<HashMap<Valve, ValveState>>,
}

impl GardenPiMock {
    fn new() -> Self {
        let mut valves = HashMap::new();
        for i in 0..8 {
            let valve = Valve::from_i32(i).expect("There should be 8 valves, numbered from 0 to 7");
            valves.insert(valve, ValveState::Off);
        }
        let valves = Mutex::new(valves);

        Self { valves }
    }

    fn show(&self) -> () {
        print!(
            "\n\n---------------------------------------------------------------------------------\n|"
        );
        for i in 0..8 {
            let valve = Valve::from_i32(i).expect("There should be 8 valves, numbered from 0 to 7");
            let state = *self
                .valves
                .lock()
                .expect("Mutex shouldn't be poisoned")
                .get(&valve)
                .expect("All valves have state");

            let state_string = match state {
                ValveState::On => "On ",
                ValveState::Off => "Off",
            };
            print!("   {}   |", state_string);
        }
        print!("   {}", Local::now().format("[%d-%b-%Y](%H:%M:%S)"));
        println!(
            "\n---------------------------------------------------------------------------------"
        );
    }
}

#[tonic::async_trait]
impl GardenPi for GardenPiMock {
    async fn set_valve_state(
        &self,
        request: Request<SetValveStateRequest>,
    ) -> Result<Response<()>, Status> {
        let request = request.into_inner();

        self.valves
            .lock()
            .expect("Mutex shouldn't be poisoned")
            .insert(request.valve(), request.state());

        self.show();

        Ok(Response::new(()))
    }

    async fn get_valve_state(
        &self,
        request: Request<GetValveStateRequest>,
    ) -> Result<Response<GetValveStateResponse>, Status> {
        let request = request.into_inner();

        let mut response = GetValveStateResponse::default();
        response.set_state(
            *self
                .valves
                .lock()
                .expect("Mutex shouldn't be poisoned")
                .get(&request.valve())
                .expect("All valves have state"),
        );
        let response = Response::new(response);

        self.show();

        Ok(response)
    }

    async fn get_valve_states(
        &self,
        _request: Request<()>,
    ) -> Result<Response<GetValveStatesResponse>, Status> {
        use Valve::*;

        let mut response = GetValveStatesResponse::default();
        {
            let valves = self.valves.lock().expect("Mutex shouldn't be poisoned");
            response.set_valve1_state(
                *valves
                    .get(&Valve1)
                    .expect("The state of all valves should exist"),
            );
            response.set_valve2_state(
                *valves
                    .get(&Valve2)
                    .expect("The state of all valves should exist"),
            );
            response.set_valve3_state(
                *valves
                    .get(&Valve3)
                    .expect("The state of all valves should exist"),
            );
            response.set_valve4_state(
                *valves
                    .get(&Valve4)
                    .expect("The state of all valves should exist"),
            );
            response.set_valve5_state(
                *valves
                    .get(&Valve5)
                    .expect("The state of all valves should exist"),
            );
            response.set_valve6_state(
                *valves
                    .get(&Valve6)
                    .expect("The state of all valves should exist"),
            );
            response.set_valve7_state(
                *valves
                    .get(&Valve7)
                    .expect("The state of all valves should exist"),
            );
            response.set_valve8_state(
                *valves
                    .get(&Valve8)
                    .expect("The state of all valves should exist"),
            );
        }
        let response = Response::new(response);

        self.show();

        Ok(response)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let address = "[::1]:50050".parse()?;
    let garden_pi = GardenPiMock::new();

    Server::builder()
        .add_service(GardenPiServer::new(garden_pi))
        .serve(address)
        .await?;

    Ok(())
}
