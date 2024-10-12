use std::sync::{Arc, Mutex, MutexGuard};

use crate::leases::LeaseRange;

#[derive(Debug, Clone)]
pub struct DHCPState {
    inner: Arc<DHCPStateInner>,
}

#[derive(Debug)]
pub struct DHCPStateInner {
    lease_range: Mutex<LeaseRange>,
}

impl DHCPState {
    pub fn from_lease(lease_range: LeaseRange) -> Self {
        DHCPState {
            inner: Arc::new(DHCPStateInner { lease_range: lease_range.into() }),
        }
    }

    pub fn lock<'a>(&'a self) -> MutexGuard<'a, LeaseRange> {
        self.inner.lease_range.lock().unwrap()
    }
}
