use core::net::SocketAddr;

use std::env;
use std::process::ExitCode;

use log::error;

use tonic::transport::{server::Router, Server};
use tonic::Status;

use memdatabase::memdatabase::v1::memory_database_service_server::MemoryDatabaseServiceServer;

use memdatabase::chan::btree::svc::chan_svc_new_default;

const LISTEN_ADDR_DEFAULT: &str = "0.0.0.0:50051";

async fn sub() -> Result<(), Status> {
    let mem_svc = chan_svc_new_default().await;
    let mem_svr: MemoryDatabaseServiceServer<_> = MemoryDatabaseServiceServer::new(mem_svc);

    let mut server: Server = Server::builder();
    let router: Router<_> = server.add_service(mem_svr);

    let listen_addr: String = env::var("ENV_LISTEN_ADDR")
        .ok()
        .unwrap_or_else(|| LISTEN_ADDR_DEFAULT.into());
    let sa: SocketAddr = str::parse(listen_addr.as_str())
        .map_err(|e| Status::invalid_argument(format!("invalid listen addr: {e}")))?;

    router
        .serve(sa)
        .await
        .map_err(|e| Status::internal(format!("unable to listen: {e}")))?;
    Ok(())
}

#[tokio::main]
async fn main() -> ExitCode {
    sub().await.map(|_| ExitCode::SUCCESS).unwrap_or_else(|e| {
        error!("{e}");
        ExitCode::FAILURE
    })
}
