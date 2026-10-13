use std::collections::{HashMap, HashSet};

use super::model::{Effects, MonitorInfo, TileAudio};

#[derive(Default)]
pub(in crate::frontend::effects) struct DisplaySelectionState {
    pub(in crate::frontend::effects) monitors: Vec<MonitorInfo>,
    pub(in crate::frontend::effects) selected_outputs: HashSet<String>,
    pub(in crate::frontend::effects) hovered: Option<String>,
    pub(in crate::frontend::effects) tile_animation: HashMap<String, f32>,
}

impl Effects {
    pub fn clear_monitors(&mut self) {
        self.displays.monitors.clear();
        self.displays.selected_outputs.clear();
        self.displays.hovered = None;
    }
    pub fn connector_for_target(&self, target: &str) -> Option<&str> {
        self.displays
            .monitors
            .iter()
            .find(|monitor| monitor.target == target)
            .map(|monitor| monitor.name.as_str())
    }

    pub fn toggle_output(&mut self, name: &str) {
        if !self.displays.selected_outputs.remove(name) {
            self.displays.selected_outputs.insert(name.to_string());
        }
    }

    pub fn select_only_output(&mut self, target: &str) {
        self.displays.selected_outputs.clear();
        self.displays.selected_outputs.insert(target.to_string());
        self.displays.hovered = Some(target.to_string());
    }

    pub fn record_output_source(
        &mut self,
        target: &str,
        kind: crate::domain::library::catalog::WallpaperKind,
        path: String,
        we_id: String,
        thumb: Option<String>,
    ) {
        let Some(monitor) = self
            .displays
            .monitors
            .iter_mut()
            .find(|monitor| monitor.target == target || monitor.name == target)
        else {
            return;
        };
        monitor.kind = kind;
        monitor.current = path;
        monitor.we_id = we_id;
        monitor.current_thumb = thumb;
    }

    pub fn output_selected(&self, name: &str) -> bool {
        self.displays.selected_outputs.contains(name)
    }

    pub fn has_selected_outputs(&self) -> bool {
        !self.displays.selected_outputs.is_empty()
    }

    pub fn all_selected(&self) -> bool {
        !self.displays.monitors.is_empty()
            && self
                .displays
                .monitors
                .iter()
                .all(|monitor| self.displays.selected_outputs.contains(&monitor.target))
    }

    pub fn toggle_all(&mut self) {
        if self.all_selected() {
            self.displays.selected_outputs.clear();
        } else {
            self.displays.selected_outputs =
                self.displays.monitors.iter().map(|monitor| monitor.target.clone()).collect();
        }
    }

    pub fn set_hover(&mut self, name: Option<String>) {
        self.displays.hovered = name;
    }

    pub fn set_mon_mute(&mut self, name: &str, mute: bool) {
        if let Some(monitor) =
            self.displays.monitors.iter_mut().find(|monitor| monitor.target == name)
        {
            monitor.mute = mute;
        }
    }

    pub fn audio_group_outputs(&self, name: &str) -> Vec<String> {
        let Some(source) = self.displays.monitors.iter().find(|monitor| monitor.target == name)
        else {
            return vec![name.to_string()];
        };
        let mut outputs: Vec<String> = self
            .displays
            .monitors
            .iter()
            .filter(|monitor| same_audio_source(source, monitor))
            .map(|monitor| monitor.target.clone())
            .collect();
        if outputs.is_empty() {
            outputs.push(name.to_string());
        }
        outputs.sort();
        outputs
    }

    pub fn set_mon_fill(&mut self, name: &str, fill: &str) {
        if let Some(monitor) =
            self.displays.monitors.iter_mut().find(|monitor| monitor.target == name)
        {
            monitor.fill = fill.to_string();
        }
    }

    pub fn set_mon_locked(&mut self, name: &str, locked: bool) {
        if let Some(monitor) =
            self.displays.monitors.iter_mut().find(|monitor| monitor.target == name)
        {
            monitor.locked = locked;
        }
    }

    pub fn set_theme_source(&mut self, connector: &str) {
        for monitor in &mut self.displays.monitors {
            monitor.theme_source = monitor.name == connector;
        }
    }

    pub fn set_mon_volume(&mut self, name: &str, volume: u32) {
        if let Some(monitor) =
            self.displays.monitors.iter_mut().find(|monitor| monitor.target == name)
        {
            monitor.volume = volume.min(100);
        }
    }

    pub fn mon_volume(&self, name: &str) -> Option<u32> {
        self.displays
            .monitors
            .iter()
            .find(|monitor| monitor.target == name)
            .map(|monitor| monitor.volume)
    }

    pub fn monitor_apply_details(
        &self,
        name: &str,
    ) -> Option<(crate::domain::library::catalog::WallpaperKind, String, String, bool, u32)> {
        self.displays.monitors.iter().find(|monitor| monitor.target == name).map(|monitor| {
            (
                monitor.kind,
                monitor.current.clone(),
                monitor.we_id.clone(),
                monitor.mute,
                monitor.volume,
            )
        })
    }

    pub fn apply_targets(&self) -> Vec<String> {
        let targets = self.selected_targets();
        if targets.is_empty() || targets.len() == self.displays.monitors.len() {
            vec!["*".to_string()]
        } else {
            targets
        }
    }

    pub fn selected_targets(&self) -> Vec<String> {
        let mut targets: Vec<_> = self.displays.selected_outputs.iter().cloned().collect();
        targets.sort();
        targets
    }

    pub fn tile_audio(&self, monitor: &MonitorInfo, selected: bool) -> TileAudio {
        if matches!(
            monitor.kind,
            crate::domain::library::catalog::WallpaperKind::Video
                | crate::domain::library::catalog::WallpaperKind::We
        ) {
            TileAudio::Live
        } else if selected && self.preview.is_audible() {
            TileAudio::PreApply
        } else {
            TileAudio::None
        }
    }

    pub fn tile_thumb(&self, monitor: &MonitorInfo) -> Option<String> {
        let incoming = self.displays.selected_outputs.contains(&monitor.target)
            || self.displays.hovered.as_deref() == Some(monitor.target.as_str());
        if incoming {
            self.preview.thumb.clone().or_else(|| monitor.current_thumb.clone())
        } else if self.activity.supported {
            monitor.current_thumb.clone()
        } else {
            monitor.current_thumb.clone().or_else(|| self.preview.thumb.clone())
        }
    }
}

fn same_audio_source(left: &MonitorInfo, right: &MonitorInfo) -> bool {
    use crate::domain::library::catalog::WallpaperKind;

    match (left.kind, right.kind) {
        (WallpaperKind::Video, WallpaperKind::Video) => {
            !left.current.is_empty() && left.current == right.current
        }
        (WallpaperKind::We, WallpaperKind::We) => {
            !left.we_id.is_empty() && left.we_id == right.we_id
        }
        _ => false,
    }
}

pub(super) fn align_shared_audio(monitors: &mut [MonitorInfo]) {
    let mut visited = HashSet::new();
    for index in 0..monitors.len() {
        if visited.contains(&monitors[index].target) {
            continue;
        }
        let members: Vec<usize> = (0..monitors.len())
            .filter(|&candidate| same_audio_source(&monitors[index], &monitors[candidate]))
            .collect();
        if members.len() < 2 {
            continue;
        }
        let muted = members.iter().all(|&member| monitors[member].mute);
        let volume = members
            .iter()
            .find(|&&member| !monitors[member].mute)
            .map(|&member| monitors[member].volume)
            .or_else(|| members.iter().map(|&member| monitors[member].volume).max())
            .unwrap_or(100);
        for member in members {
            monitors[member].mute = muted;
            monitors[member].volume = volume;
            visited.insert(monitors[member].target.clone());
        }
    }
}
