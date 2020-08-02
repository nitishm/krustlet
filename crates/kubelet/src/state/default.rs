//! Default implementation for state machine graph.

use crate::pod::Pod;
use crate::pod::Phase;
use crate::state;
use crate::state::State;
use crate::state::Transition;
use log::error;
use std::sync::Arc;

#[async_trait::async_trait]
/// Trait for implementing default state machine.
pub trait DefaultStateProvider: 'static + Sync + Send {
    /// A new Pod has been created.
    async fn registered(&self, _pod: &Pod) -> anyhow::Result<()> {
        Ok(())
    }

    /// Pull images for containers.
    async fn image_pull(&self, _pod: &Pod) -> anyhow::Result<()> {
        Ok(())
    }

    /// Image pull has failed several times.
    async fn image_pull_backoff(&self, _pod: &Pod) -> anyhow::Result<()> {
        tokio::time::delay_for(std::time::Duration::from_secs(30)).await;
        Ok(())
    }

    /// Mount volumes for containers.
    async fn volume_mount(&self, _pod: &Pod) -> anyhow::Result<()> {
        Ok(())
    }

    /// Volume mount has failed several times.
    async fn volume_mount_backoff(&self, _pod: &Pod) -> anyhow::Result<()> {
        tokio::time::delay_for(std::time::Duration::from_secs(30)).await;
        Ok(())
    }

    /// Start containers.
    async fn starting(&self, _pod: &Pod) -> anyhow::Result<()> {
        Ok(())
    }

    /// Running state.
    async fn running(&self, _pod: &Pod) -> anyhow::Result<()> {
        Ok(())
    }

    /// Handle any errors, on Ok, will transition to Starting again.
    async fn error(&self, _pod: &Pod) -> anyhow::Result<()> {
        tokio::time::delay_for(std::time::Duration::from_secs(30)).await;
        Ok(())
    }
}

//
// * Would be nice to support passing types to the next state (error messages, etc.).
// * We probably need to explore a more concise way for describing status patches.
// * Can we offer a macro that doesnt require a trait?
// * How can we expose a nice way for updating container statuses?
//

state!(
    /// The Kubelet is aware of the Pod.
    Registered,
    DefaultStateProvider,
    ImagePull,
    Error,
    {
        match provider.registered(pod).await {
            Ok(_) => Ok(Transition::Advance(ImagePull)),
            Err(e) => {
                error!(
                    "Pod {} encountered an error in state {:?}: {:?}",
                    pod.name(),
                    Self,
                    e
                );
                Ok(Transition::Error(Error))
            }
        }
    },
    { 
        Ok(serde_json::json!(
            {
                "metadata": {
                    "resourceVersion": "",
                },
                "status": {
                    "phase": Phase::Pending,
                    "reason": "Registered",
                    "containerStatuses": Vec::<()>::new(),
                    "initContainerStatuses": Vec::<()>::new(),
                }
            }
        )) 
    }
);

state!(
    /// The Kubelet is pulling container images.
    ImagePull,
    DefaultStateProvider,
    VolumeMount,
    ImagePullBackoff,
    {
        match provider.image_pull(pod).await {
            Ok(_) => Ok(Transition::Advance(VolumeMount)),
            Err(e) => {
                error!(
                    "Pod {} encountered an error in state {:?}: {:?}",
                    pod.name(),
                    Self,
                    e
                );
                Ok(Transition::Error(ImagePullBackoff))
            }
        }
    },
    {
        Ok(serde_json::json!(
            {
                "metadata": {
                    "resourceVersion": "",
                },
                "status": {
                    "phase": Phase::Pending,
                    "reason": "ImagePull",
                    "containerStatuses": Vec::<()>::new(),
                    "initContainerStatuses": Vec::<()>::new(),
                }
            }
        )) 
    }
);

state!(
    /// Image pull has failed several times.
    ImagePullBackoff,
    DefaultStateProvider,
    ImagePull,
    ImagePullBackoff,
    {
        match provider.image_pull_backoff(pod).await {
            Ok(_) => Ok(Transition::Advance(ImagePull)),
            Err(e) => {
                error!(
                    "Pod {} encountered an error in state {:?}: {:?}",
                    pod.name(),
                    Self,
                    e
                );
                Ok(Transition::Error(ImagePullBackoff))
            }
        }
    },
    {
        Ok(serde_json::json!(
            {
                "metadata": {
                    "resourceVersion": "",
                },
                "status": {
                    "phase": Phase::Pending,
                    "reason": "ImagePullBackoff",
                    "containerStatuses": Vec::<()>::new(),
                    "initContainerStatuses": Vec::<()>::new(),
                }
            }
        )) 
    }
);

state!(
    /// The Kubelet is provisioning volumes.
    VolumeMount,
    DefaultStateProvider,
    Starting,
    VolumeMountBackoff,
    {
        match provider.volume_mount(pod).await {
            Ok(_) => Ok(Transition::Advance(Starting)),
            Err(e) => {
                error!(
                    "Pod {} encountered an error in state {:?}: {:?}",
                    pod.name(),
                    Self,
                    e
                );
                Ok(Transition::Error(VolumeMountBackoff))
            }
        }
    },
    {
        Ok(serde_json::json!(
            {
                "metadata": {
                    "resourceVersion": "",
                },
                "status": {
                    "phase": Phase::Pending,
                    "reason": "VolumeMount",
                    "containerStatuses": Vec::<()>::new(),
                    "initContainerStatuses": Vec::<()>::new(),
                }
            }
        )) 
    }
);

state!(
    /// Volume mount has failed several times.
    VolumeMountBackoff,
    DefaultStateProvider,
    VolumeMount,
    VolumeMountBackoff,
    {
        match provider.volume_mount_backoff(pod).await {
            Ok(_) => Ok(Transition::Advance(VolumeMount)),
            Err(e) => {
                error!(
                    "Pod {} encountered an error in state {:?}: {:?}",
                    pod.name(),
                    Self,
                    e
                );
                Ok(Transition::Error(VolumeMountBackoff))
            }
        }
    },
    {
        Ok(serde_json::json!(
            {
                "metadata": {
                    "resourceVersion": "",
                },
                "status": {
                    "phase": Phase::Pending,
                    "reason": "VolumeMountBackoff",
                    "containerStatuses": Vec::<()>::new(),
                    "initContainerStatuses": Vec::<()>::new(),
                }
            }
        )) 
    }
);

state!(
    /// The Kubelet is starting the containers.
    Starting,
    DefaultStateProvider,
    Running,
    Error,
    {
        match provider.starting(pod).await {
            Ok(_) => Ok(Transition::Advance(Running)),
            Err(e) => {
                error!(
                    "Pod {} encountered an error in state {:?}: {:?}",
                    pod.name(),
                    Self,
                    e
                );
                Ok(Transition::Error(Error))
            }
        }
    },
    {
        Ok(serde_json::json!(
            {
                "metadata": {
                    "resourceVersion": "",
                },
                "status": {
                    "phase": Phase::Pending,
                    "reason": "Starting",
                    "containerStatuses": Vec::<()>::new(),
                    "initContainerStatuses": Vec::<()>::new(),
                }
            }
        )) 
    }
);

state!(
    /// The Kubelet is provisioning volumes.
    Running,
    DefaultStateProvider,
    Finished,
    Error,
    {
        match provider.running(pod).await {
            Ok(_) => Ok(Transition::Advance(Finished)),
            Err(e) => {
                error!(
                    "Pod {} encountered an error in state {:?}: {:?}",
                    pod.name(),
                    Self,
                    e
                );
                Ok(Transition::Error(Error))
            }
        }
    },
    {
        Ok(serde_json::json!(
            {
                "metadata": {
                    "resourceVersion": "",
                },
                "status": {
                    "phase": Phase::Running,
                    "reason": "Running",
                    "containerStatuses": Vec::<()>::new(),
                    "initContainerStatuses": Vec::<()>::new(),
                }
            }
        )) 
    }
);

state!(
    /// The Pod encountered an error.
    Error,
    DefaultStateProvider,
    Starting,
    Error,
    {
        match provider.error(pod).await {
            Ok(_) => Ok(Transition::Advance(Starting)),
            Err(e) => {
                error!(
                    "Pod {} encountered an error in state {:?}: {:?}",
                    pod.name(),
                    Self,
                    e
                );
                Ok(Transition::Error(Error))
            }
        }
    },
    {
        Ok(serde_json::json!(
            {
                "metadata": {
                    "resourceVersion": "",
                },
                "status": {
                    "phase": Phase::Failed,
                    "reason": "Error",
                    "containerStatuses": Vec::<()>::new(),
                    "initContainerStatuses": Vec::<()>::new(),
                }
            }
        )) 
    }
);

state!(
    /// The Pod was terminated before it completed.
    Terminated,
    DefaultStateProvider,
    Terminated,
    Terminated,
    { Ok(Transition::Complete(Ok(()))) },
    {
        Ok(serde_json::json!(
            {
                "metadata": {
                    "resourceVersion": "",
                },
                "status": {
                    "phase": Phase::Failed,
                    "reason": "Failed",
                    "containerStatuses": Vec::<()>::new(),
                    "initContainerStatuses": Vec::<()>::new(),
                }
            }
        )) 
    }
);

state!(
    /// The Pod completed execution with no errors.
    Finished,
    DefaultStateProvider,
    Finished,
    Finished,
    { Ok(Transition::Complete(Ok(()))) },
    {
        Ok(serde_json::json!(
            {
                "metadata": {
                    "resourceVersion": "",
                },
                "status": {
                    "phase": Phase::Succeeded,
                    "reason": "Failed",
                    "containerStatuses": Vec::<()>::new(),
                    "initContainerStatuses": Vec::<()>::new(),
                }
            }
        )) 
    }
);
