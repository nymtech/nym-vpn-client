use crate::NodeIdentity;
use anyhow::{Result, anyhow};
use std::{
    collections::HashMap,
    fmt,
    hash::Hash,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

/// Why a gateway was added to the blacklist, kept around so the selector can log a useful
/// message when it later excludes that gateway, instead of just "it's blacklisted".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlacklistReason {
    /// The WG handshake timed out, or a post-handshake connectivity probe failed, without a
    /// healthy metadata path ever being established through this gateway acting as exit.
    ConnectionFailed,
    /// This entry gateway was blamed after several consecutive pre-handshake connection
    /// failures while the exit gateway kept changing.
    EntryBlamedForRepeatedFailures,
    /// Registration with this gateway failed.
    RegistrationFailed,
    /// The WG handshake with this entry gateway never completed: the entry hop is dead from
    /// the current network (e.g. its WG port is blackholed), regardless of the exit gateway.
    EntryHandshakeFailed,
}

impl fmt::Display for BlacklistReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConnectionFailed => write!(f, "connection failed"),
            Self::EntryBlamedForRepeatedFailures => {
                write!(f, "blamed for repeated pre-handshake failures")
            }
            Self::RegistrationFailed => write!(f, "registration failed"),
            Self::EntryHandshakeFailed => write!(f, "entry WireGuard handshake failed"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Entry {
    expiry: Instant,
    reason: BlacklistReason,
}

#[derive(Debug, Clone, Default)]
pub struct BlacklistedGateways(Arc<RwLock<HashMap<NodeIdentity, Entry>>>);

impl BlacklistedGateways {
    const TTL: Duration = Duration::from_mins(20);

    pub fn new() -> Self {
        Default::default()
    }

    pub fn add(&self, identity: NodeIdentity, reason: BlacklistReason) -> Result<()> {
        match self.0.write() {
            Ok(mut map) => {
                let now = Instant::now();
                map.insert(
                    identity,
                    Entry {
                        expiry: now + Self::TTL,
                        reason,
                    },
                );
                map.retain(|_, entry| entry.expiry >= now); // Housekeeping
                Ok(())
            }
            Err(e) => Err(anyhow!("Failed to acquire write lock: {e}")),
        }
    }

    pub fn remove(&self, identity: &NodeIdentity) -> Result<()> {
        match self.0.write() {
            Ok(mut map) => {
                let now = Instant::now();
                map.remove(identity);
                map.retain(|_, entry| entry.expiry >= now); // Housekeeping
                Ok(())
            }
            Err(e) => Err(anyhow!("Failed to acquire write lock: {e}")),
        }
    }

    pub fn clear(&self) -> Result<()> {
        match self.0.write() {
            Ok(mut map) => {
                map.clear();
                Ok(())
            }
            Err(e) => Err(anyhow!("Failed to acquire write lock: {e}")),
        }
    }

    pub fn exists(&self, identity: &NodeIdentity) -> Result<bool> {
        Ok(self.reason(identity)?.is_some())
    }

    /// Returns why `identity` is currently blacklisted, or `None` if it isn't (either never
    /// added, or its entry has expired).
    pub fn reason(&self, identity: &NodeIdentity) -> Result<Option<BlacklistReason>> {
        match self.0.read() {
            Ok(map) => Ok(map
                .get(identity)
                .filter(|entry| entry.expiry > Instant::now())
                .map(|entry| entry.reason)),
            Err(e) => Err(anyhow!("Failed to acquire read lock: {e}")),
        }
    }

    pub fn is_empty(&self) -> Result<bool> {
        match self.0.read() {
            Ok(map) => Ok(map.is_empty()),
            Err(e) => Err(anyhow!("Failed to acquire read lock: {e}")),
        }
    }
}

impl PartialEq for BlacklistedGateways {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for BlacklistedGateways {}

impl Hash for BlacklistedGateways {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    fn create_test_identity(s: &str) -> NodeIdentity {
        NodeIdentity::from_base58_string(s).unwrap()
    }

    #[test]
    fn test_add_and_exists() {
        let blacklist = BlacklistedGateways::new();
        let identity = create_test_identity("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42");

        assert!(!blacklist.exists(&identity).unwrap());
        blacklist
            .add(identity, BlacklistReason::ConnectionFailed)
            .unwrap();
        assert!(blacklist.exists(&identity).unwrap());
    }

    #[test]
    fn test_reason_is_recorded_and_retrievable() {
        let blacklist = BlacklistedGateways::new();
        let identity = create_test_identity("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42");

        assert_eq!(blacklist.reason(&identity).unwrap(), None);
        blacklist
            .add(identity, BlacklistReason::RegistrationFailed)
            .unwrap();
        assert_eq!(
            blacklist.reason(&identity).unwrap(),
            Some(BlacklistReason::RegistrationFailed)
        );
    }

    #[test]
    fn test_remove() {
        let blacklist = BlacklistedGateways::new();
        let identity = create_test_identity("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42");

        blacklist
            .add(identity, BlacklistReason::ConnectionFailed)
            .unwrap();
        assert!(blacklist.exists(&identity).unwrap());

        blacklist.remove(&identity).unwrap();
        assert!(!blacklist.exists(&identity).unwrap());
    }

    #[test]
    fn test_clear() {
        let blacklist = BlacklistedGateways::new();
        let id1 = create_test_identity("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42");
        let id2 = create_test_identity("HiVGQq2riqPFoPyYRYCZq3zFmFk15gnJzH4s9mHEbgKH");

        blacklist
            .add(id1, BlacklistReason::ConnectionFailed)
            .unwrap();
        blacklist
            .add(id2, BlacklistReason::ConnectionFailed)
            .unwrap();
        assert!(!blacklist.is_empty().unwrap());

        blacklist.clear().unwrap();
        assert!(blacklist.is_empty().unwrap());
        assert!(!blacklist.exists(&id1).unwrap());
        assert!(!blacklist.exists(&id2).unwrap());
    }

    #[test]
    fn test_is_empty() {
        let blacklist = BlacklistedGateways::new();
        assert!(blacklist.is_empty().unwrap());

        let identity = create_test_identity("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42");
        blacklist
            .add(identity, BlacklistReason::ConnectionFailed)
            .unwrap();
        assert!(!blacklist.is_empty().unwrap());

        blacklist.remove(&identity).unwrap();
        assert!(blacklist.is_empty().unwrap());
    }

    #[test]
    fn test_ttl_expiration() {
        // This test verifies the TTL logic by manually manipulating the expiry time
        let blacklist = BlacklistedGateways::new();
        let identity = create_test_identity("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42");

        // Add an entry with an expired timestamp
        if let Ok(mut map) = blacklist.0.write() {
            map.insert(
                identity,
                Entry {
                    expiry: Instant::now() - Duration::from_secs(1),
                    reason: BlacklistReason::ConnectionFailed,
                },
            );
        }

        // Should return false because the entry is expired
        assert!(!blacklist.exists(&identity).unwrap());
        assert_eq!(blacklist.reason(&identity).unwrap(), None);
    }

    #[test]
    fn test_housekeeping_on_add() {
        let blacklist = BlacklistedGateways::new();
        let id1 = create_test_identity("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42");
        let id2 = create_test_identity("HiVGQq2riqPFoPyYRYCZq3zFmFk15gnJzH4s9mHEbgKH");

        // Add an expired entry manually
        if let Ok(mut map) = blacklist.0.write() {
            map.insert(
                id1,
                Entry {
                    expiry: Instant::now() - Duration::from_secs(1),
                    reason: BlacklistReason::ConnectionFailed,
                },
            );
        }

        // Add a new entry, which should trigger housekeeping
        blacklist
            .add(id2, BlacklistReason::ConnectionFailed)
            .unwrap();

        // The expired entry should have been cleaned up
        if let Ok(map) = blacklist.0.read() {
            assert!(!map.contains_key(&id1));
            assert!(map.contains_key(&id2));
        }
    }

    #[test]
    fn test_housekeeping_on_remove() {
        let blacklist = BlacklistedGateways::new();
        let id1 = create_test_identity("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42");
        let id2 = create_test_identity("HiVGQq2riqPFoPyYRYCZq3zFmFk15gnJzH4s9mHEbgKH");

        // Add an expired entry manually
        if let Ok(mut map) = blacklist.0.write() {
            map.insert(
                id1,
                Entry {
                    expiry: Instant::now() - Duration::from_secs(1),
                    reason: BlacklistReason::ConnectionFailed,
                },
            );
        } else {
            panic!("Failed to acquire write lock");
        }

        blacklist
            .add(id2, BlacklistReason::ConnectionFailed)
            .unwrap();

        // Remove id2, which should trigger housekeeping
        blacklist.remove(&id2).unwrap();

        // Both entries should be gone (id1 expired, id2 removed)
        if let Ok(map) = blacklist.0.read() {
            assert!(!map.contains_key(&id1));
            assert!(!map.contains_key(&id2));
        } else {
            panic!("Failed to acquire read lock");
        }
    }

    #[test]
    fn test_thread_safety() {
        let blacklist = BlacklistedGateways::new();
        let blacklist_for_thread1 = blacklist.clone();
        let blacklist_for_thread2 = blacklist.clone();

        let id1 = create_test_identity("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42");
        let id2 = create_test_identity("HiVGQq2riqPFoPyYRYCZq3zFmFk15gnJzH4s9mHEbgKH");

        let handle1 = thread::spawn(move || {
            for _ in 0..100 {
                blacklist_for_thread1
                    .add(id1, BlacklistReason::ConnectionFailed)
                    .unwrap();
                thread::sleep(Duration::from_micros(10));
            }
        });

        let handle2 = thread::spawn(move || {
            for _ in 0..100 {
                blacklist_for_thread2
                    .add(id2, BlacklistReason::ConnectionFailed)
                    .unwrap();
                thread::sleep(Duration::from_micros(10));
            }
        });

        handle1.join().unwrap();
        handle2.join().unwrap();

        assert!(
            blacklist.exists(&id1).unwrap(),
            "id1 should exist in shared blacklist"
        );
        assert!(
            blacklist.exists(&id2).unwrap(),
            "id2 should exist in shared blacklist"
        );
    }

    #[test]
    fn test_clone_shares_state() {
        let blacklist = BlacklistedGateways::new();
        let blacklist_clone = blacklist.clone();
        let identity = create_test_identity("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42");

        blacklist
            .add(identity, BlacklistReason::ConnectionFailed)
            .unwrap();

        // Clone should see the same state
        assert!(blacklist_clone.exists(&identity).unwrap());

        blacklist_clone.remove(&identity).unwrap();

        // Original should see the removal
        assert!(!blacklist.exists(&identity).unwrap());
    }

    #[test]
    fn entry_handshake_failed_reason_is_human_readable() {
        assert_eq!(
            BlacklistReason::EntryHandshakeFailed.to_string(),
            "entry WireGuard handshake failed"
        );
    }
}
