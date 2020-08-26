#![recursion_limit = "256"]

use crate::river::judge_response::JudgeResult;
use futures::StreamExt;
use futures_core::Stream;
use river::river_server::{River, RiverServer};
use river::{JudgeRequest, JudgeResponse};
use std::pin::Pin;
use tonic::transport::Server;
use tonic::{Request, Response, Status};

mod error;
mod judger;
mod process;

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

        let output = async_stream::try_stream! {
            let mut need_compile = true;
            while let Some(req) = stream.next().await {
                // TODO: 使用锁或者资源量等机制限制并发
                yield judger::pending();
                let req = req?;

                // 首次获取流进行编译
                if need_compile {
                    yield judger::compiling();
                    let result = match judger::compile(&req).await {
                        Ok(res) => res,
                        Err(e) => error::system_error(e)
                    };
                    // 如果编译错误，则不进行后续流程
                    if result.result != JudgeResult::Accepted as i32 {
                        yield result;
                        break;
                    }
                }
                need_compile = false;

                yield judger::running();
                let result = match judger::judger(&req).await {
                    Ok(res) => res,
                    Err(e) => error::system_error(e)
                };

                yield result;
            }
            while let Some(_) = stream.next().await {}
        };

        Ok(Response::new(Box::pin(output) as Self::JudgeStream))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:4003".parse()?;
    let greeter = RiverService::default();

    Server::builder()
        .add_service(RiverServer::new(greeter))
        .serve(addr)
        .await?;

    Ok(())
}
