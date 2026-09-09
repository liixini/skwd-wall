use iced::Task;
use log::info;
use serde_json::{Value, json};

#[allow(clippy::wildcard_imports)]
use super::super::*;

use crate::frontend::effects::EffectsMsg;

pub(super) fn update(app: &mut App, msg: EffectsMsg) -> Task<Message> {
    match msg {
        EffectsMsg::Select(id) => {
            effects_edit(app, |eff| eff.select_effect(id));
            if effects_shader_live(app) {
                app.retick();
            } else {
                effects_do_preview(app);
            }
            Task::none()
        }
        EffectsMsg::SetNum(id, val) => {
            effects_edit(app, |eff| eff.set_num(&id, val));
            if effects_shader_live(app) {
                app.retick();
            } else {
                effects_request_preview(app);
            }
            Task::none()
        }
        EffectsMsg::SetStr(id, val) => {
            effects_edit(app, |eff| eff.set_str(&id, val));
            Task::none()
        }
        EffectsMsg::SetChoice(id, val) => {
            effects_edit(app, |eff| eff.set_str(&id, val));
            effects_request_preview(app);
            Task::none()
        }
        EffectsMsg::Preview => {
            effects_request_preview(app);
            Task::none()
        }
        EffectsMsg::ToggleSaved => {
            effects_edit(app, crate::frontend::effects::Effects::toggle_saved);
            if effects_shader_live(app)
                || app
                    .panels
                    .effects
                    .as_ref()
                    .is_some_and(|effects| effects.preview_effects().is_empty())
            {
                app.retick();
            } else {
                effects_request_preview(app);
            }
            Task::none()
        }
        EffectsMsg::Apply => {
            effects_do_commit(app);
            if let Some(eff) = app.panels.effects.as_mut()
                && eff.is_busy()
            {
                eff.begin_apply();
            }
            Task::none()
        }
        EffectsMsg::MonitorToggle(output) => {
            effects_edit(app, |eff| eff.toggle_output(&output));
            app.retick();
            Task::none()
        }
        EffectsMsg::MonitorToggleAll => {
            effects_edit(app, crate::frontend::effects::Effects::toggle_all);
            app.retick();
            Task::none()
        }
        EffectsMsg::MonitorLock(output, locked) => {
            let connector = app
                .panels
                .effects
                .as_ref()
                .and_then(|effects| effects.connector_for_target(&output))
                .unwrap_or(&output)
                .to_string();
            let path = format!("{}.{}", skwd_config::keys::display::OUTPUT_LOCKS, connector);
            super::settings_policy::save_value(app, &path, &json!(locked));
            effects_edit(app, |effects| effects.set_mon_locked(&output, locked));
            app.invalidate_settings();
            app.retick();
            Task::none()
        }
        EffectsMsg::MonitorHover(name) => {
            effects_edit(app, |eff| eff.set_hover(name));
            app.retick();
            Task::none()
        }
        EffectsMsg::MonitorApplySelected => monitor_apply_selected(app),
        EffectsMsg::SrcMute(mute) => {
            effects_edit(app, |effects| effects.set_source_mute(mute));
            Task::none()
        }
        EffectsMsg::SrcVolume(vol) => {
            effects_edit(app, |effects| {
                effects.set_source_volume(vol);
                effects.set_source_mute(vol == 0);
            });
            Task::none()
        }
    }
}

pub(super) fn audio_update(
    app: &mut App,
    msg: crate::frontend::audio_panel::AudioMsg,
) -> Task<Message> {
    use crate::frontend::audio_panel::AudioMsg;
    match msg {
        AudioMsg::MonPause(output, paused) => {
            app.call_tracked(
                "wall.set_paused",
                json!({"output": output, "paused": paused}),
                Pending::AudioPause,
            );
            Task::none()
        }
        AudioMsg::VolumeStep(delta) => audio_volume_step(app, delta),
        AudioMsg::MonMute(output, mute) => audio_mon_mute(app, &output, mute),
        AudioMsg::MonVolume(output, vol) => audio_mon_volume(app, &output, vol),
        AudioMsg::MonVolumeRelease(output) => audio_mon_volume_release(app, &output),
    }
}

pub(super) fn audio_volume_step(app: &mut App, delta: i32) -> Task<Message> {
    let cur = app.config.wallpaper_volume() as i32;
    let next = (cur + delta).clamp(0, 100) as u32;
    if next == cur as u32 {
        return Task::none();
    }
    super::settings_policy::save_audio_volume(app, next, true);
    app.chrome.bar.cache.clear();
    Task::none()
}

pub(super) fn toggle_audio_panel(app: &mut App) -> Task<Message> {
    if app.panels.audio.is_some() {
        app.panels.audio = None;
        return Task::none();
    }
    app.panels.audio =
        Some(crate::frontend::audio_panel::AudioPanel::new_with_motion(app.motion_profile()));
    app.call_tracked("wall.outputs", json!({}), Pending::AudioOutputs);
    app.retick();
    Task::none()
}

pub(super) fn mon_fill(app: &mut App, output: &str, mode: &str) -> Task<Message> {
    let connector = app
        .panels
        .effects
        .as_ref()
        .and_then(|effects| effects.connector_for_target(output))
        .unwrap_or(output)
        .to_string();
    let clearing = app
        .panels
        .effects
        .as_ref()
        .and_then(|effects| effects.monitors().iter().find(|monitor| monitor.target == output))
        .is_some_and(|mon| mon.fill == mode);
    let key = format!("display.fillModes.{connector}");
    if clearing {
        app.config.remove_key(&key);
        app.config.persist();
    } else {
        app.config.save_key(&key, Value::String(mode.to_string()));
    }
    let applied = if clearing { "" } else { mode };
    let target = app.panels.effects.as_mut().and_then(|eff| {
        eff.set_mon_fill(output, applied);
        eff.monitor_apply_details(output)
    });
    if let Some((kind, current, we_id, mute, volume)) = target {
        let params = match kind {
            crate::domain::library::catalog::WallpaperKind::Static if !current.is_empty() => {
                Some(json!({
                    "type": wall_proto::kind::STATIC, "path": current, "output": output,
                    "notify": false
                }))
            }
            crate::domain::library::catalog::WallpaperKind::Video if !current.is_empty() => {
                Some(json!({
                    "type": wall_proto::kind::VIDEO, "path": current, "output": output,
                    "notify": false, "mute": mute, "volume": volume
                }))
            }
            crate::domain::library::catalog::WallpaperKind::We if !we_id.is_empty() => {
                Some(json!({
                    "type": wall_proto::kind::WE, "we_id": we_id, "output": output,
                    "notify": false, "mute": mute, "volume": volume
                }))
            }
            _ => None,
        };
        if let Some(params) = params {
            app.daemon.client.call("wall.apply", params);
        }
    }
    app.call_tracked("wall.outputs", json!({}), Pending::Outputs);
    Task::none()
}

pub(super) fn audio_mon_mute(app: &mut App, output: &str, mute: bool) -> Task<Message> {
    let targets = audio_control_targets(app, output);
    if let Some(panel) = app.panels.audio.as_mut() {
        for target in &targets {
            panel.set_mon_mute(target, mute);
        }
    }
    if let Some(eff) = app.panels.effects.as_mut() {
        for target in &targets {
            eff.set_mon_mute(target, mute);
        }
    }
    app.panels.audio_playing = app
        .panels
        .audio
        .as_ref()
        .is_some_and(crate::frontend::audio_panel::AudioPanel::any_playing)
        || app.panels.effects.as_ref().is_some_and(|effects| {
            effects.monitors().iter().any(|monitor| {
                matches!(
                    monitor.kind,
                    crate::domain::library::catalog::WallpaperKind::Video
                        | crate::domain::library::catalog::WallpaperKind::We
                ) && !monitor.mute
                    && monitor.volume > 0
            })
        });
    app.chrome.bar.cache.clear();
    if output == "*" {
        app.daemon.client.call("wall.set_audio", json!({ "mute": mute }));
    } else {
        app.daemon.client.call("wall.set_audio", json!({ "mute": mute, "outputs": &targets }));
    }
    let pend = if app.panels.effects.is_some() { Pending::Outputs } else { Pending::AudioOutputs };
    app.call_tracked("wall.outputs", json!({}), pend);
    Task::none()
}

pub(super) fn audio_mon_volume(app: &mut App, output: &str, vol: u32) -> Task<Message> {
    let targets = audio_control_targets(app, output);
    if let Some(panel) = app.panels.audio.as_mut() {
        for target in &targets {
            panel.set_mon_volume(target, vol);
            panel.set_mon_mute(target, vol == 0);
        }
    }
    if let Some(eff) = app.panels.effects.as_mut() {
        for target in &targets {
            eff.set_mon_volume(target, vol);
            eff.set_mon_mute(target, vol == 0);
        }
    }
    app.panels.audio_playing = app
        .panels
        .audio
        .as_ref()
        .is_some_and(crate::frontend::audio_panel::AudioPanel::any_playing)
        || app.panels.effects.as_ref().is_some_and(|effects| {
            effects.monitors().iter().any(|monitor| {
                matches!(
                    monitor.kind,
                    crate::domain::library::catalog::WallpaperKind::Video
                        | crate::domain::library::catalog::WallpaperKind::We
                ) && !monitor.mute
                    && monitor.volume > 0
            })
        });
    app.chrome.bar.cache.clear();
    Task::none()
}

pub(super) fn audio_mon_volume_release(app: &mut App, output: &str) -> Task<Message> {
    let targets = audio_control_targets(app, output);
    let vol = app
        .panels
        .audio
        .as_ref()
        .and_then(|panel| panel.mons.iter().find(|mon| mon.name == output).map(|mon| mon.volume))
        .or_else(|| app.panels.effects.as_ref().and_then(|eff| eff.mon_volume(output)));
    let Some(vol) = vol else {
        return Task::none();
    };
    let mute = vol == 0;
    if output == "*" {
        app.daemon.client.call("wall.set_audio", json!({ "volume": vol, "mute": mute }));
    } else {
        app.daemon
            .client
            .call("wall.set_audio", json!({ "volume": vol, "mute": mute, "outputs": &targets }));
    }
    if let Some(panel) = app.panels.audio.as_mut() {
        for target in &targets {
            panel.set_mon_mute(target, mute);
        }
    }
    if let Some(eff) = app.panels.effects.as_mut() {
        for target in &targets {
            eff.set_mon_mute(target, mute);
        }
    }
    Task::none()
}

fn audio_control_targets(app: &App, output: &str) -> Vec<String> {
    if output == "*" {
        return vec![String::from("*")];
    }
    app.panels
        .effects
        .as_ref()
        .map(|effects| effects.audio_group_outputs(output))
        .or_else(|| app.panels.audio.as_ref().map(|panel| panel.audio_group_outputs(output)))
        .unwrap_or_else(|| vec![output.to_string()])
}

pub(super) fn toggle_effects(app: &mut App) -> Task<Message> {
    let taken = app.panels.effects.take();
    match taken {
        Some(eff) => discard_effect_previews(app, eff),
        None => open_effects_for_current(app),
    }
    Task::none()
}

pub(super) fn discard_effect_previews(app: &mut App, eff: crate::frontend::effects::Effects) {
    for path in eff.discardable_previews() {
        app.discard_effect_preview(&path);
    }
}

pub(super) fn open_effects_for_current(app: &mut App) {
    crate::app::helpers::open_effects(
        app,
        app.scene.current,
        crate::frontend::effects::EffectsMode::Studio,
    );
}

pub(super) fn effects_edit(
    app: &mut App,
    func: impl FnOnce(&mut crate::frontend::effects::Effects),
) {
    if let Some(eff) = app.panels.effects.as_mut() {
        func(eff);
    }
}

pub(super) fn monitor_apply_selected(app: &mut App) -> Task<Message> {
    let info = app
        .panels
        .effects
        .as_ref()
        .and_then(|eff| app.library_session.filtered.get(eff.card()))
        .and_then(|&si| app.library_session.library.catalog().items.get(si as usize))
        .map(|it| (it.kind, it.path.clone(), it.we_id.clone()));
    let targets: Vec<String> = app
        .panels
        .effects
        .as_ref()
        .map(crate::frontend::effects::Effects::apply_targets)
        .unwrap_or_default();
    let (smute, svol) = app
        .panels
        .effects
        .as_ref()
        .map_or((true, 0), crate::frontend::effects::Effects::source_audio);
    if let Some((kind, path, we_id)) = info {
        info!("monitor apply: kind={} targets={targets:?}", kind.as_str());
        for (idx, output) in targets.iter().enumerate() {
            let notify = idx == 0;
            let params = match kind.as_str() {
                wall_proto::kind::WE => {
                    json!({"type": wall_proto::kind::WE, "we_id": we_id, "output": output, "notify": notify, "mute": smute, "volume": svol, "override_locks": true})
                }
                wall_proto::kind::VIDEO => {
                    json!({"type": wall_proto::kind::VIDEO, "path": path, "output": output, "notify": notify, "mute": smute, "volume": svol, "override_locks": true})
                }
                _ => {
                    json!({"type": wall_proto::kind::STATIC, "path": path, "output": output, "notify": notify, "override_locks": true})
                }
            };
            info!("  -> wall.apply output={output} type={}", kind.as_str());
            app.daemon.client.call("wall.apply", params);
        }
    } else {
        info!("monitor apply: no card selected, targets={targets:?}");
    }
    if !targets.is_empty() {
        app.panels.effects = None;
    }
    app.retick();
    Task::none()
}

pub(super) fn effects_shader_live(app: &App) -> bool {
    app.panels.effects.as_ref().is_some_and(crate::frontend::effects::Effects::shader_live)
}

pub(super) fn effects_nav(app: &mut App, dx: i32, dy: i32) -> Option<Task<Message>> {
    let eff = app.panels.effects.as_ref()?;
    if eff.mode() != crate::frontend::effects::EffectsMode::Studio || !eff.has_effects_page() {
        return None;
    }
    if dy != 0 {
        if let Some(eff) = app.panels.effects.as_mut() {
            eff.nav_move(dy);
        }
        app.retick();
        return Some(effects_scroll_task(app));
    }
    let msg = app.panels.effects.as_ref().and_then(|eff| eff.nav_horizontal(dx));
    if let Some(message) = msg {
        let select = super::update_inner(app, Message::Effects(message));
        return Some(Task::batch([select, effects_scroll_task(app)]));
    }
    app.retick();
    Some(effects_scroll_task(app))
}

fn effects_scroll_task(app: &App) -> Task<Message> {
    match app.panels.effects.as_ref().and_then(crate::frontend::effects::Effects::nav_scroll) {
        Some((key, frac)) => iced::widget::operation::snap_to(
            crate::frontend::ui::pane_id(key),
            iced::widget::scrollable::RelativeOffset { x: Some(frac), y: None },
        ),
        None => Task::none(),
    }
}

pub(super) fn effects_request_preview(app: &mut App) {
    match app.panels.effects.as_mut() {
        Some(eff) if eff.is_busy() => {
            eff.mark_preview_queued();
            return;
        }
        Some(_) => {}
        None => return,
    }
    effects_do_preview(app);
}

pub(crate) fn effects_do_preview(app: &mut App) {
    let (input, effects) = match app.panels.effects.as_ref() {
        Some(eff) if !eff.source_path().is_empty() => (
            eff.source_path().to_string(),
            crate::infrastructure::effects::encode_steps(&eff.preview_effects()),
        ),
        _ => return,
    };
    if effects.as_array().is_none_or(Vec::is_empty) {
        return;
    }
    if let Some(eff) = app.panels.effects.as_mut() {
        eff.begin_request();
    }
    let cache_key = serde_json::to_string(&effects).unwrap_or_default();
    app.call_tracked(
        "effects.preview",
        json!({ "input": input, "effects": effects }),
        Pending::EffectsPreview { source: input, cache_key },
    );
}

pub(super) fn effects_do_commit(app: &mut App) {
    let (input, effects, preview) = match app.panels.effects.as_ref() {
        Some(eff) if !eff.is_busy() && eff.has_saved_effects() && !eff.source_path().is_empty() => {
            (
                eff.source_path().to_string(),
                crate::infrastructure::effects::encode_steps(eff.saved_effects()),
                eff.preview_path().unwrap_or_default().to_string(),
            )
        }
        _ => return,
    };
    if let Some(eff) = app.panels.effects.as_mut() {
        eff.begin_request();
    }
    app.call_tracked(
        "effects.commit",
        json!({ "input": input, "effects": effects, "preview": preview }),
        Pending::EffectsCommit { apply: true },
    );
}
