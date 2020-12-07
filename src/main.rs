#![recursion_limit = "256"]
#[macro_use]
extern crate log;

use env_logger::Env;
use futures_core::Stream;
use river::river_server::{River, RiverServer};
use river::{JudgeRequest, JudgeResponse, UploadFile, UploadState};
use std::path::Path;
use std::pin::Pin;
use tonic::transport::Server;
use tonic::{Request, Response, Status};

mod error;
mod file;

pub mod river {
    tonic::include_proto!("river");
}

#[derive(Debug, Default)]
pub struct RiverService {}

#[tonic::async_trait]
impl River for RiverService {
    type JudgeStream =
        Pin<Box<dyn Stream<Item = Result<JudgeResponse, Status>> + Send + Sync + 'static>>;

    // 上传文件接口
    async fn upload(&self, request: Request<UploadFile>) -> Result<Response<UploadState>, Status> {
        let upload_file = request.into_inner();

        // 文件放在 judger/data/ 目录下
        let prefix_path = Path::new("judger/data/");
        let path = prefix_path.join(&upload_file.filepath);

        let result = file::extract(&path, &upload_file.data);
        let state = match result {
            Ok(_) => river::UploadState {
                state: Some(river::upload_state::State::Filepath(
                    upload_file.filepath.to_string(),
                )),
            },
            Err(e) => river::UploadState {
                state: Some(river::upload_state::State::Errmsg(format!("{}", e))),
            },
        };
        Ok(Response::new(state))
    }

    async fn judge(
        &self,
        request: Request<tonic::Streaming<JudgeRequest>>,
    ) -> Result<Response<Self::JudgeStream>, Status> {
        let mut _stream = request.into_inner();

        let output = async_stream::try_stream! {
            yield JudgeResponse {
                state: Some(river::judge_response::State::Status(river::JudgeStatus::Pending as i32))
            };
        };

        Ok(Response::new(Box::pin(output) as Self::JudgeStream))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env = Env::default()
        .filter_or("LOG_LEVEL", "debug,h2=info,hyper=info")
        .write_style_or("LOG_STYLE", "always");

    env_logger::init_from_env(env);

    let addr = "0.0.0.0:4003".parse()?;
    let river = RiverService::default();

    info!("listen on: {}", addr);

    Server::builder()
        .concurrency_limit_per_connection(5)
        .add_service(RiverServer::new(river))
        .serve(addr)
        .await?;

    Ok(())
}
