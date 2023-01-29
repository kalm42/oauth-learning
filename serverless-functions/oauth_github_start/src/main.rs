use lambda_runtime::{Error, service_fn, LambdaEvent};
use serde::{Deserialize, Serialize};
use log::{info, error};

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

// Implement Display for the Failure response so that we can then implement Error.
impl std::fmt::Display for FailureResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.body)
    }
}

// Implement Error for the FailureResponse so that we can `?` (try) the Response
// returned by `lambda_runtime::run(func).await` in `fn main`.
impl std::error::Error for FailureResponse {}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = service_fn(handler);
    lambda_runtime::run(func).await?;

    Ok(())
}

async fn handler(event: LambdaEvent<Request>) -> Result<SuccessResponse, FailureResponse> {
    info!("handling a request...");

    let bucket_name = std::env::var("BUCKET_NAME")
        .expect("A BUCKET_NAME must be set in the environmental variables!");
    
    let config = aws_config::load_from_env().await;
    let s3_client = aws_sdk_s3::Client::new(&config);
    let filename = format!("{}.txt", time::OffsetDateTime::now_utc().unix_timestamp());

    let _ = s3_client
        .put_object()
        .bucket(bucket_name)
        .body(event.payload.body.as_bytes().to_owned().into())
        .key(&filename)
        .send()
        .await
        .map_err(|err| {
            error!("failed to upload file to '{}' to S3 with error: {}", &filename, err);
            FailureResponse {
                body: "The lambda encountered an error while uploading the file to S3.".to_owned(),
            }
        })?;
    info!("Successfully stored the incoming request in S3 with the name '{}'", &filename);

    Ok(SuccessResponse {
        body: format!(
            "the lambda has successfully stored your request in S3 with the name '{}'",
            filename
        ),
    })
}