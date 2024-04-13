use std::{collections::HashSet, hash::Hash};

use crate::packet::{DHCPOption, LeaseTime};
use chrono::{DateTime, Utc};

use super::DEFAULT_LEASE_TIME;



#[derive(Debug, Clone)]
pub enum DHCPStates {
    /// er is een dhcp offer request gestuurd met dit ip,
    Offered (
        /// in de clients zitten alle ip-s van de clients waar het id naar
        /// toe is gestuurd
        HashSet<DHCPOffer>
    ),
    /// Dit ip is in gebruik
    Used {
        /// IP van
        client_id: (),
        lease_time: LeaseTime,
        start_time: DateTime<Utc>,
        options: HashSet<DHCPOption>,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct DHCPOffer {
    pub xid: u32,
    pub lease_time: LeaseTime
}

impl PartialEq for DHCPOffer {
    fn eq(&self, other: &Self) -> bool {
        self.xid == other.xid
    }
}

impl Eq for DHCPOffer {}

impl Hash for DHCPOffer {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u32(self.xid);
    }
}

impl DHCPStates {
    pub fn new_offered(xid: u32) -> Self {
        let mut clients = HashSet::new();
        clients.insert(DHCPOffer {
            xid,
            lease_time: DEFAULT_LEASE_TIME
        });
        DHCPStates::Offered(clients)
    }
}

impl From<u32> for DHCPOffer {
    fn from(xid: u32) -> Self {
        Self {
            xid,
            lease_time: DEFAULT_LEASE_TIME
        }
    }
}
