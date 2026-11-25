use std::time::Duration;

use ferrodock_core::{AppError, AppResult, ErrorKind};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

const TIMEOUT: Duration = Duration::from_secs(3);
const REQUEST: &[u8] =
    b"GET /api/v1/health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";

pub async fn probe(address: &str) -> AppResult<()> {
    let status_line = tokio::time::timeout(TIMEOUT, fetch_status_line(address))
        .await
        .map_err(AppError::internal)??;
    if status_line.split_whitespace().nth(1) == Some("200") {
        Ok(())
    } else {
        Err(AppError::new(ErrorKind::Unavailable, "HEALTHCHECK_FAILED")
            .with_param("status", status_line))
    }
}

async fn fetch_status_line(address: &str) -> AppResult<String> {
    let mut stream = TcpStream::connect(address)
        .await
        .map_err(AppError::internal)?;
    stream
        .write_all(REQUEST)
        .await
        .map_err(AppError::internal)?;
    let mut buffer = [0u8; 64];
    let read = stream.read(&mut buffer).await.map_err(AppError::internal)?;
    let head = String::from_utf8_lossy(&buffer[..read]);
    Ok(head.lines().next().unwrap_or_default().to_owned())
}
