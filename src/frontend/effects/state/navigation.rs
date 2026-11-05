use crate::domain::effects::{EffectParam, EffectParamKind, EffectValue};

use super::model::{Effects, EffectsMsg, NAV_CATEGORY, NAV_CONFIG_BASE, NAV_EFFECT};

impl Effects {
    fn effects_in(&self, category: &str) -> Vec<String> {
        self.editor
            .definitions
            .iter()
            .filter(|definition| definition.category == category)
            .map(|definition| definition.id.clone())
            .collect()
    }

    pub(in crate::frontend::effects) fn nav_params(&self) -> Vec<(String, EffectParamKind)> {
        self.selected()
            .map(|definition| {
                definition
                    .params
                    .iter()
                    .filter(|param| {
                        param.kind.is_numeric() || matches!(&param.kind, EffectParamKind::Dropdown)
                    })
                    .map(|param| (param.id.clone(), param.kind.clone()))
                    .collect()
            })
            .unwrap_or_default()
    }

    fn param_def(&self, id: &str) -> Option<&EffectParam> {
        self.selected()?.params.iter().find(|param| param.id == id)
    }

    pub fn nav_stops(&self) -> usize {
        if !self.has_effects_page() {
            return 0;
        }
        NAV_CONFIG_BASE + self.nav_params().len()
    }

    pub fn nav_move(&mut self, delta: i32) {
        let stops = self.nav_stops();
        if stops == 0 {
            return;
        }
        let current = self.editor.nav_focus.min(stops - 1);
        self.editor.nav_focus =
            crate::frontend::nav::step(current, delta, stops, false).unwrap_or(current);
    }

    pub fn nav_horizontal(&self, delta: i32) -> Option<EffectsMsg> {
        if !self.has_effects_page() {
            return None;
        }
        let focus = self.editor.nav_focus.min(self.nav_stops().saturating_sub(1));
        match focus {
            NAV_CATEGORY => {
                let categories = self.categories();
                let current = self.editor.category_of(&self.editor.selected_id);
                let index = categories.iter().position(|category| *category == current)?;
                let next = crate::frontend::nav::step(index, delta, categories.len(), false)?;
                Some(EffectsMsg::Select(self.first_effect_of(&categories[next])))
            }
            NAV_EFFECT => {
                let ids = self.effects_in(&self.editor.category_of(&self.editor.selected_id));
                let index = ids.iter().position(|id| *id == self.editor.selected_id)?;
                let next = crate::frontend::nav::step(index, delta, ids.len(), false)?;
                Some(EffectsMsg::Select(ids[next].clone()))
            }
            row => {
                let (id, kind) = self.nav_params().get(row - NAV_CONFIG_BASE)?.clone();
                let parameter = self.param_def(&id)?;
                if matches!(kind, EffectParamKind::Dropdown) {
                    let current = self
                        .editor
                        .values
                        .get(&id)
                        .and_then(EffectValue::as_str)
                        .unwrap_or_default();
                    let index = parameter
                        .options
                        .iter()
                        .position(|option| option.mode == current)
                        .unwrap_or(0);
                    let next =
                        crate::frontend::nav::step(index, delta, parameter.options.len(), false)?;
                    Some(EffectsMsg::SetChoice(id, parameter.options[next].mode.clone()))
                } else {
                    let current = self
                        .editor
                        .values
                        .get(&id)
                        .and_then(EffectValue::as_f64)
                        .unwrap_or(parameter.min);
                    let next = (current + f64::from(delta) * parameter.step)
                        .clamp(parameter.min, parameter.max);
                    (next != current).then_some(EffectsMsg::SetNum(id, next))
                }
            }
        }
    }

    pub fn nav_scroll(&self) -> Option<(&'static str, f32)> {
        if !self.has_effects_page() {
            return None;
        }
        let fraction = |index: usize, len: usize| {
            if len <= 1 { 0.0 } else { index as f32 / (len - 1) as f32 }
        };
        match self.editor.nav_focus.min(self.nav_stops().saturating_sub(1)) {
            NAV_CATEGORY => {
                let categories = self.categories();
                let index = categories.iter().position(|category| {
                    *category == self.editor.category_of(&self.editor.selected_id)
                })?;
                Some(("fx.cats", fraction(index, categories.len())))
            }
            NAV_EFFECT => {
                let ids = self.effects_in(&self.editor.category_of(&self.editor.selected_id));
                let index = ids.iter().position(|id| *id == self.editor.selected_id)?;
                Some(("fx.reel", fraction(index, ids.len())))
            }
            row => {
                let (id, kind) = self.nav_params().get(row - NAV_CONFIG_BASE)?.clone();
                if !matches!(kind, EffectParamKind::Dropdown) {
                    return None;
                }
                let parameter = self.param_def(&id)?;
                let current =
                    self.editor.values.get(&id).and_then(EffectValue::as_str).unwrap_or_default();
                let index = parameter.options.iter().position(|option| option.mode == current)?;
                Some(("fx.params", fraction(index, parameter.options.len())))
            }
        }
    }
}
