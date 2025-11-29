use anyhow::Result;
use tokio_cron_scheduler::{Job, JobScheduler};
use tokio::sync::mpsc;
use crate::core::messaging::message::Message;
use tracing::{info, error};

pub struct Scheduler {
    sched: JobScheduler,
    message_tx: mpsc::Sender<Message>,
}

impl Scheduler {
    pub async fn new(message_tx: mpsc::Sender<Message>) -> Result<Self> {
        let sched = JobScheduler::new().await?;
        Ok(Self { sched, message_tx })
    }

    pub async fn start(&self) -> Result<()> {
        self.sched.start().await?;
        Ok(())
    }

    /// Add a cron job that sends a message
    pub async fn add_cron_job(&self, schedule: &str, message: Message) -> Result<uuid::Uuid> {
        let tx = self.message_tx.clone();
        let message_clone = message.clone();
        let schedule_str = schedule.to_string();

        // Job::new_async requires a static future or similar, we need to be careful with closures.
        // cloning data into the closure.
        let job = Job::new_async(schedule, move |uuid, _l| {
            let tx = tx.clone();
            let msg = message_clone.clone();
            let sched_str = schedule_str.clone();
            Box::pin(async move {
                info!("Executing cron job {}: {}", uuid, sched_str);
                if let Err(e) = tx.send(msg).await {
                    error!("Failed to send scheduled message: {}", e);
                }
            })
        })?;

        let guid = self.sched.add(job).await?;
        Ok(guid)
    }

    /// Remove a scheduled job
    pub async fn remove_job(&self, uuid: uuid::Uuid) -> Result<()> {
        self.sched.remove(&uuid).await?;
        Ok(())
    }

    // For one-off jobs, tokio-cron-scheduler might be overkill or less precise, 
    // but we can use it if we format the time as a cron string or use its other features if available.
    // For now, let's assume CRON support is the main requirement.
}
