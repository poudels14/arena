use pyo3::Python;
use tokio::sync::mpsc;
use tonic::{Request, Response, Status};

use crate::grpc::{
  server::PythonRuntime, ExecCodeRequest, ExecCodeResponse, ExecResult,
  HealthyRequest, HealthyResponse,
};
use crate::runtime::context::Context;

#[derive(Default)]
pub struct RuntimeServer {}

#[tonic::async_trait]
impl PythonRuntime for RuntimeServer {
  async fn healthy(
    &self,
    _request: Request<HealthyRequest>,
  ) -> Result<Response<HealthyResponse>, Status> {
    let reply = HealthyResponse { healthy: true };
    Ok(Response::new(reply))
  }

  async fn exec_code(
    &self,
    request: Request<ExecCodeRequest>,
  ) -> Result<Response<ExecCodeResponse>, Status> {
    let request = request.into_inner();
    let (tx, mut rx) = mpsc::channel(1);
    rayon::spawn(move || {
      pyo3::prepare_freethreaded_python();
      let res = Python::with_gil(|py| {
        let context = Context::new(py)?;
        let res = context.exec(&request.code);

        let stdout = context.stdout()?;
        let stderr = context.stderr()?;
        match res {
          Ok(data) => {
            let _ = tx.blocking_send(ExecCodeResponse {
              success: true,
              data: data.map(|d| ExecResult {
                r#type: d.r#type,
                value: d.value,
              }),
              stdout,
              stderr,
              error: None,
            });
            Ok::<(), anyhow::Error>(())
          }
          Err(err) => {
            let _ = tx.blocking_send(ExecCodeResponse {
              success: false,
              data: None,
              stdout,
              stderr,
              error: Some(err.to_string()),
            });
            return Err(err);
          }
        }
      });

      if let Err(err) = res {
        let _ = tx.blocking_send(ExecCodeResponse {
          success: false,
          data: None,
          stdout: "".to_owned(),
          stderr: "".to_owned(),
          error: Some(err.to_string()),
        });
      }
    });

    let response = rx
      .recv()
      .await
      .ok_or_else(|| Status::unknown("Error getting execution response"))?;
    Ok(Response::new(response))
  }
}
