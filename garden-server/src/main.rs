use std::{
    collections::HashMap,
    error::Error,
    sync::{Arc, Mutex},
    time::Duration,
};

use chrono::Local;
use garden_rpc::{
    garden_pi_server::{GardenPi, GardenPiServer},
    set_valve_state_response, GetModeResponse, GetScheduleResponse, GetValveStateRequest,
    GetValveStateResponse, GetValveStatesResponse, Mode, Schedule, SetModeRequest,
    SetScheduleRequest, SetScheduleResponse, SetValveStateRequest, SetValveStateResponse, Timespan,
    Timestamp, Valve, ValveState,
};
use tonic::{transport::Server, Request, Response, Status};

/// This struct will be shared between the worker thread and the server. The server will
/// update it and the worker thread will periodically update the open valves accordingly.
struct GardenState {
    mode: Mode,
    // This array is used to save the valve states when using `Mode::Manual`.
    manual_valve_states: [ValveState; 8],
    // The schedule that is used when using `Mode::Scheduled`.
    schedule: Schedule,
}

impl GardenState {
    fn new() -> Self {
        Self {
            mode: Mode::Scheduled,
            manual_valve_states: [ValveState::Off; 8],
            schedule: Schedule {
                valve1: Vec::new(),
                valve2: Vec::new(),
                valve3: Vec::new(),
                valve4: Vec::new(),
                valve5: Vec::new(),
                valve6: Vec::new(),
                valve7: Vec::new(),
                valve8: Vec::new(),
            },
        }
    }

    fn get_current_valves(&self) -> [ValveState; 8] {
        [ValveState::Off; 8]
    }
}

/// A mock implementation of the garden server.
/// This is to be used for testing.
struct GardenPiImpl {
    state: Arc<Mutex<GardenState>>,
}

impl GardenPiImpl {
    fn new(state: Arc<Mutex<GardenState>>) -> Self {
        Self { state }
    }

    //     fn show(&self) -> () {
    //         print!(
    //             "\n\n---------------------------------------------------------------------------------\n|"
    //         );
    //         for i in 0..8 {
    //             let valve = Valve::from_i32(i).expect("There should be 8 valves, numbered from 0 to 7");
    //             let state = *self
    //                 .valves
    //                 .lock()
    //                 .expect("Mutex shouldn't be poisoned")
    //                 .get(&valve)
    //                 .expect("All valves have state");
    //
    //             let state_string = match state {
    //                 ValveState::On => "On ",
    //                 ValveState::Off => "Off",
    //             };
    //             print!("   {}   |", state_string);
    //         }
    //         print!("   {}", Local::now().format("[%d-%b-%Y](%H:%M:%S)"));
    //         println!(
    //             "\n---------------------------------------------------------------------------------"
    //         );
    //     }
}

#[tonic::async_trait]
impl GardenPi for GardenPiImpl {
    async fn set_mode(&self, request: Request<SetModeRequest>) -> Result<Response<()>, Status> {
        let new_mode = request.into_inner().mode();
        self.state.lock().unwrap().mode = new_mode;

        Ok(Response::new(()))
    }

    async fn get_mode(&self, _: Request<()>) -> Result<Response<GetModeResponse>, Status> {
        let current_mode = self.state.lock().unwrap().mode;

        let mut response = GetModeResponse::default();
        response.set_mode(current_mode);
        Ok(Response::new(response))
    }

    async fn set_schedule(
        &self,
        request: Request<SetScheduleRequest>,
    ) -> Result<Response<SetScheduleResponse>, Status> {
        let request = request.into_inner();
        let new_schedule = request.schedule.unwrap();
        self.state.lock().unwrap().schedule = new_schedule;

        let mut response = SetScheduleResponse::default();
        response.success = true;
        Ok(Response::new(response))
    }

    async fn get_schedule(&self, _: Request<()>) -> Result<Response<GetScheduleResponse>, Status> {
        let schedule = self.state.lock().unwrap().schedule.clone();
        let mut response = GetScheduleResponse::default();
        response.schedule = Some(schedule);

        Ok(Response::new(response))
    }

    async fn set_valve_state(
        &self,
        request: Request<SetValveStateRequest>,
    ) -> Result<Response<SetValveStateResponse>, Status> {
        let request = request.into_inner();

        let mut response = SetValveStateResponse::default();
        response.success = {
            let mut state = self.state.lock().unwrap();
            match state.mode {
                Mode::Scheduled => false,
                Mode::Manual => {
                    state.manual_valve_states[request.valve as usize] = request.state();
                    true
                }
            }
        };

        Ok(Response::new(response))
    }

    async fn get_valve_state(
        &self,
        request: Request<GetValveStateRequest>,
    ) -> Result<Response<GetValveStateResponse>, Status> {
        let request = request.into_inner();

        let mut response = GetValveStateResponse::default();
        let valve_state = {
            let state = self.state.lock().unwrap();
            state.get_current_valves()[request.valve() as usize]
        };
        response.set_state(valve_state);
        let response = Response::new(response);

        Ok(response)
    }

    async fn get_valve_states(
        &self,
        _request: Request<()>,
    ) -> Result<Response<GetValveStatesResponse>, Status> {
        let mut response = GetValveStatesResponse::default();
        {
            let current_valves = self.state.lock().unwrap().get_current_valves();
            response.set_valve1_state(current_valves[0]);
            response.set_valve2_state(current_valves[1]);
            response.set_valve3_state(current_valves[2]);
            response.set_valve4_state(current_valves[3]);
            response.set_valve5_state(current_valves[4]);
            response.set_valve6_state(current_valves[5]);
            response.set_valve7_state(current_valves[6]);
            response.set_valve8_state(current_valves[7]);
        }
        let response = Response::new(response);

        Ok(response)
    }
}

fn worker_thread(state: Arc<Mutex<GardenState>>, interval: Duration) {
    loop {
        // Get the current state of the valves.
        let valve_states = {
            let state = state.lock().unwrap();
            state.get_current_valves()
        };

        // Update the real valves.
        let mut line = String::new();
        for valve_state in &valve_states {
            let state = match valve_state {
                ValveState::Off => "| Off ",
                ValveState::On => "| On  ",
            };
            line.push_str(state);
        }
        // Create the separator **before** appending the date, so that the
        // chars count is accurate.
        let separator = "-".repeat(line.chars().count() + 1);
        line.push_str(&format!("| {}", Local::now().format("%d %h %Y [%H:%M:%S]")));
        println!("{}", separator);
        println!("{}", line);
        println!("{}\n\n", separator);

        std::thread::sleep(interval);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let address = "[::1]:50050".parse()?;
    let state = Arc::new(Mutex::new(GardenState::new()));
    let garden_pi = GardenPiImpl::new(state.clone());

    let worker_state = state.clone();
    std::thread::spawn(move || worker_thread(worker_state, Duration::from_secs(5)));

    Server::builder()
        .add_service(GardenPiServer::new(garden_pi))
        .serve(address)
        .await?;

    Ok(())
}
