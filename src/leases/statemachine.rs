use crate::packet::{DHCPOption, LeaseTime};
use chrono::{DateTime, Utc};



#[derive(Debug, Clone)]
pub enum DHCPStates {
    /// er is een dhcp offer request gestuurd met dit ip,
    Offered {
        /// in de clients zitten alle ip-s van de clients waar het id naar
        /// toe is gestuurd
        clients: Vec<u32>,
    },
    /// Dit ip is in gebruik
    Used {
        /// IP van
        client_id: (),
        lease_time: LeaseTime,
        start_time: DateTime<Utc>,
        options: Vec<DHCPOption>,
    },
}

impl DHCPStates {
    pub fn new_offered(xid: u32) -> Self {
        DHCPStates::Offered { clients: vec![xid] }
    }
}
