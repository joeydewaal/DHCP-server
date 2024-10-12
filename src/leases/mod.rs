use self::statemachine::DHCPStates;
use crate::{
    leases::statemachine::DHCPOffer,
    packet::{DHCPOption, LeaseTime, Packet},
};
use chrono::Utc;
use std::{collections::HashMap, net::Ipv4Addr, time::Duration};

pub const DEFAULT_LEASE_TIME: LeaseTime = LeaseTime::Finite(Duration::from_secs(86600));

mod statemachine;

#[derive(Debug, Clone)]
pub struct LeaseRange {
    pub start_lease: Ipv4Addr,
    pub end_lease: Ipv4Addr,
    pub server_addr: Ipv4Addr,
    pub subnet: Ipv4Addr,
    pub data: HashMap<Ipv4Addr, DHCPStates>,
}

impl LeaseRange {
    pub fn new(
        start_lease: Ipv4Addr,
        end_lease: Ipv4Addr,
        server_addr: Ipv4Addr,
        subnet: Ipv4Addr,
    ) -> Self {
        LeaseRange {
            start_lease,
            end_lease,
            subnet,
            server_addr,
            data: HashMap::new(),
        }
    }

    pub fn get_subnet_option(&self) -> DHCPOption {
        DHCPOption::Subnet(self.subnet)
    }

    pub fn get_leasetime_option(&self) -> DHCPOption {
        DHCPOption::IpLeasetime(LeaseTime::Infinite)
    }

    pub fn get_server_id_option(&self) -> DHCPOption {
        DHCPOption::ServerIdentifier(self.server_addr)
    }

    /// Zoekt naar een beschikbaar ip in de range.
    /// Geeft `None` terug als er geen beschikbaar is
    pub fn get_available_ip(&mut self, xid: u32) -> Option<Ipv4Addr> {
        for opt_leasable_ip in self.start_lease..self.end_lease {
            // kijken of er een state is opgeslagen + kijken of deze al geoffered is
            if let Some(state) = self.data.get_mut(&opt_leasable_ip) {
                match state {
                    DHCPStates::Offered(clients) => {
                        // de xid van huidige client aan de reeds geofferde ip toevoegen
                        println!("IP is al geoffered, nog eens offeren");
                        clients.insert(DHCPOffer {
                            xid,
                            lease_time: DEFAULT_LEASE_TIME,
                        });
                        return Some(opt_leasable_ip);
                    }
                    _ => continue,
                }
            } else {
                self.data
                    .insert(opt_leasable_ip, DHCPStates::new_offered(xid));
                println!("IP nog niet uitgeleased");
                return Some(opt_leasable_ip);
            }
        }
        None
    }

    pub fn reserve_ip(&mut self, packet: &Packet, ip: Ipv4Addr) -> Result<(), LeaseReserveError> {
        let Some(state) = self.data.get_mut(&ip) else {
            return Err(LeaseReserveError::NotRequested);
        };
        let lease_time: LeaseTime = match state {
            DHCPStates::Offered(clients) => {
                if let Some(offer) = clients.get(&packet.xid.into()) {
                    offer.lease_time
                } else {
                    return Err(LeaseReserveError::NotRequested);
                }
            }
            _ => return Err(LeaseReserveError::InUse),
        };
        *state = DHCPStates::Used {
            client_id: (),
            lease_time,
            start_time: Utc::now(),
            options: packet.options_cloned(),
        };
        Ok(())
    }
}

#[derive(Debug)]
pub enum LeaseReserveError {
    NotRequested,
    InUse,
}
