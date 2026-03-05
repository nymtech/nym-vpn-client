// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use tokio::{io::Result, runtime::Builder, sync::mpsc, task::LocalSet};
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub(crate) struct LocalSpawner<T> {
    task_sender: mpsc::UnboundedSender<T>,
}

impl<T> Clone for LocalSpawner<T> {
    fn clone(&self) -> Self {
        Self {
            task_sender: self.task_sender.clone(),
        }
    }
}

impl<T> LocalSpawner<T>
where
    T: Send + 'static,
{
    pub(crate) fn new<F, Fut>(run_task: F, shutdown_token: CancellationToken) -> Result<Self>
    where
        F: Fn(T, CancellationToken) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + 'static,
    {
        let (task_sender, mut task_receiver) = mpsc::unbounded_channel();

        let rt = Builder::new_current_thread().enable_all().build()?;

        std::thread::spawn(move || {
            let local = LocalSet::new();

            local.spawn_local(async move {
                loop {
                    tokio::select! {
                        biased;
                        _ = shutdown_token.cancelled() => {
                            break;
                        }
                        Some(new_task) = task_receiver.recv() => {
                            tokio::task::spawn_local(run_task(new_task, shutdown_token.child_token()));
                        }
                    }
                }
            });

            rt.block_on(local);
        });

        Ok(Self { task_sender })
    }

    pub(crate) fn spawn(&self, task: T) {
        if self.task_sender.send(task).is_err() {
            tracing::error!("Thread with LocalSet has shut down");
        }
    }
}
