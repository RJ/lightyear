//! Transport using the WebTransport protocol (based on QUIC)
cfg_if::cfg_if! {
    if #[cfg(all(feature = "webtransport", target_family = "wasm"))] {
            pub mod client_wasm;
            pub use client_wasm as client;
    } else if #[cfg(all(feature = "webtransport", not(target_family = "wasm")))]{
            pub mod server;
            pub mod client_native;
            pub use client_native as client;
    }
}

// Maximum transmission units; maximum size in bytes of a UDP packet
// See: https://gafferongames.com/post/packet_fragmentation_and_reassembly/
const MTU: usize = 1472;

#[cfg(test)]
mod tests {
    use super::client::*;
    use super::server::*;
    use crate::transport::{PacketReceiver, PacketSender, Transport};
    use bevy::tasks::{IoTaskPool, TaskPoolBuilder};
    use std::time::Duration;
    use tracing::info;
    use tracing_subscriber::fmt::format::FmtSpan;
    use wtransport::tls::Certificate;

    #[cfg(not(target_family = "wasm"))]
    #[tokio::test]
    async fn test_webtransport_native() -> anyhow::Result<()> {
        // tracing_subscriber::FmtSubscriber::builder()
        //     .with_span_events(FmtSpan::ENTER)
        //     .with_max_level(tracing::Level::INFO)
        //     .init();
        let certificate = Certificate::self_signed(["localhost"]);
        let server_addr = "127.0.0.1:7000".parse().unwrap();
        let client_addr = "127.0.0.1:8000".parse().unwrap();

        let mut client_socket = WebTransportClientSocket::new(client_addr, server_addr);
        let mut server_socket = WebTransportServerSocket::new(server_addr, certificate);

        let (mut server_send, mut server_recv) = server_socket.listen();
        let (mut client_send, mut client_recv) = client_socket.listen();

        let msg = b"hello world";

        // client to server
        client_send.send(msg, &server_addr)?;

        // sleep a little to give time to the message to arrive in the socket
        tokio::time::sleep(Duration::from_millis(20)).await;

        let Some((recv_msg, address)) = server_recv.recv()? else {
            panic!("server expected to receive a packet from client");
        };
        assert_eq!(address, client_addr);
        assert_eq!(recv_msg, msg);

        // server to client
        server_send.send(msg, &client_addr)?;

        // sleep a little to give time to the message to arrive in the socket
        tokio::time::sleep(Duration::from_millis(20)).await;

        let Some((recv_msg, address)) = client_recv.recv()? else {
            panic!("client expected to receive a packet from server");
        };
        assert_eq!(address, server_addr);
        assert_eq!(recv_msg, msg);
        dbg!(recv_msg);
        Ok(())
    }
}

#[cfg(target_family = "wasm")]
#[cfg(test)]
pub mod wasm_test {
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    #[tokio::test]
    async fn test_webtransport_wasm() -> anyhow::Result<()> {
        // tracing_subscriber::FmtSubscriber::builder()
        //     .with_span_events(FmtSpan::ENTER)
        //     .with_max_level(tracing::Level::INFO)
        //     .init();
        let certificate = Certificate::self_signed(["localhost"]);
        let server_addr = "127.0.0.1:7000".parse().unwrap();
        let client_addr = "127.0.0.1:8000".parse().unwrap();

        let mut client_socket = WebTransportClientSocket::new(client_addr, server_addr);
        let mut server_socket = WebTransportServerSocket::new(server_addr, certificate);

        let (mut server_send, mut server_recv) = server_socket.listen();
        let (mut client_send, mut client_recv) = client_socket.listen();

        let msg = b"hello world";

        // client to server
        client_send.send(msg, &server_addr)?;

        // sleep a little to give time to the message to arrive in the socket
        tokio::time::sleep(Duration::from_millis(20)).await;

        let Some((recv_msg, address)) = server_recv.recv()? else {
            panic!("server expected to receive a packet from client");
        };
        assert_eq!(address, client_addr);
        assert_eq!(recv_msg, msg);

        // server to client
        server_send.send(msg, &client_addr)?;

        // sleep a little to give time to the message to arrive in the socket
        tokio::time::sleep(Duration::from_millis(20)).await;

        let Some((recv_msg, address)) = client_recv.recv()? else {
            panic!("client expected to receive a packet from server");
        };
        assert_eq!(address, server_addr);
        assert_eq!(recv_msg, msg);
        dbg!(recv_msg);
        ok(())
    }
}
