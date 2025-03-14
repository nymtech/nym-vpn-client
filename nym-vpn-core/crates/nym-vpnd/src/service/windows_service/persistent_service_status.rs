use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use windows_service::{
    service::{ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus, ServiceType},
    service_control_handler::ServiceStatusHandle,
};

/// Service status helper with persistent checkpoint counter.
#[derive(Debug, Clone)]
pub struct PersistentServiceStatus {
    service_type: ServiceType,
    status_handle: ServiceStatusHandle,
    checkpoint_counter: Arc<AtomicUsize>,
}

pub type Result<T> = windows_service::Result<T>;

impl PersistentServiceStatus {
    pub fn new(service_type: ServiceType, status_handle: ServiceStatusHandle) -> Self {
        PersistentServiceStatus {
            service_type,
            status_handle,
            checkpoint_counter: Arc::new(AtomicUsize::new(1)),
        }
    }

    /// Tell the system that the service is pending start and provide the time estimate until
    /// initialization is complete.
    pub fn set_pending_start(&mut self, wait_hint: Duration) -> Result<()> {
        self.report_status(
            ServiceState::StartPending,
            wait_hint,
            ServiceExitCode::default(),
        )
    }

    /// Tell the system that the service is running.
    pub fn set_running(&mut self) -> Result<()> {
        self.report_status(
            ServiceState::Running,
            Duration::default(),
            ServiceExitCode::default(),
        )
    }

    /// Tell the system that the service is pending stop and provide the time estimate until the
    /// service is stopped.
    pub fn set_pending_stop(&mut self, wait_hint: Duration) -> Result<()> {
        self.report_status(
            ServiceState::StopPending,
            wait_hint,
            ServiceExitCode::default(),
        )
    }

    /// Tell the system that the service is stopped and provide the exit code.
    pub fn set_stopped(&mut self, exit_code: ServiceExitCode) -> Result<()> {
        self.report_status(ServiceState::Stopped, Duration::default(), exit_code)
    }

    /// Private helper to report the service status update.
    fn report_status(
        &mut self,
        next_state: ServiceState,
        wait_hint: Duration,
        exit_code: ServiceExitCode,
    ) -> Result<()> {
        // Automatically bump the checkpoint when updating the pending events to tell the system
        // that the service is making a progress in transition from pending to final state.
        // `wait_hint` should reflect the estimated time for transition to complete.
        let checkpoint = match next_state {
            ServiceState::StartPending
            | ServiceState::StopPending
            | ServiceState::ContinuePending
            | ServiceState::PausePending => self.checkpoint_counter.fetch_add(1, Ordering::SeqCst),
            _ => 0,
        };

        let service_status = ServiceStatus {
            service_type: self.service_type,
            current_state: next_state,
            controls_accepted: accepted_controls_by_state(next_state),
            exit_code,
            checkpoint: checkpoint as u32,
            wait_hint,
            process_id: None,
        };

        tracing::debug!(
            "Update service status: {:?}, checkpoint: {}, wait_hint: {:?}",
            service_status.current_state,
            service_status.checkpoint,
            service_status.wait_hint
        );

        self.status_handle.set_service_status(service_status)
    }
}

/// Returns the list of accepted service events at each stage of the service lifecycle.
fn accepted_controls_by_state(state: ServiceState) -> ServiceControlAccept {
    if state == ServiceState::Running {
        ServiceControlAccept::STOP | ServiceControlAccept::PRESHUTDOWN
    } else {
        ServiceControlAccept::empty()
    }
}
