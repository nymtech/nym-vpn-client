use crate::NodeIdentity;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt,
    hash::Hash,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

/// Why a gateway was added to the blacklist, kept around so the selector can log a useful
/// message when it later excludes that gateway, instead of just "it's blacklisted".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

impl BlacklistReason {
    /// How long a gateway stays excluded for this reason.
    const TRANSIENT_TTL: Duration = Duration::from_mins(20);
    const PERSISTENT_TTL: Duration = Duration::from_hours(24);

    /// Whether the exclusion is worth keeping across restarts. Entry-side verdicts describe
    /// the path between this network and the gateway, which does not heal by restarting the
    /// app, so they are persisted; exit/registration failures are transient and stay in memory.
    pub fn is_persistent(&self) -> bool {
        match self {
            Self::EntryHandshakeFailed | Self::EntryBlamedForRepeatedFailures => true,
            Self::ConnectionFailed | Self::RegistrationFailed => false,
        }
    }

    pub fn ttl(&self) -> Duration {
        if self.is_persistent() {
            Self::PERSISTENT_TTL
        } else {
            Self::TRANSIENT_TTL
        }
    }
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
    expires_at: SystemTime,
    reason: BlacklistReason,
}

impl Entry {
    fn is_live(&self, now: SystemTime) -> bool {
        self.expires_at > now
    }
}

/// On-disk form of a persisted blacklist entry.
#[derive(Debug, Serialize, Deserialize)]
struct PersistedEntry {
    expires_at_unix_secs: u64,
    reason: BlacklistReason,
}

#[derive(Debug, Default)]
struct Inner {
    map: HashMap<NodeIdentity, Entry>,
    /// When set, persistent entries are mirrored to this file after every change.
    path: Option<PathBuf>,
}

impl Inner {
    fn prune(&mut self, now: SystemTime) {
        self.map.retain(|_, entry| entry.is_live(now));
    }

    /// Mirror the live persistent entries to disk. Failures are logged, never propagated:
    /// a gateway must still get blacklisted in memory when the disk is unavailable.
    fn persist(&self, now: SystemTime) {
        let Some(path) = self.path.as_ref() else {
            return;
        };
        let persisted: HashMap<String, PersistedEntry> = self
            .map
            .iter()
            .filter(|(_, entry)| entry.reason.is_persistent() && entry.is_live(now))
            .map(|(identity, entry)| {
                let expires_at_unix_secs = entry
                    .expires_at
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                (
                    identity.to_base58_string(),
                    PersistedEntry {
                        expires_at_unix_secs,
                        reason: entry.reason,
                    },
                )
            })
            .collect();
        if let Err(err) = write_atomically(path, &persisted) {
            tracing::warn!(
                "Failed to persist blacklisted gateways to {}: {err}",
                path.display()
            );
        }
    }
}

fn write_atomically(path: &Path, persisted: &HashMap<String, PersistedEntry>) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp_path = path.with_extension("json.tmp");
    let json = serde_json::to_vec_pretty(persisted)?;
    std::fs::write(&tmp_path, json)?;
    std::fs::rename(&tmp_path, path)?;
    Ok(())
}

fn load_persisted(path: &Path, now: SystemTime) -> HashMap<NodeIdentity, Entry> {
    let contents = match std::fs::read(path) {
        Ok(contents) => contents,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return HashMap::new(),
        Err(err) => {
            tracing::warn!(
                "Failed to read blacklisted gateways from {}: {err}",
                path.display()
            );
            return HashMap::new();
        }
    };
    let persisted: HashMap<String, PersistedEntry> = match serde_json::from_slice(&contents) {
        Ok(persisted) => persisted,
        Err(err) => {
            tracing::warn!(
                "Ignoring corrupt blacklisted gateways file {}: {err}",
                path.display()
            );
            return HashMap::new();
        }
    };
    persisted
        .into_iter()
        .filter_map(|(identity, entry)| {
            let identity = NodeIdentity::from_base58_string(&identity).ok()?;
            let expires_at = UNIX_EPOCH + Duration::from_secs(entry.expires_at_unix_secs);
            let entry = Entry {
                expires_at,
                reason: entry.reason,
            };
            entry.is_live(now).then_some((identity, entry))
        })
        .collect()
}

#[derive(Debug, Clone, Default)]
pub struct BlacklistedGateways(Arc<RwLock<Inner>>);

impl BlacklistedGateways {
    /// In-memory only blacklist; nothing survives a restart.
    pub fn new() -> Self {
        Default::default()
    }

    /// Blacklist backed by `path` (when given): persistent entries recorded there by a previous
    /// run are loaded, minus the expired ones, and every later change is mirrored back. A
    /// missing or unreadable file yields an empty blacklist.
    pub fn load_or_new(path: Option<PathBuf>) -> Self {
        let Some(path) = path else {
            return Self::new();
        };
        let map = load_persisted(&path, SystemTime::now());
        if !map.is_empty() {
            tracing::info!(
                "Loaded {} persisted blacklisted gateway(s) from {}",
                map.len(),
                path.display()
            );
        }
        Self(Arc::new(RwLock::new(Inner {
            map,
            path: Some(path),
        })))
    }

    pub fn add(&self, identity: NodeIdentity, reason: BlacklistReason) -> Result<()> {
        match self.0.write() {
            Ok(mut inner) => {
                let now = SystemTime::now();
                inner.map.insert(
                    identity,
                    Entry {
                        expires_at: now + reason.ttl(),
                        reason,
                    },
                );
                inner.prune(now); // Housekeeping
                inner.persist(now);
                Ok(())
            }
            Err(e) => Err(anyhow!("Failed to acquire write lock: {e}")),
        }
    }

    pub fn remove(&self, identity: &NodeIdentity) -> Result<()> {
        match self.0.write() {
            Ok(mut inner) => {
                let now = SystemTime::now();
                inner.map.remove(identity);
                inner.prune(now); // Housekeeping
                inner.persist(now);
                Ok(())
            }
            Err(e) => Err(anyhow!("Failed to acquire write lock: {e}")),
        }
    }

    pub fn clear(&self) -> Result<()> {
        match self.0.write() {
            Ok(mut inner) => {
                inner.map.clear();
                inner.persist(SystemTime::now());
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
            Ok(inner) => {
                let now = SystemTime::now();
                Ok(inner
                    .map
                    .get(identity)
                    .filter(|entry| entry.is_live(now))
                    .map(|entry| entry.reason))
            }
            Err(e) => Err(anyhow!("Failed to acquire read lock: {e}")),
        }
    }

    pub fn is_empty(&self) -> Result<bool> {
        match self.0.read() {
            Ok(inner) => Ok(inner.map.is_empty()),
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
        if let Ok(mut inner) = blacklist.0.write() {
            inner.map.insert(
                identity,
                Entry {
                    expires_at: SystemTime::now() - Duration::from_secs(1),
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
        if let Ok(mut inner) = blacklist.0.write() {
            inner.map.insert(
                id1,
                Entry {
                    expires_at: SystemTime::now() - Duration::from_secs(1),
                    reason: BlacklistReason::ConnectionFailed,
                },
            );
        }

        // Add a new entry, which should trigger housekeeping
        blacklist
            .add(id2, BlacklistReason::ConnectionFailed)
            .unwrap();

        // The expired entry should have been cleaned up
        if let Ok(inner) = blacklist.0.read() {
            assert!(!inner.map.contains_key(&id1));
            assert!(inner.map.contains_key(&id2));
        }
    }

    #[test]
    fn test_housekeeping_on_remove() {
        let blacklist = BlacklistedGateways::new();
        let id1 = create_test_identity("7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42");
        let id2 = create_test_identity("HiVGQq2riqPFoPyYRYCZq3zFmFk15gnJzH4s9mHEbgKH");

        // Add an expired entry manually
        if let Ok(mut inner) = blacklist.0.write() {
            inner.map.insert(
                id1,
                Entry {
                    expires_at: SystemTime::now() - Duration::from_secs(1),
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
        if let Ok(inner) = blacklist.0.read() {
            assert!(!inner.map.contains_key(&id1));
            assert!(!inner.map.contains_key(&id2));
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

    const ID_A: &str = "24h2yanCFU5iy7xNQmW6RowFa6EzmAYQdM1bs8Y1X6iH";
    const ID_B: &str = "26ZmTxTVBKHZg8MTKwypHkXZVJhDC7QHuv3BdsyRyTuk";
    const ID_C: &str = "27GwHdmXLULVieyXmxZ6v9DHzRJtTEjfode1dzbptEAK";
    const ID_D: &str = "28tXg9mEW4mifgU1TdetVVAN5PvmhtLpHzFRMfJBT6ND";

    #[test]
    fn entry_side_reasons_are_persisted_across_reload_but_transient_ones_are_not() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("blacklist.json");

        let blacklist = BlacklistedGateways::load_or_new(Some(path.clone()));
        blacklist
            .add(
                create_test_identity(ID_A),
                BlacklistReason::EntryHandshakeFailed,
            )
            .unwrap();
        blacklist
            .add(
                create_test_identity(ID_B),
                BlacklistReason::EntryBlamedForRepeatedFailures,
            )
            .unwrap();
        blacklist
            .add(
                create_test_identity(ID_C),
                BlacklistReason::ConnectionFailed,
            )
            .unwrap();
        blacklist
            .add(
                create_test_identity(ID_D),
                BlacklistReason::RegistrationFailed,
            )
            .unwrap();

        let reloaded = BlacklistedGateways::load_or_new(Some(path));
        assert_eq!(
            reloaded.reason(&create_test_identity(ID_A)).unwrap(),
            Some(BlacklistReason::EntryHandshakeFailed)
        );
        assert_eq!(
            reloaded.reason(&create_test_identity(ID_B)).unwrap(),
            Some(BlacklistReason::EntryBlamedForRepeatedFailures)
        );
        assert!(
            !reloaded.exists(&create_test_identity(ID_C)).unwrap(),
            "an exit connection failure is transient and must not survive a restart"
        );
        assert!(!reloaded.exists(&create_test_identity(ID_D)).unwrap());
    }

    #[test]
    fn persistent_reasons_get_a_long_ttl_and_transient_ones_keep_the_short_one() {
        for reason in [
            BlacklistReason::EntryHandshakeFailed,
            BlacklistReason::EntryBlamedForRepeatedFailures,
        ] {
            assert!(reason.is_persistent(), "{reason} should be persisted");
            assert!(
                reason.ttl() >= Duration::from_hours(12),
                "{reason} ttl too short"
            );
        }
        for reason in [
            BlacklistReason::ConnectionFailed,
            BlacklistReason::RegistrationFailed,
        ] {
            assert!(!reason.is_persistent(), "{reason} should stay in memory");
            assert_eq!(reason.ttl(), Duration::from_mins(20));
        }
    }

    #[test]
    fn remove_and_clear_are_reflected_in_the_persisted_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("blacklist.json");
        let id_a = create_test_identity(ID_A);
        let id_b = create_test_identity(ID_B);

        let blacklist = BlacklistedGateways::load_or_new(Some(path.clone()));
        blacklist
            .add(id_a, BlacklistReason::EntryHandshakeFailed)
            .unwrap();
        blacklist
            .add(id_b, BlacklistReason::EntryHandshakeFailed)
            .unwrap();
        blacklist.remove(&id_a).unwrap();

        let reloaded = BlacklistedGateways::load_or_new(Some(path.clone()));
        assert!(!reloaded.exists(&id_a).unwrap());
        assert!(reloaded.exists(&id_b).unwrap());

        reloaded.clear().unwrap();
        let reloaded = BlacklistedGateways::load_or_new(Some(path));
        assert!(reloaded.is_empty().unwrap());
    }

    #[test]
    fn expired_persisted_entries_are_dropped_on_load() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("blacklist.json");
        std::fs::write(
            &path,
            format!(
                r#"{{"{ID_A}":{{"expires_at_unix_secs":1,"reason":"EntryHandshakeFailed"}},
                   "{ID_B}":{{"expires_at_unix_secs":4102444800,"reason":"EntryHandshakeFailed"}}}}"#
            ),
        )
        .unwrap();

        let blacklist = BlacklistedGateways::load_or_new(Some(path));
        assert!(!blacklist.exists(&create_test_identity(ID_A)).unwrap());
        assert!(blacklist.exists(&create_test_identity(ID_B)).unwrap());
    }

    #[test]
    fn corrupt_or_missing_file_falls_back_to_an_empty_blacklist() {
        let dir = tempfile::tempdir().unwrap();
        let missing = BlacklistedGateways::load_or_new(Some(dir.path().join("nope.json")));
        assert!(missing.is_empty().unwrap());

        let corrupt_path = dir.path().join("corrupt.json");
        std::fs::write(&corrupt_path, "not json at all").unwrap();
        let corrupt = BlacklistedGateways::load_or_new(Some(corrupt_path.clone()));
        assert!(corrupt.is_empty().unwrap());

        // and it recovers: the next persistent add rewrites the file
        corrupt
            .add(
                create_test_identity(ID_A),
                BlacklistReason::EntryHandshakeFailed,
            )
            .unwrap();
        let reloaded = BlacklistedGateways::load_or_new(Some(corrupt_path));
        assert!(reloaded.exists(&create_test_identity(ID_A)).unwrap());
    }
}
