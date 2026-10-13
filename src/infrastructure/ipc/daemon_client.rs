use std::sync::Arc;

use futures_channel::mpsc::UnboundedSender;
use serde_json::Value;

use crate::infrastructure::runtime::Wake;

use super::client::IpcHandle;

enum Transport {
    Socket(IpcHandle),
    #[cfg(test)]
    Recording {
        calls: std::sync::Mutex<Vec<(String, Value)>>,
        next_id: std::sync::atomic::AtomicU64,
        observer: Option<Arc<dyn Fn(&str, &Value) + Send + Sync>>,
    },
}

#[derive(Clone)]
pub struct DaemonClient {
    transport: Arc<Transport>,
}

impl DaemonClient {
    pub fn start(tx: UnboundedSender<Wake>) -> Self {
        Self { transport: Arc::new(Transport::Socket(IpcHandle::start(tx))) }
    }

    #[cfg(test)]
    pub fn disconnected() -> Self {
        Self { transport: Arc::new(Transport::Socket(IpcHandle::disconnected())) }
    }

    #[cfg(test)]
    pub fn recording() -> Self {
        Self::record(None)
    }

    #[cfg(test)]
    pub fn recording_with_observer(
        observer: impl Fn(&str, &Value) + Send + Sync + 'static,
    ) -> Self {
        Self::record(Some(Arc::new(observer)))
    }

    #[cfg(test)]
    fn record(observer: Option<Arc<dyn Fn(&str, &Value) + Send + Sync>>) -> Self {
        Self {
            transport: Arc::new(Transport::Recording {
                calls: std::sync::Mutex::new(Vec::new()),
                next_id: std::sync::atomic::AtomicU64::new(1),
                observer,
            }),
        }
    }

    pub fn call(&self, method: &str, params: Value) -> u64 {
        match self.transport.as_ref() {
            Transport::Socket(handle) => handle.call(method, params),
            #[cfg(test)]
            Transport::Recording { calls, next_id, observer } => {
                if let Some(observer) = observer {
                    observer(method, &params);
                }
                calls
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .push((method.to_string(), params));
                next_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            }
        }
    }

    #[cfg(test)]
    pub fn drain_calls(&self) -> Vec<(String, Value)> {
        match self.transport.as_ref() {
            Transport::Socket(_) => Vec::new(),
            Transport::Recording { calls, .. } => std::mem::take(
                &mut *calls.lock().unwrap_or_else(std::sync::PoisonError::into_inner),
            ),
        }
    }
}

mod tests;

pub fn apply_token() -> Result<String, getrandom::Error> {
    const TOKEN_BYTES: usize = size_of::<u128>();
    let mut bytes = [0; TOKEN_BYTES];
    getrandom::fill(&mut bytes)?;
    Ok(format!("{:032x}", u128::from_ne_bytes(bytes)))
}
