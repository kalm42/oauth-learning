use lambda_http::{Response, tower::BoxError};
use lambda_runtime::{Error, service_fn, LambdaEvent};
use serde::{Deserialize, Serialize};
use log::info;


#[derive(Deserialize)]
struct Request {
    pub body: String,
}

#[derive(Debug, Serialize)]
struct SuccessResponse {
    pub body: String,
}

#[derive(Debug, Serialize)]
struct FailureResponse {
    pub body: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = service_fn(handler);
    lambda_runtime::run(func).await?;

    Ok(())
}

async fn handler(event: LambdaEvent<Request>) -> Result<SuccessResponse, BoxError> {
    info!("handling a request...");

    Ok(SuccessResponse {
        body: format!(
            "the lambda has successfully stored your request in S3 with the name ",
        ),
    })
}