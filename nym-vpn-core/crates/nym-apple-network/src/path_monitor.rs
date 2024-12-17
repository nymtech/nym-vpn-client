// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::ptr::NonNull;

use objc2::rc::Retained;

use super::{path::Path, sys};

/// An observer that you use to monitor and react to network changes.
#[derive(Debug)]
pub struct PathMonitor {
    inner: Retained<sys::OS_nw_path_monitor>,
}

impl Default for PathMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl PathMonitor {
    pub fn new() -> Self {
        Self {
            inner: unsafe { Retained::from_raw(sys::nw_path_monitor_create()) }
                .expect("failed to create nw_path_monitor"),
        }
    }

    pub fn start(&mut self) {
        unsafe { sys::nw_path_monitor_start(self.as_raw_mut()) };
    }

    pub fn cancel(&mut self) {
        unsafe { sys::nw_path_monitor_cancel(self.as_raw_mut()) };
    }

    /// Set dispatch queue on which updates will be delivered.
    pub fn set_dispatch_queue(&mut self, dispatch_queue: &dispatch2::Queue) {
        unsafe { sys::nw_path_monitor_set_queue(self.as_raw_mut(), dispatch_queue.as_raw()) };
    }

    /// Set path update handler.
    pub fn set_update_handler(&mut self, update_handler: impl Fn(Path) + 'static) {
        let block = block2::RcBlock::new(move |nw_path_ref| {
            let nw_path = Path::retain(NonNull::new(nw_path_ref).expect("invalid nw_path_ref"));

            update_handler(nw_path)
        });
        unsafe { sys::nw_path_monitor_set_update_handler(self.as_raw_mut(), &block) };
    }

    pub fn set_cancel_handler(&mut self, cancel_handler: impl Fn() + 'static) {
        let block = block2::RcBlock::new(cancel_handler);
        unsafe { sys::nw_path_monitor_set_cancel_handler(self.as_raw_mut(), &block) };
    }

    fn as_raw_mut(&self) -> sys::nw_path_monitor_t {
        Retained::as_ptr(&self.inner).cast_mut()
    }
}

impl Drop for PathMonitor {
    fn drop(&mut self) {
        self.cancel();
    }
}

#[cfg(test)]
mod tests {
    use crate::{Endpoint, PathMonitor};
    use dispatch2::{Queue, QueueAttribute};

    use std::sync::mpsc;

    #[test]
    fn test_create_path_monitor() {
        let queue = Queue::new("net.nymtech.test", QueueAttribute::Serial);
        let (tx, rx) = mpsc::channel();

        let mut path_monitor = PathMonitor::new();
        path_monitor.set_dispatch_queue(&queue);
        path_monitor.set_update_handler(move |nw_path| {
            let interfaces = nw_path.available_interfaces();
            let gateways = nw_path.gateways();

            println!("Path: {}", nw_path.description());
            println!("Status: {:?}", nw_path.status());

            for iface in interfaces {
                println!(
                    "Interface: name={} interface_type={:?} index={}",
                    iface.name().unwrap(),
                    iface.interface_type(),
                    iface.index()
                )
            }

            for gateway in gateways.iter() {
                if let Endpoint::Address(ep) = gateway {
                    println!("Gateway: {}", ep.address().unwrap());
                }
            }

            _ = tx.send(());
        });
        path_monitor.start();

        _ = rx.recv();
    }
}
