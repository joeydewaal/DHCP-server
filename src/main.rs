#![allow(dead_code)]
use std::net::Ipv4Addr;

use error::DHCPError;
use leases::LeaseRange;
use server::{Client, Server};
use state::DHCPState;
use tokio::task;

use crate::{
    handlers::{on_dhcp_discover, on_dhcp_request},
    packet::DHCPMessageType,
    standard::{BROADCAST_ADDR, SERVER_PORT},
};

mod buffer;
mod error;
mod handlers;
mod leases;
mod packet;
mod server;
mod standard;
mod state;

#[tokio::main]
async fn main() -> Result<(), error::DHCPError> {
    tracing_subscriber::fmt().init();

    let mut server = Server::start().await?;

    let lease_range = LeaseRange::new(
        Ipv4Addr::new(192, 168, 56, 3),
        Ipv4Addr::new(192, 168, 56, 255),
        Ipv4Addr::new(192, 168, 56, 1),
        Ipv4Addr::new(255, 255, 255, 0),
    );

    let server_state = DHCPState::from_lease(lease_range);
    tracing::info!("Server started: {}:{}", BROADCAST_ADDR, SERVER_PORT);

    loop {
        let client = server.receive().await?;

        let state = server_state.clone();
        let _ = task::spawn(handle_request(client, state));
    }
}

async fn handle_request(client: Client, state: DHCPState) -> Result<(), DHCPError> {
    let response = match client.packet.dhcp_message_type {
        DHCPMessageType::DHCPDISCOVER => on_dhcp_discover(client.packet.clone(), state)?,
        DHCPMessageType::DHCPREQUEST => on_dhcp_request(client.packet.clone(), state)?,
        _ => unimplemented!(),
    };

    client.send_back(response).await;
    Ok(())
}
