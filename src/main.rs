use futures_core::Stream;
use river::river_server::{River, RiverServer};
use river::{JudgeRequest, JudgeResponse};
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::mpsc;
use tonic::{transport::Server, Request, Response, Status};

pub mod river {
    tonic::include_proto!("river"); // The string specified here must match the proto package name
}

#[derive(Debug, Default)]
pub struct RiverService {}

#[tonic::async_trait]
impl River for RiverService {
    type JudgeStream =
        Pin<Box<dyn Stream<Item = Result<JudgeResponse, Status>> + Send + Sync + 'static>>;

    async fn judge(
        &self,
        request: Request<tonic::Streaming<JudgeRequest>>,
    ) -> Result<Response<Self::JudgeStream>, Status> {
        let mut stream = request.into_inner();

        let output = async_stream::try_stream! {
            // while let Some(note) = stream.next().await {
                yield river::JudgeResponse {
                    time_used: 1,
                    memory_used: 2,
                    result: 0,
                    errno: 0,
                    exit_code: 0,
                    stdout: "stdout".into(),
                    stderr: "stderr".into(),
                };
            // }
        };

        Ok(Response::new(Box::pin(output) as Self::JudgeStream))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let greeter = RiverService::default();

    Server::builder()
        .add_service(RiverServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}
