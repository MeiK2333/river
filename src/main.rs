#![recursion_limit = "512"]
#[macro_use]
extern crate log;

use futures::StreamExt;
use futures_core::Stream;
use log4rs;
use river::judge_request::Data;
use river::river_server::{River, RiverServer};
use river::{
    Empty, JudgeRequest, JudgeResponse, JudgeResultEnum, LanguageConfigResponse, LanguageItem,
    LsRequest, LsResponse,
};
use std::path::Path;
use std::pin::Pin;
use tempfile::tempdir_in;
use tokio::fs::read_dir;
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
            debug!("{:?}", pwd);
            // 是否通过编译
            let mut compile_success = false;
            let mut language = String::from("");
            while let Some(req) = stream.next().await {
                yield result::pending();
                // TODO: 限制并发数量
                yield result::running();
                let req = req?;
                let result = match &req.data {
                    Some(Data::CompileData(data)) => {
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

    async fn language_config(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<LanguageConfigResponse>, Status> {
        let mut languages: Vec<LanguageItem> = vec![];
        for (key, value) in &config::CONFIG.languages {
            languages.push(LanguageItem {
                language: String::from(key),
                compile: String::from(&value.compile_cmd),
                run: String::from(&value.run_cmd),
                version: String::from(&value.version),
            });
        }
        let response = LanguageConfigResponse {
            languages: languages,
        };
        Ok(Response::new(response))
    }

    async fn ls(&self, request: Request<LsRequest>) -> Result<Response<LsResponse>, Status> {
        // TODO: 将获取文件的接口剥离出来，归入评测文件管理系统中
        // TODO: 最终将会删除这个接口
        // TODO: 目前有安全隐患，可以获取到任意目录文件
        let dir = request.into_inner().dir;
        let mut response = LsResponse { files: vec![] };
        let directory_stream = match read_dir(Path::new("runtime/data").join(dir)).await {
            Ok(val) => val,
            Err(_) => return Ok(Response::new(response)),
        };
        let files: Vec<_> = directory_stream
            .filter_map(|file| async move {
                Some(file.unwrap().file_name().into_string().unwrap())
            })
            .collect()
            .await;
        response.files = files;
        Ok(Response::new(response))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();

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
