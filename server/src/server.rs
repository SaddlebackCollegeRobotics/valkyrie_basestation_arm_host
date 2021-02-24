use rover_postcard_protocol::Driver;
use rover_tonic::borealis::kinematic_arm_state_servicer_server::{
    KinematicArmStateServicer, KinematicArmStateServicerServer,
};
use rover_tonic::borealis::{ArmState, Empty};
use std::path::Path;
use tokio::sync::mpsc;
use tokio_serial::{DataBits, FlowControl, Parity, Serial, SerialPortSettings, StopBits};
use tonic::{transport::Server, Request, Response, Status};

pub struct KinematicArmServer {
    driver: Driver,
}

impl KinematicArmServer {
    pub fn new(path: &str) -> anyhow::Result<Self> {
        Ok(Self {
            driver: Driver::new(Serial::from_path(
                path,
                &SerialPortSettings {
                    baud_rate: 9600,
                    data_bits: DataBits::Five,
                    flow_control: FlowControl::None,
                    parity: Parity::None,
                    stop_bits: StopBits::One,
                    timeout: Default::default(),
                },
            )?),
        })
    }
}

#[tonic::async_trait]
impl KinematicArmStateServicer for KinematicArmServer {
    async fn get_arm_state(&self, request: Request<Empty>) -> Result<Response<ArmState>, Status> {
        let hardware_response = self
            .driver
            .do_hardware_action(rover_postcard_protocol::rover_postcards::Request {
                kind: rover_postcard_protocol::rover_postcards::RequestKind::GetKinematicArmPose,
                state: rand::random(),
            })
            .await
            // need to manually map the error type as they arn't compatible
            .or(Err(Status::aborted("failed to interrogate model arm.")))?;
        if let Some(rover_postcard_protocol::rover_postcards::ResponseKind::KinematicArmPose(
            pose,
        )) = hardware_response.data
        {
            Ok(Response::new(ArmState {
                lower_axis: pose.lower_axis,
                upper_axis: pose.upper_axis,
                rotation: pose.rotation_axis,
                gripper: None,
                driving_arm: true,
                driving_gripper: false,
            }))
        } else {
            Err(Status::aborted("invalid response from model arm hardware."))
        }
    }
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    //let addr = "[::1]:50051".parse().unwrap();
    let addr = "[::]:50051".parse().unwrap();
    let server = KinematicArmServer::new("/dev/ttyS0")?;
    println!("Server listening on {}", addr);
    Server::builder()
        .add_service(KinematicArmStateServicerServer::new(server))
        .serve(addr)
        .await?;
    Ok(())
}