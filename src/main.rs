#![recursion_limit = "256"]
#[macro_use]
extern crate log;

use env_logger::Env;
use futures::StreamExt;
use futures_core::Stream;
use river::judge_request::Data;
use river::river_server::{River, RiverServer};
use river::{JudgeRequest, JudgeResponse};
use std::pin::Pin;
use tempfile::tempdir_in;
use tonic::transport::Server;
use tonic::{Request, Response, Status};

mod config;
mod error;

mod cgroup;
mod exec_args;
mod judger;
mod process;
mod seccomp;

pub mod river {
    tonic::include_proto!("river");
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

        // 此处编译很慢
        // 为啥，是 try_stream! 这个宏导致的吗？
        // 同时内部代码无法被 cargo fmt 格式化
        // why?
        let output = async_stream::try_stream! {
            while let Some(req) = stream.next().await {
                    let req = req?;
                // TODO
                let pwd = tempdir_in("/tmp").unwrap();
                match &req.data {
                    Some(Data::CompileData(data)) => {
                        debug!("compile request");
                        let language = &data.language;
                        let code = &data.code;
                        debug!("language: {}", language);
                        debug!("code: {}", code);
                        break;
                    },
                    Some(Data::JudgeData(data)) => {
                        debug!("judge request");
                        let in_file = &data.in_file;
                        let out_file = &data.out_file;
                        let time_limit = &data.time_limit;
                        let memory_limit = &data.memory_limit;
                        let judge_type = &data.judge_type;
                        debug!("in_file: {}", in_file);
                        debug!("out_file: {}", out_file);
                        debug!("time_limit: {}", time_limit);
                        debug!("memory_limit: {}", memory_limit);
                        debug!("judge_type: {}", judge_type);
                        break;
                    },
                    None => break,
                    _ => break,
                };
            }
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
