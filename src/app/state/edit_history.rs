use serde_json::Value;

const MAX_STATES: usize = 100;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct EditSnapshot {
    pub(crate) values: Vec<(String, Option<Value>)>,
}

#[derive(Clone, Debug)]
pub(crate) struct EditEntry<S = EditSnapshot> {
    pub(crate) label: String,
    pub(crate) snapshot: S,
}

#[derive(Clone, Debug)]
pub(crate) struct EditHistory<S = EditSnapshot> {
    original: S,
    entries: Vec<EditEntry<S>>,
    index: usize,
}

impl<S: Clone + PartialEq> EditHistory<S> {
    pub(crate) fn new(original: S, label: String) -> Self {
        Self {
            original: original.clone(),
            entries: vec![EditEntry { label, snapshot: original }],
            index: 0,
        }
    }

    pub(crate) fn original(&self) -> &S {
        &self.original
    }

    pub(crate) fn current(&self) -> &S {
        &self.entries[self.index].snapshot
    }

    pub(crate) fn index(&self) -> usize {
        self.index
    }

    pub(crate) fn entries(&self) -> &[EditEntry<S>] {
        &self.entries
    }

    pub(crate) fn record(&mut self, snapshot: S, label: String) -> bool {
        if *self.current() == snapshot {
            return false;
        }

        self.entries.truncate(self.index + 1);
        self.entries.push(EditEntry { label, snapshot });
        if self.entries.len() > MAX_STATES {
            self.entries.remove(1);
        }
        self.index = self.entries.len() - 1;
        true
    }

    pub(crate) fn undo(&mut self) -> Option<&S> {
        if self.index == 0 {
            return None;
        }
        self.index -= 1;
        Some(self.current())
    }

    pub(crate) fn redo(&mut self) -> Option<&S> {
        if self.index + 1 == self.entries.len() {
            return None;
        }
        self.index += 1;
        Some(self.current())
    }

    pub(crate) fn amend(&mut self, snapshot: S) {
        self.entries.truncate(self.index + 1);
        self.entries[self.index].snapshot = snapshot;
    }
}

#[cfg(test)]
#[path = "edit_history/tests.rs"]
mod tests;
