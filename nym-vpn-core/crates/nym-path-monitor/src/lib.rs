mod sys;

use std::{
    cell::RefCell,
    ffi::{c_int, CString},
    sync::Arc,
};

pub use sys::dispatch_queue_t;

pub struct NWPathMonitor {
    inner: sys::nw_path_monitor_t,
}

impl NWPathMonitor {
    pub fn new() -> Self {
        Self {
            inner: unsafe { sys::nw_path_monitor_create() },
        }
    }

    pub fn start(&mut self) {
        unsafe {
            sys::nw_path_monitor_start(self.inner);
        }
    }

    pub fn cancel(&mut self) {
        unsafe {
            sys::nw_path_monitor_cancel(self.inner);
        }
    }

    pub fn set_dispatch_queue(&mut self, dispatch_queue: sys::dispatch_queue_t) {
        unsafe {
            sys::nw_path_monitor_set_queue(self.inner, dispatch_queue);
        }
    }

    pub fn set_update_handler(&mut self, update_handler: impl Fn(NWPath) + 'static) {
        let block =
            block2::RcBlock::new(move |nw_path_ref| update_handler(NWPath::new(nw_path_ref)));
        unsafe {
            sys::nw_path_monitor_set_update_handler(self.inner, &block);
        }
    }
}

impl Drop for NWPathMonitor {
    fn drop(&mut self) {
        unsafe { sys::nw_release(self.inner as _) };
    }
}

pub struct NWPath {
    inner: sys::nw_path_t,
}

impl NWPath {
    fn new(nw_path_ref: sys::nw_path_t) -> Self {
        Self { inner: nw_path_ref }
    }

    pub fn get_status(&self) -> NWPathStatus {
        NWPathStatus::from(unsafe { sys::nw_path_get_status(self.inner) })
    }

    pub fn available_interfaces(&self) -> Vec<NWInterface> {
        let interfaces = Arc::new(RefCell::new(Vec::new()));
        let cloned_interfaces = interfaces.clone();
        let block = block2::StackBlock::new(move |nw_interface_ref| {
            let interface = NWInterface::new(nw_interface_ref);
            cloned_interfaces.borrow_mut().push(interface);
            objc2::runtime::Bool::YES
        });

        unsafe {
            sys::nw_path_enumerate_interfaces(self.inner, &block);
        }
        interfaces.take()
    }
}

impl Clone for NWPath {
    fn clone(&self) -> Self {
        unsafe { sys::nw_retain(self.inner as _) };
        Self { inner: self.inner }
    }
}

impl PartialEq for NWPath {
    fn eq(&self, other: &Self) -> bool {
        unsafe { sys::nw_path_is_equal(self.inner, other.inner) }
    }
}

impl Drop for NWPath {
    fn drop(&mut self) {
        unsafe { sys::nw_release(self.inner as _) };
    }
}

pub struct NWInterface {
    inner: sys::nw_interface_t,
}

impl NWInterface {
    fn new(nw_interface_t: sys::nw_interface_t) -> Self {
        Self {
            inner: nw_interface_t,
        }
    }

    pub fn name(&self) -> String {
        unsafe {
            let ptr = sys::nw_interface_get_name(self.inner);
            assert!(!ptr.is_null());
            CString::from_raw(ptr)
                .to_str()
                .expect("This must always be correct UTF-8")
                .to_owned()
        }
    }

    pub fn index(&self) -> u32 {
        unsafe { sys::nw_interface_get_index(self.inner) }
    }

    pub fn interface_type(&self) -> NWInterfaceType {
        let raw_interface_type = unsafe { sys::nw_interface_get_type(self.inner) };
        NWInterfaceType::from(raw_interface_type)
    }
}

impl Clone for NWInterface {
    fn clone(&self) -> Self {
        unsafe { sys::nw_retain(self.inner as _) };
        Self { inner: self.inner }
    }
}

impl Drop for NWInterface {
    fn drop(&mut self) {
        unsafe { sys::nw_release(self.inner as _) };
    }
}

#[derive(Debug, Copy, Clone)]
pub enum NWInterfaceType {
    Other,
    Wifi,
    Cellular,
    Wired,
    Loopback,
    Unknown(c_int),
}
impl From<sys::nw_interface_type_t> for NWInterfaceType {
    fn from(value: sys::nw_interface_type_t) -> Self {
        match value {
            sys::nw_interface_type_other => Self::Other,
            sys::nw_interface_type_wifi => Self::Wifi,
            sys::nw_interface_type_cellular => Self::Cellular,
            sys::nw_interface_type_wired => Self::Wired,
            sys::nw_interface_type_loopback => Self::Loopback,
            other => Self::Unknown(other),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum NWPathStatus {
    Invalid,
    Unsatisfied,
    Satisfied,
    Satisfiable,
    Unknown(c_int),
}

impl From<sys::nw_path_status_t> for NWPathStatus {
    fn from(value: sys::nw_path_status_t) -> Self {
        match value {
            sys::nw_path_status_invalid => Self::Invalid,
            sys::nw_path_status_satisfied => Self::Satisfied,
            sys::nw_path_status_unsatisfied => Self::Unsatisfied,
            sys::nw_path_status_satisfiable => Self::Satisfiable,
            other => Self::Unknown(other),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_create_path_monitor() {
        let mut path_monitor = NWPathMonitor::new();
        path_monitor.set_update_handler(|nw_path| {
            println!("Got nw_path with status: {:?}", nw_path.get_status());
        });
        path_monitor.start();
    }
}
