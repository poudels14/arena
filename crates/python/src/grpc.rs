tonic::include_proto!("grpc");

#[cfg(feature = "grpc-client")]
pub use self::python_runtime_client as client;
pub use self::python_runtime_server as server;
