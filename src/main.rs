#![recursion_limit = "512"]
#[macro_use]
extern crate log;

use env_logger::Env;
use futures::StreamExt;
use futures_core::Stream;
use river::judge_request::Data;
use river::judge_response::State;
use river::river_server::{River, RiverServer};
use river::{JudgeRequest, JudgeResponse, JudgeResult};
use std::pin::Pin;
use tempfile::tempdir_in;
use tonic::transport::Server;
use tonic::{Request, Response, Status};

mod config;
mod error;
mod exec_args;
mod judger;
mod process;
mod runner;

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
            // 创建评测使用的临时目录
            // 无需主动删除，变量在 drop 之后会自动删除临时文件夹
            let pwd = match tempdir_in("./runner") {
                Ok(val) => val,
                Err(err) => {
                    yield error::system_error(error::Error::CreateTempDirError(err));
                    return
                },
            };
            // 是否通过编译
            let mut compile_success = false;
            debug!("create tempdir: {:?}", pwd);

            while let Some(req) = stream.next().await {
                // TODO: 使用锁或者资源量等机制限制并发
                yield judger::pending();

                let req = req?;

                yield judger::running();
                let result = match &req.data {
                    Some(Data::CompileData(data)) => {
                        debug!("compile request");
                        judger::compile(&req, &data, &pwd.path()).await
                    },
                    Some(Data::JudgeData(data)) => {
                        debug!("judge request");
                        // 必须通过编译才能运行
                        if !compile_success {
                            Ok(error::system_error(error::Error::CustomError("Please compile first".to_string())))
                        } else {
                            judger::judger(&req, &data, &pwd.path()).await
                        }
                    },
                    None => Err(error::Error::RequestDataNotFound),
                    _ => Err(error::Error::UnknownRequestData),
                };
                let result = match result {
                    Ok(res) => res,
                    Err(e) => error::system_error(e)
                };
                // 如果通过了编译，则标记为成功
                if let Some(Data::CompileData(_)) = &req.data {
                    if let Some(State::Result(rst)) = result.state {
                        if rst == JudgeResult::Accepted as i32 {
                            compile_success = true;
                        }
                    }
                }
                yield result;
            }
            while let Some(_) = stream.next().await {}
        };

        Ok(Response::new(Box::pin(output) as Self::JudgeStream))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "debug")
        .write_style_or("MY_LOG_STYLE", "always");

    env_logger::init_from_env(env);

    let addr = "127.0.0.1:4003".parse()?;
    let river = RiverService::default();

    info!("listen on: {}", addr);

    Server::builder()
        .concurrency_limit_per_connection(5)
        .add_service(RiverServer::new(river))
        .serve(addr)
        .await?;

    Ok(())
}
