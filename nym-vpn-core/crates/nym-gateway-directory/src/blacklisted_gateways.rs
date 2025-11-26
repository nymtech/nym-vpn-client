use crate::NodeIdentity;
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BlacklistedGateways(HashMap<NodeIdentity, Instant>);

impl BlacklistedGateways {
    const TTL: Duration = Duration::from_mins(20);

    pub fn new() -> Self {
        Default::default()
    }

    pub fn add(&mut self, identity: NodeIdentity) {
        let now = Instant::now();
        self.0.insert(identity, now + Self::TTL);
        self.0.retain(|_, expiry| *expiry >= now); // Housekeeping
    }

    pub fn remove(&mut self, identity: &NodeIdentity) {
        let now = Instant::now();
        self.0.remove(identity);
        self.0.retain(|_, expiry| *expiry >= now); // Housekeeping
    }
    
    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn exists(&self, identity: &NodeIdentity) -> bool {
        match self.0.get(identity) {
            Some(expiry) => *expiry > Instant::now(),
            None => false,
        }
    }
    
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
