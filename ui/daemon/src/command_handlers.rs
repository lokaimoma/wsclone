use tokio::io::{AsyncRead, AsyncWrite};

pub async fn handle_connection<T>(stream: T)
where
    T: AsyncRead + AsyncWrite + Send,
{
}
