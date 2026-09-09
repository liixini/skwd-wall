use crate::contracts::settings::{SettingsSource, keys, schema::Setting};
use crate::i18n::tr;

use super::{ActionId, Card, Control, Row};

pub(super) struct Builder<'a> {
    pub(super) cfg: &'a dyn SettingsSource,
    pub(super) cards: Vec<(Card, Vec<Row>)>,
}

impl Builder<'_> {
    pub(super) fn card(&mut self, title: &'static str, subtitle: &'static str) {
        self.cards.push((Card { title, subtitle }, Vec::new()));
    }

    pub(super) fn info(&mut self, title: &str, desc: &str) {
        self.row(title, desc, Control::Static);
    }

    pub(super) fn row(&mut self, title: &str, desc: &str, control: Control) {
        if let Some((_, rows)) = self.cards.last_mut() {
            rows.push(Row { title: title.to_string(), desc: desc.to_string(), control });
        }
    }

    pub(super) fn toggle(&mut self, title: &str, desc: &str, path: &str) {
        let value = self.cfg.flag(path);
        self.row(title, desc, Control::Toggle { path: path.to_string(), value });
    }

    pub(super) fn toggle_setting(&mut self, title: &str, desc: &str, setting: Setting<bool>) {
        self.toggle(title, desc, setting.path());
    }

    pub(super) fn toggle_default_true(&mut self, title: &str, desc: &str, path: &str) {
        let value = self.cfg.flag_default_true(path);
        self.row(title, desc, Control::Toggle { path: path.to_string(), value });
    }

    pub(super) fn num(&mut self, title: &str, desc: &str, path: &str, unit: &'static str) {
        self.row(
            title,
            desc,
            Control::Number { key: path.to_string(), path: path.to_string(), unit },
        );
    }

    pub(super) fn num_setting(
        &mut self,
        title: &str,
        desc: &str,
        setting: Setting<f64>,
        unit: &'static str,
    ) {
        self.row(
            title,
            desc,
            Control::Number {
                key: setting.path().to_string(),
                path: setting.path().to_string(),
                unit,
            },
        );
    }

    pub(super) fn text_field(
        &mut self,
        title: &str,
        desc: &str,
        path: &str,
        placeholder: &'static str,
    ) {
        self.row(
            title,
            desc,
            Control::TextField { key: path.to_string(), path: path.to_string(), placeholder },
        );
    }

    pub(super) fn key_binding(&mut self, title: &str, path: &str, default: &'static str) {
        self.row(
            title,
            "",
            Control::KeyBinding { key: path.to_string(), path: path.to_string(), default },
        );
    }

    pub(super) fn dropdown(
        &mut self,
        title: &str,
        desc: &str,
        path: &str,
        options: &[(&str, &str)],
    ) {
        let current = self.cfg.text(path);
        self.dropdown_cur(title, desc, path, options, current);
    }

    pub(super) fn dropdown_setting(
        &mut self,
        title: &str,
        desc: &str,
        setting: Setting<String>,
        options: &[(&str, &str)],
    ) {
        self.dropdown(title, desc, setting.path(), options);
    }

    pub(super) fn dropdown_cur(
        &mut self,
        title: &str,
        desc: &str,
        path: &str,
        options: &[(&str, &str)],
        current: String,
    ) {
        self.row(
            title,
            desc,
            Control::Dropdown {
                palettes: Vec::new(),
                path: path.to_string(),
                options: options
                    .iter()
                    .map(|(key, label)| (key.to_string(), label.to_string()))
                    .collect(),
                current,
            },
        );
    }

    pub(super) fn dynamic_dropdown(
        &mut self,
        title: &str,
        desc: &str,
        path: &str,
        options: Vec<(String, String)>,
    ) {
        let current = self.cfg.text(path);
        self.row(
            title,
            desc,
            Control::Dropdown { palettes: Vec::new(), path: path.to_string(), options, current },
        );
    }

    pub(super) fn theme_dropdown(
        &mut self,
        title: &str,
        desc: &str,
        path: &str,
        themes: &[String],
    ) {
        let current = self.cfg.text(path);
        let options = if themes.is_empty() {
            vec![(current.clone(), current.clone())]
        } else {
            themes.iter().map(|theme| (theme.clone(), theme.clone())).collect()
        };
        self.row(
            title,
            desc,
            Control::Dropdown { palettes: Vec::new(), path: path.to_string(), options, current },
        );
    }

    pub(super) fn chips(&mut self, title: &str, desc: &str, path: &str, options: &[(&str, &str)]) {
        self.chips_with_disabled(title, desc, path, options, &[]);
    }

    pub(super) fn chips_with_disabled(
        &mut self,
        title: &str,
        desc: &str,
        path: &str,
        options: &[(&str, &str)],
        disabled: &[&str],
    ) {
        let current = self.cfg.text(path);
        self.row(
            title,
            desc,
            Control::Chips {
                path: path.to_string(),
                options: options
                    .iter()
                    .map(|(key, label)| ((*key).to_string(), (*label).to_string()))
                    .collect(),
                current,
                disabled: disabled.iter().map(|key| (*key).to_string()).collect(),
            },
        );
    }

    pub(super) fn dynamic_chips(
        &mut self,
        title: &str,
        desc: &str,
        path: &str,
        options: Vec<(String, String)>,
    ) {
        let current = self.cfg.text(path);
        self.row(
            title,
            desc,
            Control::Chips { path: path.to_string(), options, current, disabled: Vec::new() },
        );
    }

    pub(super) fn action(&mut self, title: &str, desc: &str, id: ActionId, label: &str) {
        self.row(title, desc, Control::ActionBtn { id, label: label.to_string() });
    }

    pub(super) fn details(
        &mut self,
        title: &str,
        desc: &str,
        id: String,
        summary: String,
        build: impl FnOnce(&mut Self),
    ) {
        let Some(card) = self.cards.len().checked_sub(1) else {
            return;
        };
        let start = self.cards[card].1.len();
        build(self);
        let rows = self.cards[card].1.split_off(start);
        self.row(title, desc, Control::Details { id, summary, rows });
    }

    pub(super) fn motion_weights(&mut self) {
        self.row(
            tr("settings-motion-weights-label"),
            tr("settings-motion-weights-desc"),
            Control::MotionWeights {
                weights: vec![
                    (
                        tr("settings-motion-fast").to_string(),
                        String::from(keys::motion::FAST_MS),
                        ActionId::ResetMotionFast,
                    ),
                    (
                        tr("settings-motion-standard").to_string(),
                        String::from(keys::motion::STANDARD_MS),
                        ActionId::ResetMotionStandard,
                    ),
                    (
                        tr("settings-motion-slow").to_string(),
                        String::from(keys::motion::SLOW_MS),
                        ActionId::ResetMotionSlow,
                    ),
                ],
            },
        );
    }
}
