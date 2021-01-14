#![recursion_limit = "512"]
#[macro_use]
extern crate log;

use env_logger::Env;
use futures::StreamExt;
use futures_core::Stream;
use river::judge_request::Data;
use river::river_server::{River, RiverServer};
use river::{JudgeRequest, JudgeResponse, JudgeResultEnum};
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
mod result;
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

        let output = async_stream::try_stream! {
            let pwd = match tempdir_in(&config::CONFIG.judge_dir) {
                Ok(val) => val,
                Err(e) => {
                    yield result::system_error(error::Error::IOError(e));
                    return;
                }
            };
            // 是否通过编译
            let mut compile_success = false;
            while let Some(req) = stream.next().await {
                yield result::pending();
                // TODO: 限制并发数量
                yield result::running();
                let req = req?;
                let mut language = String::from("");
                let result = match &req.data {
                    Some(Data::CompileData(data)) => {
                        debug!("compile request");
                        // 因为评测时还需要 language 的信息，因此此处进行复制保存
                        language = String::from(&data.language);
                        let res = judger::compile(&language, &data.code, &pwd.path()).await;
                        // 判断编译结果
                        if let Ok(ref val) = res {
                            if let Some(river::judge_response::State::Result(rst)) = &val.state {
                                if rst.result == JudgeResultEnum::CompileSuccess as i32 {
                                    // 标记编译成功
                                    compile_success = true;
                                }
                            }
                        }
                        res
                    },
                    Some(Data::JudgeData(data)) => {
                        debug!("judge request");
                        // 必须通过编译才能运行
                        if language == "" || !compile_success {
                            Err(error::Error::CustomError(String::from("not compiled")))
                        } else {
                            judger::judge(
                                &language,
                                &data.in_file,
                                &data.out_file,
                                data.time_limit,
                                data.memory_limit,
                                data.judge_type,
                                &pwd.path()
                            ).await
                        }
                    },
                    None => Err(error::Error::CustomError(String::from("unrecognized request types"))),
                    _ => Err(error::Error::CustomError(String::from("unrecognized request types"))),
                };
                let res = match result {
                    Ok(res) => res,
                    Err(e) => result::system_error(e)
                };
                yield res;
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
