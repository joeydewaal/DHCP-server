use self::statemachine::DHCPStates;
use std::{
    borrow::BorrowMut,
    cell::{Cell, RefCell, RefMut},
    collections::HashMap,
    net::Ipv4Addr,
    rc::Rc,
};

mod statemachine;

#[derive(Debug, Clone)]
pub struct LeaseRange {
    pub start_lease: Ipv4Addr,
    pub end_lease: Ipv4Addr,
    pub data: HashMap<Ipv4Addr, DHCPStates>,
}

impl LeaseRange {
    pub fn new(start_lease: Ipv4Addr, end_lease: Ipv4Addr) -> Self {
        LeaseRange {
            start_lease,
            end_lease,
            data: HashMap::new(),
        }
    }

    pub fn get_available_ip(&mut self, xid: u32) -> Option<Ipv4Addr> {
        let mut data = &mut self.data;

        for opt_leasable_ip in self.start_lease..self.end_lease {
            // kijken of er een state is opgeslagen + kijken of deze al geoffered is
            if let Some(state) = data.get_mut(&opt_leasable_ip) {
                match state {
                    DHCPStates::Offered { clients } => {
                        // de xid van huidige client aan de reeds geofferde ip toevoegen
                        println!("IP is al geoffered, nog eens offeren");
                        clients.push(xid);
                        return Some(opt_leasable_ip);
                    }
                    _ => continue,
                }
            } else {
                data.insert(opt_leasable_ip, DHCPStates::new_offered(xid));
                println!("IP nog niet uitgeleased");
                return Some(opt_leasable_ip);
            }
        }
        None
    }
}
