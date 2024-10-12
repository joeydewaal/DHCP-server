use std::net::{IpAddr, SocketAddr};

use tokio::{
    net::UdpSocket,
    sync::mpsc::{Receiver, Sender},
    task,
};

use crate::{
    error::DHCPError,
    packet::Packet,
    standard::{BROADCAST_ADDR, CLIENT_PORT, SERVER_PORT},
};

pub struct Server {
    sender: Sender<Client>,
    receiver: Receiver<Client>,
}

impl Server {
    pub async fn start() -> Result<Self, DHCPError> {
        // send client responses back
        let (sender1, mut receiver1) = tokio::sync::mpsc::channel(10);

        // receive client packets
        let (sender2, receiver2) = tokio::sync::mpsc::channel(10);

        let server = UdpSocket::bind((BROADCAST_ADDR, SERVER_PORT)).await?;
        server.set_broadcast(true)?;

        let mut buff = [0; 4096];

        let _sender1 = sender1.clone();
        task::spawn(async move {
            loop {
                tokio::select! {
                    // client packet to send back
                    packet = receiver1.recv() => {
                        let Some(packet) = packet else {
                            tracing::error!("Server closed");
                            panic!();
                        };
                        if let Err(error) = Server::server_send_back(packet, &server).await {
                            tracing::error!("Could not send packet: {error}");
                        };
                    },
                    // server receives from client
                    client = server.recv_from(&mut buff) => {
                        match client {
                            Ok((len, src)) => {
                                if let Err(error) = Server::server_receive(len, src, &buff, &sender2, _sender1.clone()).await {
                                    tracing::error!("Could receive packet: {error}");
                                };
                            },
                            Err(error) => {
                                tracing::error!("UDP SOCK error {error}");
                                continue;
                            }
                        }

                    }
                }
            }
        });

        Ok(Server {
            sender: sender1,
            receiver: receiver2,
        })
    }

    pub async fn receive(&mut self) -> Result<Client, DHCPError> {
        Ok(self.receiver.recv().await.expect("Receiv channel closed"))
    }

    async fn server_receive(
        len: usize,
        src: SocketAddr,
        buff: &[u8],
        sender2: &Sender<Client>,
        _sender1: Sender<Client>
    ) -> Result<(), DHCPError> {
        if len == 0 {
            tracing::error!("Received empty packet");
        }

        let packet = Packet::try_from(&buff[..len])?;

        if sender2
            .send(Client {
                packet,
                src,
                sender: _sender1,
            })
            .await.is_err() {
                tracing::error!("Channel closed");
                panic!();
            };
        Ok(())
    }

    async fn server_send_back(client: Client, server: &UdpSocket) -> Result<(), DHCPError> {
        tracing::info!("Sending stuff back");
        let mut buff = [0; 4096];
        let len = client.packet.write_to_bytes(&mut buff);

        let mut response_addr = client.src.ip();
        if client.packet.is_broadcast() {
            response_addr = IpAddr::from(BROADCAST_ADDR);
        }
        println!("sent: {response_addr:?}");
        let sent_len = server
            .send_to(&buff[0..len], (response_addr, CLIENT_PORT))
            .await?;

        assert!(len == sent_len);
        Ok(())
    }
}

pub struct Client {
    pub packet: Packet,
    pub src: SocketAddr,
    sender: Sender<Client>,
}

impl Client {
    pub async fn send_back(mut self, packet: Packet) {
        self.packet = packet;
        self.sender.clone().send(self).await.expect("Channel closed");
    }
}
