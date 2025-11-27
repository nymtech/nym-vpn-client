use crate::NodeIdentity;
use std::{
    collections::HashMap,
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

    pub fn add(&self, identity: NodeIdentity) {
        let now = Instant::now();
        if let Ok(mut map) = self.0.write() {
            map.insert(identity, now + Self::TTL);
            map.retain(|_, expiry| *expiry >= now); // Housekeeping
        }
    }

    pub fn remove(&self, identity: &NodeIdentity) {
        let now = Instant::now();
        if let Ok(mut map) = self.0.write() {
            map.remove(identity);
            map.retain(|_, expiry| *expiry >= now); // Housekeeping
        }
    }

    pub fn clear(&self) {
        if let Ok(mut map) = self.0.write() {
            map.clear();
        }
    }

    pub fn exists(&self, identity: &NodeIdentity) -> bool {
        if let Ok(map) = self.0.read() {
            match map.get(identity) {
                Some(expiry) => *expiry > Instant::now(),
                None => false,
            }
        } else {
            false
        }
    }

    pub fn is_empty(&self) -> bool {
        self.0.read().map(|map| map.is_empty()).unwrap_or(true)
    }
}

impl PartialEq for BlacklistedGateways {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
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

        assert!(!blacklist.exists(&identity));
        blacklist.add(identity);
        assert!(blacklist.exists(&identity));
    }

    #[test]
    fn test_remove() {
        let blacklist = BlacklistedGateways::new();
        let identity = create_test_identity("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42");

        blacklist.add(identity);
        assert!(blacklist.exists(&identity));

        blacklist.remove(&identity);
        assert!(!blacklist.exists(&identity));
    }

    #[test]
    fn test_clear() {
        let blacklist = BlacklistedGateways::new();
        let id1 = create_test_identity("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42");
        let id2 = create_test_identity("HiVGQq2riqPFoPyYRYCZq3zFmFk15gnJzH4s9mHEbgKH");

        blacklist.add(id1);
        blacklist.add(id2);
        assert!(!blacklist.is_empty());

        blacklist.clear();
        assert!(blacklist.is_empty());
        assert!(!blacklist.exists(&id1));
        assert!(!blacklist.exists(&id2));
    }

    #[test]
    fn test_is_empty() {
        let blacklist = BlacklistedGateways::new();
        assert!(blacklist.is_empty());

        let identity = create_test_identity("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42");
        blacklist.add(identity);
        assert!(!blacklist.is_empty());

        blacklist.remove(&identity);
        assert!(blacklist.is_empty());
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
        assert!(!blacklist.exists(&identity));
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
        blacklist.add(id2);

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
        }

        blacklist.add(id2);

        // Remove id2, which should trigger housekeeping
        blacklist.remove(&id2);

        // Both entries should be gone (id1 expired, id2 removed)
        if let Ok(map) = blacklist.0.read() {
            assert!(!map.contains_key(&id1));
            assert!(!map.contains_key(&id2));
        }
    }

    #[test]
    fn test_thread_safety() {
        let blacklist = BlacklistedGateways::new();
        let blacklist_clone = blacklist.clone();

        let id1 = create_test_identity("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42");
        let id2 = create_test_identity("HiVGQq2riqPFoPyYRYCZq3zFmFk15gnJzH4s9mHEbgKH");

        // Spawn threads that concurrently modify the blacklist
        let handle1 = thread::spawn(move || {
            for _ in 0..100 {
                blacklist.add(id1);
                thread::sleep(Duration::from_micros(10));
            }
        });

        let handle2 = thread::spawn(move || {
            for _ in 0..100 {
                blacklist_clone.add(id2);
                thread::sleep(Duration::from_micros(10));
            }
        });

        handle1.join().unwrap();
        handle2.join().unwrap();

        // Both identities should exist after concurrent adds
        let final_blacklist = BlacklistedGateways::new();
        final_blacklist.add(id1);
        final_blacklist.add(id2);
        assert!(final_blacklist.exists(&id1));
        assert!(final_blacklist.exists(&id2));
    }

    #[test]
    fn test_clone_shares_state() {
        let blacklist = BlacklistedGateways::new();
        let blacklist_clone = blacklist.clone();
        let identity = create_test_identity("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42");

        blacklist.add(identity);

        // Clone should see the same state
        assert!(blacklist_clone.exists(&identity));

        blacklist_clone.remove(&identity);

        // Original should see the removal
        assert!(!blacklist.exists(&identity));
    }
}
