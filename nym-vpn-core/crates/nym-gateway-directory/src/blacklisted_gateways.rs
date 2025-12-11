use crate::NodeIdentity;
use anyhow::{Result, anyhow};
use std::{
    collections::HashMap,
    hash::Hash,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

#[derive(Debug, Clone, Default)]
pub struct BlacklistedGateways(Arc<RwLock<HashMap<NodeIdentity, Instant>>>);

impl BlacklistedGateways {
    const TTL: Duration = Duration::from_secs(20 * 60); // 20 minutes

    pub fn new() -> Self {
        Default::default()
    }

    pub fn add(&self, identity: NodeIdentity) -> Result<()> {
        match self.0.write() {
            Ok(mut map) => {
                let now = Instant::now();
                map.insert(identity, now + Self::TTL);
                map.retain(|_, expiry| *expiry >= now); // Housekeeping
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
                map.retain(|_, expiry| *expiry >= now); // Housekeeping
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
        match self.0.read() {
            Ok(map) => match map.get(identity) {
                Some(expiry) => Ok(*expiry > Instant::now()),
                None => Ok(false),
            },
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
        blacklist.add(identity).unwrap();
        assert!(blacklist.exists(&identity).unwrap());
    }

    #[test]
    fn test_remove() {
        let blacklist = BlacklistedGateways::new();
        let identity = create_test_identity("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42");

        blacklist.add(identity).unwrap();
        assert!(blacklist.exists(&identity).unwrap());

        blacklist.remove(&identity).unwrap();
        assert!(!blacklist.exists(&identity).unwrap());
    }

    #[test]
    fn test_clear() {
        let blacklist = BlacklistedGateways::new();
        let id1 = create_test_identity("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42");
        let id2 = create_test_identity("HiVGQq2riqPFoPyYRYCZq3zFmFk15gnJzH4s9mHEbgKH");

        blacklist.add(id1).unwrap();
        blacklist.add(id2).unwrap();
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
        blacklist.add(identity).unwrap();
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
            map.insert(identity, Instant::now() - Duration::from_secs(1));
        }

        // Should return false because the entry is expired
        assert!(!blacklist.exists(&identity).unwrap());
    }

    #[test]
    fn test_housekeeping_on_add() {
        let blacklist = BlacklistedGateways::new();
        let id1 = create_test_identity("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42");
        let id2 = create_test_identity("HiVGQq2riqPFoPyYRYCZq3zFmFk15gnJzH4s9mHEbgKH");

        // Add an expired entry manually
        if let Ok(mut map) = blacklist.0.write() {
            map.insert(id1, Instant::now() - Duration::from_secs(1));
        }

        // Add a new entry, which should trigger housekeeping
        blacklist.add(id2).unwrap();

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
            map.insert(id1, Instant::now() - Duration::from_secs(1));
        } else {
            panic!("Failed to acquire write lock");
        }

        blacklist.add(id2).unwrap();

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
                blacklist_for_thread1.add(id1).unwrap();
                thread::sleep(Duration::from_micros(10));
            }
        });

        let handle2 = thread::spawn(move || {
            for _ in 0..100 {
                blacklist_for_thread2.add(id2).unwrap();
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

        blacklist.add(identity).unwrap();

        // Clone should see the same state
        assert!(blacklist_clone.exists(&identity).unwrap());

        blacklist_clone.remove(&identity).unwrap();

        // Original should see the removal
        assert!(!blacklist.exists(&identity).unwrap());
    }
}
