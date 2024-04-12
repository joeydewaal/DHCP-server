use std::{collections::HashMap, net::Ipv4Addr};


pub struct LeaseRange {
    pub begin: Ipv4Addr,
    pub end: Ipv4Addr,
    pub data: HashMap<Ipv4Addr, DHCPStates>
}


pub enum DHCPStates {
    Offered {
        clients: Vec<()>,
    },
    Used {
        client_ip: Ipv4Addr,
    }
}
