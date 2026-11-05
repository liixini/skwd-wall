use std::sync::{Arc, Mutex, PoisonError};

#[derive(Debug, Clone, PartialEq)]
pub enum Undo {
    Toggle(bool),
    Pick(String),
    Input { key: String, raw: String },
    Binding,
    Forget,
    Clear,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SettingChange {
    pub place: &'static str,
    pub spot: String,
    pub title: String,
    pub path: String,
    pub was: String,
    pub now: String,
    pub undo: Undo,
    pub spared: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Overview {
    pub changes: Vec<SettingChange>,
}

impl Overview {
    pub fn count(&self, place: &str) -> usize {
        self.changes.iter().filter(|change| change.place == place).count()
    }
}

pub type Scan<'a> = &'a dyn Fn() -> Overview;

#[derive(Default)]
pub struct OverviewMemo {
    slot: Mutex<Option<(u64, Arc<Overview>)>>,
}

impl OverviewMemo {
    pub fn get(&self, stamp: u64, scan: Scan<'_>) -> Arc<Overview> {
        let held = self.slot.lock().unwrap_or_else(PoisonError::into_inner).clone();
        if let Some((held, found)) = held
            && held == stamp
        {
            return found;
        }
        let found = Arc::new(scan());
        *self.slot.lock().unwrap_or_else(PoisonError::into_inner) =
            Some((stamp, Arc::clone(&found)));
        found
    }

    pub fn take(&self) {
        *self.slot.lock().unwrap_or_else(PoisonError::into_inner) = None;
    }
}
