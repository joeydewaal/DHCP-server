use std::time::Duration;


#[derive(Debug, Clone, Copy)]
pub enum LeaseTime {
    Infinite,
    Finite(Duration),
}

impl From<u32> for LeaseTime {
    fn from(value: u32) -> Self {
        match value {
            0xffffffff => LeaseTime::Infinite,
            secs => LeaseTime::Finite(Duration::from_secs(secs as u64)),
        }
    }
}

impl From<Duration> for LeaseTime {
    fn from(value: Duration) -> Self {
        Self::Finite(value)
    }
}

impl LeaseTime {
    pub fn to_bytes(&self) -> u32 {
        match self {
            LeaseTime::Infinite => 0xffffffff,
            LeaseTime::Finite(duration) => duration.as_secs() as u32,
        }
    }
}

