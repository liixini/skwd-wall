use crate::domain::theme::{Candidate, THEME_ROLE_COUNT, hex_to_hsv, hsv_to_hex};

pub struct ThemeDesigner {
    pub wallpaper: Option<crate::contracts::daemon::CurrentTheme>,
    pub profile_enabled: bool,
    pub role_filter: String,
    pub error: Option<String>,
    pub candidate: Candidate,
    pub initial: Candidate,
    loaded: Candidate,
    pub initial_name: String,
    pub selected: usize,
    pub hsv: (f32, f32, f32),
    pub hex_buf: String,
    pub name_buf: String,
    pub recent: Vec<String>,
    pub open: crate::frontend::animation::Tween,
    selected_preset: Option<String>,
    initial_preset: Option<String>,
    saved_identity: Option<String>,
    armed_delete: Option<String>,
}

impl ThemeDesigner {
    pub fn new(candidate: Candidate, name: String) -> Self {
        let hsv = hex_to_hsv(&candidate.colors[0]).unwrap_or((265.0, 0.45, 0.6));
        let hex_buf = candidate.colors[0].clone();
        let saved_identity = (!name.is_empty()).then(|| name.clone());
        let mut open = crate::frontend::animation::MotionProfile::default()
            .tween(0.0, crate::frontend::animation::MotionTier::Fast);
        open.retarget(1.0);
        Self {
            wallpaper: None,
            profile_enabled: false,
            role_filter: String::new(),
            error: None,
            initial: candidate.clone(),
            loaded: candidate.clone(),
            initial_name: name.clone(),
            candidate,
            selected: 0,
            hsv,
            hex_buf,
            name_buf: name,
            recent: Vec::new(),
            open,
            selected_preset: None,
            initial_preset: None,
            saved_identity,
            armed_delete: None,
        }
    }

    pub fn animating(&self) -> bool {
        !self.open.settled()
    }

    pub fn tick(&mut self, dt: f32) {
        self.open.tick(dt);
    }

    pub fn ease(&self) -> f32 {
        crate::frontend::animation::ease_out_cubic(self.open.x)
    }

    pub fn new_from_preset(candidate: Candidate, preset: String) -> Self {
        let mut designer = Self::new(candidate, String::new());
        designer.initial_preset = Some(preset.clone());
        designer.selected_preset = Some(preset);
        designer
    }

    pub fn selected_preset(&self) -> Option<&str> {
        self.selected_preset.as_deref()
    }

    pub fn armed_delete(&self) -> Option<&str> {
        self.armed_delete.as_deref()
    }

    pub fn request_delete(&mut self, name: String) -> bool {
        if self.armed_delete.as_deref() == Some(name.as_str()) {
            self.armed_delete = None;
            true
        } else {
            self.armed_delete = Some(name);
            false
        }
    }

    pub fn clear_delete_confirmation(&mut self) {
        self.armed_delete = None;
    }

    pub fn dirty(&self) -> bool {
        !self.candidate.same_colors(&self.initial) || self.name_buf != self.initial_name
    }

    pub fn reset(&mut self) {
        self.candidate = self.initial.clone();
        self.loaded.clone_from(&self.initial);
        self.name_buf.clone_from(&self.initial_name);
        self.selected_preset.clone_from(&self.initial_preset);
        self.saved_identity = (!self.initial_name.is_empty()).then(|| self.initial_name.clone());
        self.select_role(self.selected);
    }

    pub fn loaded_colour(&self) -> &str {
        if self.candidate.dark == self.loaded.dark {
            &self.loaded.colors[self.selected]
        } else {
            &self.loaded.alternate[self.selected]
        }
    }

    pub fn colour_changed(&self) -> bool {
        self.candidate.colors[self.selected] != self.loaded_colour()
            || self.hex_buf != self.loaded_colour()
    }

    pub fn reset_colour(&mut self) {
        self.candidate.colors[self.selected] = self.loaded_colour().to_string();
        self.select_role(self.selected);
    }

    pub fn set_variant(&mut self, dark: bool) {
        self.candidate.set_dark(dark);
        self.select_role(self.selected);
    }

    pub fn select_role(&mut self, idx: usize) {
        if idx >= THEME_ROLE_COUNT {
            return;
        }
        self.selected = idx;
        self.hex_buf = self.candidate.colors[idx].clone();
        if let Some(hsv) = hex_to_hsv(&self.hex_buf) {
            self.hsv = hsv;
        }
    }

    fn commit_hsv(&mut self) {
        let hex = hsv_to_hex(self.hsv.0, self.hsv.1, self.hsv.2);
        self.hex_buf.clone_from(&hex);
        self.candidate.colors[self.selected] = hex;
        self.selected_preset = None;
    }

    pub fn set_hue(&mut self, h: f32) {
        self.hsv.0 = h.rem_euclid(360.0);
        if self.hsv.1 == 0.0 {
            self.hsv.1 = 0.5;
        }
        if self.hsv.2 == 0.0 {
            self.hsv.2 = 0.5;
        }
        self.commit_hsv();
    }

    pub fn set_sv(&mut self, s: f32, v: f32) {
        self.hsv.1 = s.clamp(0.0, 1.0);
        self.hsv.2 = v.clamp(0.0, 1.0);
        self.commit_hsv();
    }

    pub fn apply_hex(&mut self) -> bool {
        let Some(hsv) = hex_to_hsv(&self.hex_buf) else {
            return false;
        };
        self.hsv = hsv;
        let normal = hsv_to_hex(hsv.0, hsv.1, hsv.2);
        self.candidate.colors[self.selected].clone_from(&normal);
        self.hex_buf = normal;
        self.selected_preset = None;
        true
    }

    pub fn start_from(&mut self, candidate: Candidate) {
        self.detach_saved_identity();
        self.loaded.clone_from(&candidate);
        self.candidate = candidate;
        self.selected_preset = None;
        self.select_role(self.selected);
    }

    pub fn start_from_preset(&mut self, preset: String, candidate: Candidate) {
        self.detach_saved_identity();
        self.loaded.clone_from(&candidate);
        self.candidate = candidate;
        self.selected_preset = Some(preset);
        self.select_role(self.selected);
    }

    pub fn load_saved(&mut self, name: String, candidate: Candidate) {
        self.loaded.clone_from(&candidate);
        self.candidate = candidate;
        self.name_buf.clone_from(&name);
        self.saved_identity = Some(name);
        self.selected_preset = None;
        self.select_role(self.selected);
    }

    pub fn set_name(&mut self, name: String) {
        if self.saved_identity.as_deref().is_some_and(|saved| name.trim() != saved) {
            self.saved_identity = None;
        }
        self.name_buf = name;
    }

    pub fn mark_saved(&mut self, name: String) {
        self.initial.clone_from(&self.candidate);
        self.initial_name.clone_from(&name);
        self.initial_preset.clone_from(&self.selected_preset);
        self.name_buf.clone_from(&name);
        self.saved_identity = Some(name);
        self.clear_delete_confirmation();
    }

    fn detach_saved_identity(&mut self) {
        if self.saved_identity.as_deref().is_some_and(|saved| self.name_buf.trim() == saved) {
            self.name_buf.clear();
        }
        self.saved_identity = None;
    }

    pub fn push_recent(&mut self) {
        let hex = self.candidate.colors[self.selected].clone();
        self.recent.retain(|existing| *existing != hex);
        self.recent.push(hex);
        if self.recent.len() > 8 {
            self.recent.remove(0);
        }
    }

    pub fn pick_recent(&mut self, hex: &str) {
        self.candidate.colors[self.selected] = hex.to_string();
        self.hex_buf = hex.to_string();
        self.selected_preset = None;
        if let Some(hsv) = hex_to_hsv(hex) {
            self.hsv = hsv;
        }
    }
}

mod tests;
