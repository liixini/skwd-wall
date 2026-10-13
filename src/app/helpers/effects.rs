#[allow(clippy::wildcard_imports)]
use super::super::*;

pub(crate) fn open_effects(
    app: &mut App,
    index: usize,
    mode: crate::frontend::effects::EffectsMode,
) {
    open_effects_inner(app, index, mode, true);
}

pub(crate) fn open_effects_without_preview(
    app: &mut App,
    index: usize,
    mode: crate::frontend::effects::EffectsMode,
) {
    open_effects_inner(app, index, mode, false);
}

fn open_effects_inner(
    app: &mut App,
    index: usize,
    mode: crate::frontend::effects::EffectsMode,
    request_preview: bool,
) {
    let item = app.library_session.filtered.get(index).and_then(|&source_index| {
        app.library_session.library.catalog().items.get(source_index as usize)
    });
    let source = item.map(|item| item.path.clone()).unwrap_or_default();
    let source_thumbnail =
        item.map(|item| item.thumb.clone()).filter(|thumbnail| !thumbnail.is_empty());
    let source_kind = item.map(|item| item.kind).unwrap_or_default();
    let source_key = item.map(|item| item.key.clone()).unwrap_or_default();
    if app.detail_open() {
        app.scene.close_flip();
    }
    app.chrome.pane_scrolls.remove("fx.settings");
    app.chrome.pane_scrolls.remove("fx.monitors");
    let mut effects = crate::frontend::effects::Effects::new(
        app.daemon.effect_definitions.clone(),
        source,
        source_thumbnail,
        index,
        source_kind,
        app.config.wallpaper_mute(),
        app.config.wallpaper_volume(),
        source_key,
    );
    effects.set_motion_profile(app.motion_profile());
    effects.set_mode(mode);
    effects.set_chrome(crate::frontend::effects::dark_chrome(&app.theme.palette));
    let preview_source = effects.display_source();
    if !preview_source.is_empty()
        && let Some((rgba, width, height)) = crate::infrastructure::preview::decode_effect_source(
            &preview_source,
            wall_proto::EFFECT_PREVIEW_MAX_EDGE,
        )
    {
        effects.set_source_texture(rgba, width, height);
    }
    if !effects.source_path().is_empty() && effects.source_path() != preview_source {
        let source = effects.source_path().to_string();
        crate::infrastructure::preview::spawn_effect_source_decode(
            index,
            source,
            wall_proto::EFFECT_PREVIEW_MAX_EDGE,
            app.runtime_state.wake_tx.clone(),
        );
    }
    app.panels.effects = Some(effects);
    app.call_tracked("effects.list", serde_json::json!({}), Pending::EffectThemes);
    app.call_tracked("wall.outputs", serde_json::json!({}), Pending::Outputs);
    if mode == crate::frontend::effects::EffectsMode::Displays {
        request_activities(app);
    }
    if request_preview
        && mode == crate::frontend::effects::EffectsMode::Studio
        && app.panels.effects.as_ref().is_some_and(|effects| {
            effects.has_effects_page() && effects.preview_path().is_none() && !effects.shader_live()
        })
    {
        crate::app::update::effects_do_preview(app);
    }
    app.retick();
}

pub(crate) fn request_activities(app: &mut App) {
    let Some(effects) = app
        .panels
        .effects
        .as_mut()
        .filter(|effects| effects.mode() == crate::frontend::effects::EffectsMode::Displays)
    else {
        return;
    };
    effects.activity.ready = false;
    effects.activity.output_request = 0;
    let id = app.call_tracked(
        wall_proto::rpc::WALL_ACTIVITIES,
        serde_json::json!({}),
        Pending::Activities,
    );
    if let Some(effects) = app.panels.effects.as_mut() {
        effects.activity.list_request = id;
    }
    app.retick();
}
