use dotenvy::dotenv;
use interprocess::local_socket::tokio::prelude::LocalSocketStream;
use interprocess::local_socket::traits::tokio::Stream;
use scorebot_common::SOCKET_NAME;
use tracing_subscriber::fmt::init;

#[tokio::main]
async fn main() {
    init();
    dotenv().ok();
    let socket_name = SOCKET_NAME.to_fs_name_if_supported()?;

    let mut stream = LocalSocketStream::connect(socket_name).await
        .expect("Could not connect to socket");
}
