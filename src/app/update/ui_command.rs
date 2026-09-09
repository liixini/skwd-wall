use iced::Task;
use serde_json::json;

#[allow(clippy::wildcard_imports)]
use super::*;

use super::settings::settings_toggle;
use crate::frontend::scene::layout::Mode;

pub(crate) fn ui_state_json(app: &App) -> String {
    let mode = app.scene.mode.as_key();
    let (vis_lo, vis_hi) = app.scene.visible_range();
    let (min_card_area, max_card_area) = app
        .scene
        .render
        .hits
        .iter()
        .map(|hit| hit.hw * hit.hh * 4.0)
        .fold((f32::MAX, 0.0_f32), |(lo, hi), area| (lo.min(area), hi.max(area)));
    let min_card_area = if min_card_area.is_finite() { min_card_area } else { 0.0 };
    let selection = selected_key(app);
    let edited_tags = if app.tags.editing {
        app.tags.card_locked.clone()
    } else {
        app.scene
            .flipped()
            .and_then(|filtered_index| app.library_session.filtered.get(filtered_index))
            .and_then(|&source_index| {
                app.library_session.library.catalog().items.get(source_index as usize)
            })
            .and_then(|item| app.library_session.library.catalog().tags.get(&item.key))
            .cloned()
            .unwrap_or_default()
    };
    let card_back = app.scene.render.back.as_ref().map(|panel| {
        let layout = crate::frontend::ui::back_layout(panel);
        let action_count =
            3 + usize::from(layout.overview.is_some()) + usize::from(layout.effects.is_some());
        json!({
            "presentation": if panel.embedded { "embedded" } else { "external" },
            "card": { "x": layout.card.0, "y": layout.card.1,
                "w": layout.card.2, "h": layout.card.3 },
            "art": { "x": layout.card.0, "y": layout.card.1,
                "w": layout.card.2, "h": layout.card.3 },
            "sheet": { "x": layout.sheet.0, "y": layout.sheet.1,
                "w": layout.sheet.2, "h": layout.sheet.3 },
            "action_deck": { "x": layout.action_deck.0, "y": layout.action_deck.1,
                "w": layout.action_deck.2, "h": layout.action_deck.3 },
            "actions": action_count,
            "tags": panel.tags,
            "tag_chips": layout.tags.len(),
            "tag_hidden": layout.tag_overflow.map_or(0, |overflow| overflow.4),
            "tag_input_open": panel.add_open > 0.85,
        })
    });
    let display_state = app.panels.effects.as_ref().map(|effects| {
        let monitors: Vec<&str> =
            effects.monitors().iter().map(|monitor| monitor.name.as_str()).collect();
        let selected: Vec<&str> =
            monitors.iter().copied().filter(|name| effects.output_selected(name)).collect();
        json!({
            "mode": if effects.mode() == crate::frontend::effects::EffectsMode::Displays {
                "displays"
            } else {
                "studio"
            },
            "monitors": monitors,
            "selected": selected,
            "targets": effects.apply_targets(),
        })
    });
    let playlist_state = app.panels.playlists.as_ref().map(|playlists| {
        json!({
            "selected": playlists.selected,
            "lists": playlists.lists.len(),
            "members": playlists.members.len(),
            "members_for": playlists.members_for,
        })
    });
    json!({
        "demo_protocol": 15,
        "mode": mode,
        "count": app.library_session.filtered.len(),
        "current": app.scene.current,
        "selection": selection,
        "camera": { "x": app.scene.camera_pos(), "target": app.scene.camera_target() },
        "cell": { "w": app.scene.gp.cell_w(), "h": app.scene.gp.cell_h() },
        "viewport": { "w": app.scene.viewport.0, "h": app.scene.viewport.1 },
        "filter_bar": {
            "orientation": app.config.filter_bar_orientation(),
            "rendered_orientation": if app.chrome.filter_bar_vertical {
                "vertical"
            } else {
                "horizontal"
            },
            "fade": app.chrome.filter_bar_fade(),
            "animating": app.chrome.filter_bar_animating(),
            "wall_center": {
                "x": app.scene.wall_composition_center().0,
                "y": app.scene.wall_composition_center().1,
            },
        },
        "visible": { "lo": vis_lo, "hi": vis_hi },
        "geometry": {
            "wall_layout": app.scene.gp.layout.as_key(),
            "hex_curve": app.scene.hp.curve.as_key(),
            "hex_shape": app.scene.hp.shape.as_key(),
            "card_area": { "min": min_card_area, "max": max_card_area },
            "wall_field": {
                "wave": app.scene.gp.flow_wave,
                "frequency": app.scene.gp.flow_frequency,
                "scatter": app.scene.gp.scatter,
                "scale_variance": app.scene.gp.scale_variance,
                "cylinder_bend": app.scene.gp.cylinder_bend,
                "cylinder_radius": app.scene.gp.cylinder_radius,
            },
            "hex_deform": {
                "orbit": app.scene.hp.orbit,
                "orbit_radius": app.scene.hp.orbit_radius,
                "twist": app.scene.hp.twist,
                "scatter": app.scene.hp.scatter,
            },
            "wall_stage": {
                "x": app.scene.gp.stage.offset_x, "y": app.scene.gp.stage.offset_y,
                "scale": app.scene.gp.stage.scale, "rotation": app.scene.gp.stage.rotation,
                "perspective": app.scene.gp.stage.perspective,
                "shear_x": app.scene.gp.stage.shear_x, "shear_y": app.scene.gp.stage.shear_y,
                "depth_angle": app.scene.gp.stage.depth_angle,
            },
            "hex_stage": {
                "x": app.scene.hp.stage.offset_x, "y": app.scene.hp.stage.offset_y,
                "scale": app.scene.hp.stage.scale, "rotation": app.scene.hp.stage.rotation,
                "perspective": app.scene.hp.stage.perspective,
                "shear_x": app.scene.hp.stage.shear_x, "shear_y": app.scene.hp.stage.shear_y,
                "depth_angle": app.scene.hp.stage.depth_angle,
            },
        },
        "filter": {
            "tags": app.library_session.filters.tags,
            "kind": app.library_session.filters.kind,
            "resolution": app.library_session.filters.resolution,
            "query": app.tags.tag_search
        },
        "tag_editor": {
            "open": app.tags.editing,
            "drawer_open": app.tags.card_drawer_open,
            "input": app.tags.input,
            "tags": edited_tags,
        },
        "tag_organizer": {
            "cloud_open": app.tags.cloud_open,
            "bulk_mode": app.tags.mode,
            "selected": app.tags.select.len(),
            "pending": app.tags.mass_tags,
            "search_mode": if app.tags.search_mode == SearchMode::Tags { "tags" } else { "describe" },
        },
        "animating": app.scene.is_animating(),
        "layout_transition_active": app.scene.layout_transition_active(),
        "loop_active": app.preview_resources.render_loop_active,
        "filter_swap_active": app.scene.filter_swap_active(),
        "open_fade_settled": app.scene.open_fade_settled(),
        "capturing": app.menu_capturing(),
        "overlay": app.topmost_overlay().map(|ov| format!("{ov:?}")),
        "shown": app.runtime_state.overlay.is_some(),
        "demo_active": app.runtime_state.demo.is_some(),
        "card_back": card_back,
        "playlists": playlist_state,
        "displays": display_state,
        "semantic": {
            "query": app.tags.semantic.search,
            "pending": app.tags.semantic.pending,
            "resolved": app.tags.semantic.resolved,
            "error": app.tags.semantic.error,
            "ranked": app.tags.semantic.ranked.len(),
            "query_ms": app.tags.semantic.query_ms,
            "search_ms": app.tags.semantic.search_ms,
            "process_active": app.runtime_state.semantic.is_some(),
        },
        "settings": {
            "open": app.panels.settings.open,
            "tab": app.panels.settings.tab,
            "section": app.panels.settings.section,
            "control_page": app.panels.settings.control_page,
            "focused_control": app.panels.settings.focused_control,
            "focus": format!("{:?}", app.panels.settings.focus),
            "search_open": app.panels.settings.search_open,
            "search_query": app.panels.settings.search_query,
            "search_results": app.panels.settings.search_results.len(),
        },
    })
    .to_string()
}

pub(super) fn run_ui_command(app: &mut App, cmd: &str) -> Task<Message> {
    let verb = cmd.split_whitespace().next().unwrap_or("");
    let arg = cmd[verb.len()..].trim().to_string();
    match verb {
        "toggle" => crate::app::warm::toggle(app),
        "show" => Task::none(),
        "hide" => crate::app::warm::exit_picker(app),
        "demo" => match arg.as_str() {
            "begin" => demo_begin(app),
            "end" => demo_end(app),
            other => {
                log::warn!("ui command: unknown demo action '{other}'");
                Task::none()
            }
        },
        "batch-stage" if app.runtime_state.demo.is_some() => ui_demo_batch_stage(app, &arg),
        "batch-commit" if app.runtime_state.demo.is_some() => ui_demo_batch_commit(app, &arg),
        "audio-demo" if app.runtime_state.demo.is_some() => ui_demo_audio(app, &arg),
        "effect-demo" if app.runtime_state.demo.is_some() => ui_demo_effect(app, &arg),
        "picker-demo" if app.runtime_state.demo.is_some() => ui_demo_picker(app, &arg),
        "tune" => {
            let mut it = arg.splitn(2, char::is_whitespace);
            let path = it.next().unwrap_or("");
            let val = it.next().unwrap_or("").trim();
            ui_tune(app, path, val)
        }
        "motion" => ui_motion(app, &arg),
        "semantic-warm" if app.runtime_state.demo.is_some() => {
            app.prewarm_semantic_search();
            Task::none()
        }
        "badges" if app.runtime_state.demo.is_some() => ui_demo_badges(app, &arg),
        "recolour" | "recolor" => ui_recolour(app, &arg),
        "sort" if crate::frontend::ui::SORTS.iter().any(|(key, _)| *key == arg.as_str()) => {
            super::update_inner(app, Message::SetSort(arg))
        }
        "colour" | "color" => ui_colour_filter(app, &arg),
        "apply-source" if app.runtime_state.demo.is_some() && !arg.is_empty() => {
            if let Some(session) = app.runtime_state.demo.as_mut() {
                session.apply_source = Some(arg);
            }
            Task::none()
        }
        "apply" => ui_demo_apply(app, &arg),
        "restore-overrides" if app.runtime_state.demo.is_some() => {
            restore_demo_overrides(app);
            Task::none()
        }
        "override-next" if app.runtime_state.demo.is_some() => {
            if let Some(session) = app.runtime_state.demo.as_mut() {
                session.override_next_apply = true;
            }
            Task::none()
        }
        "stage-blur" => ui_demo_stage_blur(app, &arg),
        "mode" => {
            let Some(mode) = Mode::try_from_key(&arg) else {
                log::warn!("ui command: unknown picker mode '{arg}'");
                return Task::none();
            };
            super::settings::set_view_mode(app, mode.as_key())
        }
        "tab" => {
            let tab = crate::frontend::settings::canonical_category(arg);
            if !crate::frontend::settings::visible_tabs(&app.config)
                .iter()
                .any(|(key, _)| *key == tab)
            {
                return Task::none();
            }
            if app.panels.settings.open {
                return super::update_inner(app, Message::SetSettingsTab(tab));
            }
            let open = super::update_inner(app, Message::ToggleSettings);
            Task::batch([open, super::update_inner(app, Message::SetSettingsTab(tab))])
        }
        "section" => match arg.parse::<usize>() {
            Ok(section) if app.panels.settings.open => super::update_inner(
                app,
                Message::Settings(crate::frontend::settings::SettingsMsg::SelectSection(section)),
            ),
            _ => {
                log::warn!("ui command: 'section' needs an open settings panel and numeric index");
                Task::none()
            }
        },
        "filter" if arg.is_empty() => super::update_inner(app, Message::ClearTags),
        "filter" => {
            app.library_session.filters.tags = arg
                .split_whitespace()
                .map(str::to_lowercase)
                .filter(|tag| !tag.is_empty())
                .collect();
            app.refilter_from_start();
            app.chrome.bar.cache.clear();
            app.retick();
            Task::none()
        }
        "tag-edit" => ui_tag_edit(app, &arg),
        "tag-mode" if app.runtime_state.demo.is_some() => ui_demo_tag_mode(app, &arg),
        "tag-select" if app.runtime_state.demo.is_some() => ui_demo_tag_select(app, &arg),
        "tag-bulk" if app.runtime_state.demo.is_some() => ui_demo_tag_bulk(app, &arg),
        "tag-query" if app.runtime_state.demo.is_some() => ui_demo_tag_query(app, &arg),
        "tag-match" if app.runtime_state.demo.is_some() => ui_demo_tag_match(app, &arg),
        "search" if !arg.is_empty() => {
            let mut tasks = Vec::new();
            if !app.tags.cloud_open {
                tasks.push(super::update_inner(app, Message::OpenTagCloud));
            }
            tasks.push(super::update_inner(
                app,
                Message::Tag(crate::frontend::tagcloud::TagMsg::SearchMode(SearchMode::Describe)),
            ));
            tasks.push(super::update_inner(
                app,
                Message::Tag(crate::frontend::tagcloud::TagMsg::QueryInput(arg)),
            ));
            Task::batch(tasks)
        }
        "settings-search" => {
            let mut tasks = Vec::new();
            if !app.panels.settings.open {
                tasks.push(super::update_inner(app, Message::ToggleSettings));
            }
            tasks.push(super::update_inner(
                app,
                Message::Settings(crate::frontend::settings::SettingsMsg::SearchOpen),
            ));
            if !arg.is_empty() {
                tasks.push(super::update_inner(
                    app,
                    Message::Settings(crate::frontend::settings::SettingsMsg::SearchInput(arg)),
                ));
            }
            Task::batch(tasks)
        }
        "settings-result" => {
            let index = arg.parse::<usize>().unwrap_or(0);
            let Some(result) = app.panels.settings.search_results.get(index).cloned() else {
                log::warn!("ui command: settings result {index} is unavailable");
                return Task::none();
            };
            super::update_inner(
                app,
                Message::Settings(crate::frontend::settings::SettingsMsg::OpenSearchResult(
                    result.tab,
                    result.section,
                    result.row,
                )),
            )
        }
        "clear" => {
            app.tags.semantic.search.clear();
            app.clear_semantic_search();
            super::update_inner(app, Message::ClearTags)
        }
        "select" if !arg.is_empty() => select_key(app, &arg),
        "flip" => super::update_inner(app, Message::KeyFlip),
        "bar" => {
            match arg.as_str() {
                "show" => app.chrome.filter_bar_visible = true,
                "hide" => app.chrome.filter_bar_visible = false,
                "toggle" => app.chrome.filter_bar_visible = !app.chrome.filter_bar_visible,
                other => log::warn!("ui command: unknown bar action '{other}'"),
            }
            app.chrome.bar.cache.clear();
            app.retick();
            Task::none()
        }
        "kind" => {
            let kind = match arg.as_str() {
                "any" | "" => String::new(),
                "static" | "video" | "we" => arg,
                other => {
                    log::warn!("ui command: unknown wallpaper kind '{other}'");
                    return Task::none();
                }
            };
            super::update_inner(app, Message::SetTypeFilter(kind))
        }
        "orient" => {
            let orient = match arg.as_str() {
                "any" | "" => String::new(),
                "wide" | "landscape" => String::from("landscape"),
                "tall" | "portrait" => String::from("portrait"),
                other => {
                    log::warn!("ui command: unknown orientation '{other}'");
                    return Task::none();
                }
            };
            super::update_inner(app, Message::SetOrient(orient))
        }
        "resolution" => {
            let resolution = if matches!(arg.as_str(), "any" | "") { String::new() } else { arg };
            super::update_inner(app, Message::SetResolution(resolution))
        }
        "open" => ui_open(app, &arg),
        "display" => ui_display(app, &arg),
        "playlist" => arg.parse::<i64>().map_or_else(
            |_| {
                log::warn!("ui command: invalid playlist id '{arg}'");
                Task::none()
            },
            |id| {
                super::update_inner(app, Message::Pl(crate::frontend::playlists::PlMsg::Select(id)))
            },
        ),
        "playlist-demo" => ui_demo_playlist(app, &arg),
        "schedule-demo" => ui_demo_schedule(app, &arg),
        "scroll-demo" => ui_demo_scroll(app, &arg),
        "preset" => ui_preset(app, &arg),
        "dismiss" => {
            app.close_topmost_overlay();
            Task::none()
        }
        "close" => super::update_inner(app, Message::Exit),
        "wheel" => arg.parse::<f32>().ok().filter(|amount| amount.is_finite()).map_or_else(
            || {
                log::warn!("ui command: invalid wheel amount '{arg}'");
                Task::none()
            },
            |amount| super::update_inner(app, Message::Wheel(amount.clamp(-12.0, 12.0))),
        ),
        "nav" => match arg.as_str() {
            "left" => super::update_inner(app, Message::KeyPrev),
            "right" => super::update_inner(app, Message::KeyNext),
            "up" => super::update_inner(app, Message::KeyUp),
            "down" => super::update_inner(app, Message::KeyDown),
            other => {
                log::warn!("ui command: unknown direction '{other}'");
                Task::none()
            }
        },
        "pick" => super::update_inner(app, Message::ApplyCurrent),
        "set" => {
            let mut it = arg.splitn(2, char::is_whitespace);
            let path = it.next().unwrap_or("").to_string();
            let val = it.next().unwrap_or("").trim().to_string();
            ui_set(app, &path, &val)
        }
        other => {
            log::warn!("ui command: unknown '{other}'");
            Task::none()
        }
    }
}

fn ui_preset(app: &mut App, arg: &str) -> Task<Message> {
    let mut parts = arg.splitn(2, char::is_whitespace);
    let action = parts.next().unwrap_or("");
    let name = parts.next().unwrap_or("").trim();
    if action != "save" || name.is_empty() {
        log::warn!("ui command: usage 'preset save <name>'");
        return Task::none();
    }
    let mode = app.config.selector_mode();
    app.config.save_selector_preset(&mode, name);
    app.panels
        .settings
        .inputs
        .insert(crate::frontend::settings::PRESET_NAME_KEY.to_string(), name.to_string());
    app.invalidate_settings();
    app.retick();
    Task::none()
}

fn ui_tag_edit(app: &mut App, arg: &str) -> Task<Message> {
    let mut parts = arg.splitn(2, char::is_whitespace);
    match parts.next().unwrap_or("") {
        "open" => {
            let tags = app
                .scene
                .flipped()
                .and_then(|filtered_index| app.library_session.filtered.get(filtered_index))
                .and_then(|&source_index| {
                    app.library_session.library.catalog().items.get(source_index as usize)
                })
                .and_then(|item| app.library_session.library.catalog().tags.get(&item.key))
                .cloned()
                .unwrap_or_default();
            if app.scene.flipped().is_some() {
                app.tags.editing = true;
                begin_card_tag_edit(app, &tags);
                app.scene.set_tag_editing(true);
                app.retick();
            }
            Task::none()
        }
        "input" if app.tags.editing => super::update_inner(
            app,
            Message::Tag(crate::frontend::tagcloud::TagMsg::InputChanged(
                parts.next().unwrap_or("").to_string(),
            )),
        ),
        "submit" if app.tags.editing => {
            super::update_inner(app, Message::Tag(crate::frontend::tagcloud::TagMsg::Submit))
        }
        "drawer" => super::update_inner(
            app,
            Message::Tag(crate::frontend::tagcloud::TagMsg::ToggleCardDrawer),
        ),
        "remove" => {
            let index = parts.next().unwrap_or("").trim().parse::<usize>();
            if let Ok(index) = index {
                super::update_inner(
                    app,
                    Message::Tag(crate::frontend::tagcloud::TagMsg::Remove(index)),
                )
            } else {
                log::warn!("ui command: tag-edit remove requires a numeric index");
                Task::none()
            }
        }
        other => {
            log::warn!("ui command: invalid tag-edit action '{other}'");
            Task::none()
        }
    }
}

fn ui_demo_tag_mode(app: &mut App, arg: &str) -> Task<Message> {
    let desired = match arg {
        "begin" | "on" | "open" => true,
        "end" | "off" | "close" => false,
        _ => {
            log::warn!("ui command: usage 'tag-mode <begin|end>'");
            return Task::none();
        }
    };
    if app.tags.mode == desired {
        return Task::none();
    }
    super::update_inner(app, Message::ToggleTagMode)
}

fn ui_demo_tag_select(app: &mut App, arg: &str) -> Task<Message> {
    if !app.tags.mode {
        log::warn!("ui command: 'tag-select' requires tag mode");
        return Task::none();
    }
    let index = if arg == "current" { Some(app.scene.current) } else { arg.parse::<usize>().ok() };
    let Some(index) = index.filter(|index| *index < app.library_session.filtered.len()) else {
        log::warn!("ui command: invalid tag selection '{arg}'");
        return Task::none();
    };
    super::tags::tag_select_click(app, Some(index))
}

fn ui_demo_tag_bulk(app: &mut App, arg: &str) -> Task<Message> {
    let mut parts = arg.splitn(2, char::is_whitespace);
    match (parts.next().unwrap_or(""), parts.next().unwrap_or("").trim()) {
        ("add", tag) if app.tags.mode && !tag.is_empty() => {
            super::update_inner(app, Message::MassTagAdd(tag.to_string()))
        }
        _ => {
            log::warn!("ui command: usage 'tag-bulk add <tag>' while tag mode is active");
            Task::none()
        }
    }
}

fn ui_demo_tag_query(app: &mut App, arg: &str) -> Task<Message> {
    if arg.is_empty() {
        log::warn!("ui command: usage 'tag-query <tags>'");
        return Task::none();
    }
    let mut tasks = Vec::new();
    if !app.tags.cloud_open {
        tasks.push(super::update_inner(app, Message::OpenTagCloud));
    }
    tasks.push(super::update_inner(
        app,
        Message::Tag(crate::frontend::tagcloud::TagMsg::SearchMode(SearchMode::Tags)),
    ));
    tasks.push(super::update_inner(
        app,
        Message::Tag(crate::frontend::tagcloud::TagMsg::QueryInput(format!("{arg} "))),
    ));
    Task::batch(tasks)
}

fn ui_demo_tag_match(app: &mut App, arg: &str) -> Task<Message> {
    let match_any = match arg {
        "any" => true,
        "all" => false,
        _ => {
            log::warn!("ui command: usage 'tag-match <all|any>'");
            return Task::none();
        }
    };
    super::update_inner(app, Message::Tag(crate::frontend::tagcloud::TagMsg::MatchMode(match_any)))
}

fn ui_open(app: &mut App, arg: &str) -> Task<Message> {
    match arg.split_whitespace().next().unwrap_or(arg) {
        "settings" => super::update_inner(app, Message::ToggleSettings),
        "effects" => {
            if app.panels.settings.open {
                app.close_settings();
            }
            let effect = arg["effects".len()..].trim().to_string();
            if effect.is_empty() {
                super::update_inner(app, Message::ToggleEffects)
            } else if app.panels.effects.is_some() {
                super::update_inner(
                    app,
                    Message::Effects(crate::frontend::effects::EffectsMsg::Select(effect)),
                )
            } else {
                let open = super::update_inner(app, Message::ToggleEffects);
                Task::batch([
                    open,
                    super::update_inner(
                        app,
                        Message::Effects(crate::frontend::effects::EffectsMsg::Select(effect)),
                    ),
                ])
            }
        }
        "effects-cached" if app.runtime_state.demo.is_some() => {
            if app.panels.settings.open {
                app.close_settings();
            }
            let effect = arg["effects-cached".len()..].trim();
            crate::app::helpers::open_effects_without_preview(
                app,
                app.scene.current,
                crate::frontend::effects::EffectsMode::Studio,
            );
            if !effect.is_empty()
                && let Some(effects) = app.panels.effects.as_mut()
            {
                effects.select_effect(effect.to_string());
            }
            app.retick();
            Task::none()
        }
        "displays" => {
            if let Some(effects) = app.panels.effects.as_mut() {
                effects.set_mode(crate::frontend::effects::EffectsMode::Displays);
            } else {
                crate::app::helpers::open_effects(
                    app,
                    app.scene.current,
                    crate::frontend::effects::EffectsMode::Displays,
                );
            }
            Task::none()
        }
        "mixer" if app.panels.audio.is_some() => Task::none(),
        "mixer" => super::update_inner(app, Message::ToggleAudioPanel),
        "downloads" => super::update_inner(app, Message::OpenSourceBrowser),
        "tags" => super::update_inner(app, Message::OpenTagCloud),
        "playlists" => {
            if app.panels.effects.is_some() {
                app.close_overlay(crate::app::overlay::Overlay::Effects);
            }
            if app.panels.settings.open {
                app.close_settings();
            }
            super::update_inner(app, Message::OpenPlaylists)
        }
        "schedule" => {
            crate::app::helpers::sched_open(app);
            Task::none()
        }
        "theme" => {
            crate::app::helpers::theme_designer_open(app);
            Task::none()
        }
        "theme-bar" => super::update_inner(app, Message::ToggleThemePanel),
        "theme-audition" => super::update_inner(app, Message::OpenThemeAudition),
        other => {
            log::warn!("ui command: unknown panel '{other}'");
            Task::none()
        }
    }
}

fn ui_demo_batch_stage(app: &mut App, raw: &str) -> Task<Message> {
    let mut parts = raw.splitn(2, char::is_whitespace);
    let id = parts.next().unwrap_or("");
    let encoded = parts.next().unwrap_or("").trim();
    let valid_id = !id.is_empty()
        && id.len() <= 16
        && id.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-');
    let Ok(command) = serde_json::from_str::<String>(encoded) else {
        log::warn!("ui command: batch-stage needs an id and one JSON string");
        return Task::none();
    };
    let verb = command.split_whitespace().next().unwrap_or("");
    if !valid_id || verb.is_empty() || matches!(verb, "batch-stage" | "batch-commit" | "demo") {
        log::warn!("ui command: invalid batch-stage command");
        return Task::none();
    }
    let Some(session) = app.runtime_state.demo.as_mut() else {
        return Task::none();
    };
    if session.batch_id.as_deref() != Some(id) {
        session.batch_id = Some(id.to_string());
        session.batch_commands.clear();
    }
    if session.batch_commands.len() >= 64
        || session.batch_commands.iter().map(String::len).sum::<usize>() + command.len() > 16_384
    {
        log::warn!("ui command: staged batch exceeds its bounded capacity");
        session.batch_id = None;
        session.batch_commands.clear();
        return Task::none();
    }
    session.batch_commands.push(command);
    Task::none()
}

fn ui_demo_batch_commit(app: &mut App, id: &str) -> Task<Message> {
    let Some(session) = app.runtime_state.demo.as_mut() else {
        return Task::none();
    };
    if session.batch_id.as_deref() != Some(id) || session.batch_commands.is_empty() {
        log::warn!("ui command: batch-commit does not match a staged batch");
        return Task::none();
    }
    session.batch_id = None;
    let commands = std::mem::take(&mut session.batch_commands);
    let mut remaining = Vec::with_capacity(commands.len());
    let mut selector_layout_changed = false;
    for command in commands {
        let verb = command.split_whitespace().next().unwrap_or("");
        let arg = command[verb.len()..].trim();
        if verb == "mode"
            && let Some(mode) = Mode::try_from_key(arg)
        {
            app.config.ensure_selector_enabled();
            app.config.save_key(skwd_config::keys::selector::DISPLAY_MODE, json!(mode.as_key()));
            selector_layout_changed = true;
            continue;
        }
        if verb == "tune" {
            let mut parts = arg.splitn(2, char::is_whitespace);
            let path = parts.next().unwrap_or("");
            let value = parts.next().unwrap_or("").trim();
            if path.starts_with("components.wallpaperSelector.")
                && stage_demo_tune(app, path, value)
            {
                selector_layout_changed = true;
                continue;
            }
        }
        remaining.push(command);
    }
    if selector_layout_changed {
        app.apply_layout();
        app.init_settings_inputs();
        app.invalidate_settings();
    }
    Task::batch(remaining.iter().map(|command| run_ui_command(app, command)))
}

fn ui_set(app: &mut App, path: &str, val: &str) -> Task<Message> {
    if path.is_empty() || val.is_empty() {
        log::warn!("ui command: usage 'set <config.path> <value>'");
        return Task::none();
    }
    if val == "true" || val == "false" {
        return settings_toggle(app, path, val == "true");
    }
    let parsed = val.parse::<f64>().map_or_else(|_| json!(val), |num| json!(num));
    super::settings_policy::save_value(app, path, &parsed);
    app.apply_layout();
    app.invalidate_settings();
    Task::none()
}

fn ui_demo_playlist(app: &mut App, raw: &str) -> Task<Message> {
    if app.runtime_state.demo.is_none() {
        log::warn!("ui command: 'playlist-demo' requires an active demo session");
        return Task::none();
    }
    let mut words = raw.splitn(2, char::is_whitespace);
    let action = words.next().unwrap_or("");
    let value = words.next().unwrap_or("").trim();

    if action == "reset" {
        let Some(playlists) = app.panels.playlists.as_mut() else {
            log::warn!("ui command: 'playlist-demo reset' needs an open playlist panel");
            return Task::none();
        };
        let name = if value.is_empty() { crate::i18n::tr("playlists-demo-name") } else { value };
        playlists.demo = true;
        playlists.lists = vec![crate::contracts::playlists::Playlist {
            id: -1,
            name: name.to_string(),
            order: "sequential".into(),
            dwell: 5,
            ..Default::default()
        }];
        playlists.selected = Some(-1);
        playlists.members.clear();
        playlists.members_for = -1;
        playlists.assign.clear();
        playlists.sync_buffers();
        app.library_session.playlist_filter = None;
        app.refilter();
        app.retick();
        return Task::none();
    }

    let Some(is_demo) = app.panels.playlists.as_ref().map(|playlists| playlists.demo) else {
        log::warn!("ui command: 'playlist-demo' needs an open playlist panel");
        return Task::none();
    };
    if !is_demo {
        log::warn!("ui command: 'playlist-demo' needs 'playlist-demo reset' first");
        return Task::none();
    }

    match action {
        "add" => {
            let Some(wallpaper) = app
                .library_session
                .library
                .catalog()
                .items
                .iter()
                .find(|wallpaper| wallpaper.key == value)
                .cloned()
            else {
                log::warn!("ui command: demo playlist wallpaper is missing: '{value}'");
                return Task::none();
            };
            let Some(playlists) = app.panels.playlists.as_mut() else {
                return Task::none();
            };
            if playlists.members.iter().any(|member| member.key.as_deref() == Some(value)) {
                return Task::none();
            }
            playlists.members.push(crate::contracts::playlists::PlaylistMember {
                key: Some(wallpaper.key),
                kind: Some(wallpaper.kind.as_str().into()),
                preview: Some(wallpaper.preview),
                thumb: Some(wallpaper.thumb.clone()),
                thumb_sm: Some(wallpaper.thumb),
            });
            if let Some(row) = playlists.lists.first_mut() {
                row.count = playlists.members.len() as i64;
            }
        }
        "move" => {
            let mut args = value.split_whitespace();
            let key = args.next().unwrap_or("");
            let delta = args.next().and_then(|raw| raw.parse::<isize>().ok()).unwrap_or(0);
            let Some(playlists) = app.panels.playlists.as_mut() else {
                return Task::none();
            };
            if let Some(from) =
                playlists.members.iter().position(|member| member.key.as_deref() == Some(key))
            {
                let to = (from as isize + delta)
                    .clamp(0, playlists.members.len().saturating_sub(1) as isize)
                    as usize;
                if from != to {
                    let member = playlists.members.remove(from);
                    playlists.members.insert(to, member);
                }
            }
        }
        "order" if matches!(value, "shuffle" | "sequential") => {
            if let Some(row) =
                app.panels.playlists.as_mut().and_then(|playlists| playlists.lists.first_mut())
            {
                row.order = value.to_string();
            }
        }
        "dwell" => {
            if let Some(seconds) = value.parse::<i64>().ok().filter(|seconds| *seconds >= 5)
                && let Some(row) =
                    app.panels.playlists.as_mut().and_then(|playlists| playlists.lists.first_mut())
            {
                row.dwell = seconds;
            }
        }
        "activate" => {
            if let Some(playlists) = app.panels.playlists.as_mut() {
                playlists.assign = vec![("*".into(), -1)];
            }
        }
        other => log::warn!("ui command: unknown playlist-demo action '{other}'"),
    }
    if let Some(playlists) = app.panels.playlists.as_mut() {
        playlists.sync_buffers();
    }
    app.retick();
    Task::none()
}

fn demo_schedule_rows() -> Vec<crate::domain::schedule::RuleRow> {
    use crate::domain::schedule::{Block, ConditionNode, GroupOperator, RuleRow};

    vec![
        RuleRow {
            name: "Rain after dark".into(),
            enabled: true,
            condition: ConditionNode::group(
                GroupOperator::All,
                vec![
                    ConditionNode::predicate(Block::TimeCmp(">=".into(), "sunset".into())),
                    ConditionNode::predicate(Block::Weather(vec![
                        "rainy".into(),
                        "stormy".into(),
                        "foggy".into(),
                    ])),
                ],
            ),
            set: "Neon nights".into(),
            mode: "dark".into(),
        },
        RuleRow {
            name: "Slow weekend".into(),
            enabled: true,
            condition: ConditionNode::group(
                GroupOperator::All,
                vec![
                    ConditionNode::predicate(Block::Weekday(vec!["sat".into(), "sun".into()])),
                    ConditionNode::group(
                        GroupOperator::Any,
                        vec![
                            ConditionNode::predicate(Block::Weather(vec![
                                "clear".into(),
                                "sunny".into(),
                            ])),
                            ConditionNode::predicate(Block::TimeWindow(
                                "sunrise".into(),
                                "12:00".into(),
                            )),
                        ],
                    ),
                ],
            ),
            set: "Quiet landscapes".into(),
            mode: "light".into(),
        },
        RuleRow {
            name: "Daylight rotation".into(),
            enabled: true,
            condition: ConditionNode::group(
                GroupOperator::All,
                vec![ConditionNode::predicate(Block::TimeWindow(
                    "sunrise".into(),
                    "sunset".into(),
                ))],
            ),
            set: "random".into(),
            mode: String::new(),
        },
    ]
}

fn ui_demo_schedule(app: &mut App, raw: &str) -> Task<Message> {
    if app.runtime_state.demo.is_none() {
        log::warn!("ui command: 'schedule-demo' requires an active demo session");
        return Task::none();
    }
    let mut parts = raw.split_whitespace();
    match parts.next().unwrap_or("") {
        "open" => {
            close_all_overlays(app);
            let mut editor =
                crate::frontend::schedule_editor::ScheduleEditor::demo(demo_schedule_rows());
            editor.set_motion_profile(app.motion_profile());
            app.panels.schedule = Some(editor);
            app.retick();
            Task::none()
        }
        "select" => {
            let Some(row) = parts.next().and_then(|value| value.parse::<usize>().ok()) else {
                log::warn!("ui command: 'schedule-demo select' needs a rule index");
                return Task::none();
            };
            if let Some(editor) = app.panels.schedule.as_mut() {
                editor.toggle_rule_options(row);
                app.retick();
            }
            Task::none()
        }
        "edit" => {
            let Some(row) = parts.next().and_then(|value| value.parse::<usize>().ok()) else {
                log::warn!("ui command: 'schedule-demo edit' needs a rule index and node path");
                return Task::none();
            };
            let path = parts.map(str::parse::<usize>).collect::<Result<Vec<_>, _>>();
            let Ok(path) = path else {
                log::warn!("ui command: invalid 'schedule-demo edit' node path");
                return Task::none();
            };
            if let Some(editor) = app.panels.schedule.as_mut()
                && editor.node(row, &path).is_some()
            {
                editor.select(row);
                editor.toggle_editing(crate::frontend::schedule_editor::Editing::Node(row, path));
                app.retick();
            }
            Task::none()
        }
        "close" => {
            app.panels.schedule = None;
            app.retick();
            Task::none()
        }
        other => {
            log::warn!("ui command: unknown schedule demo action '{other}'");
            Task::none()
        }
    }
}

fn ui_demo_audio(app: &mut App, raw: &str) -> Task<Message> {
    let mut parts = raw.split_whitespace();
    let action = parts.next().unwrap_or("");
    let Some(volume) = parts.next().and_then(|value| value.parse::<u32>().ok()) else {
        log::warn!("ui command: 'audio-demo volume' needs a value from 0 to 100");
        return Task::none();
    };
    if action != "volume" || volume > 100 {
        log::warn!("ui command: usage 'audio-demo volume <0..100>'");
        return Task::none();
    }
    if let Some(session) = app.runtime_state.demo.as_mut() {
        session.audio_demo_volume = Some(volume);
    }
    let Some(panel) = app.panels.audio.as_mut() else {
        log::warn!("ui command: 'audio-demo volume' needs an open mixer");
        return Task::none();
    };
    let targets = panel
        .mons
        .iter()
        .filter(|monitor| monitor.has_audio_controls())
        .map(|monitor| monitor.name.clone())
        .collect::<Vec<_>>();
    for target in &targets {
        panel.set_mon_volume(target, volume);
        panel.set_mon_mute(target, true);
    }
    app.config.set_key(skwd_config::keys::wallpaper::MUTE, json!(true));
    app.daemon.client.call("wall.set_audio", json!({ "volume": 0, "mute": true }));
    app.panels.audio_playing = false;
    app.chrome.bar.cache.clear();
    app.retick();
    Task::none()
}

fn ui_demo_effect(app: &mut App, raw: &str) -> Task<Message> {
    let mut parts = raw.splitn(3, char::is_whitespace);
    let action = parts.next().unwrap_or("");
    let parameter = parts.next().unwrap_or("");
    let value = parts.next().unwrap_or("").trim();
    if !matches!(action, "choice" | "show") || parameter.is_empty() || value.is_empty() {
        log::warn!("ui command: usage 'effect-demo <choice|show> <parameter> <value>'");
        return Task::none();
    }
    let Some(effects) = app.panels.effects.as_ref() else {
        log::warn!("ui command: 'effect-demo {action}' needs an open Effects panel");
        return Task::none();
    };
    let valid = effects.selected().is_some_and(|effect| {
        effect.params.iter().any(|candidate| {
            candidate.id == parameter
                && matches!(candidate.kind, crate::domain::effects::EffectParamKind::Dropdown)
                && candidate.options.iter().any(|option| option.mode == value)
        })
    });
    if !valid {
        log::warn!("ui command: Effects choice '{parameter}={value}' is not available");
        return Task::none();
    }
    if action == "choice" {
        return super::update_inner(
            app,
            Message::Effects(crate::frontend::effects::EffectsMsg::SetChoice(
                parameter.to_string(),
                value.to_string(),
            )),
        );
    }

    let cache_key = {
        let effects = app.panels.effects.as_mut().expect("validated above");
        effects.set_str(parameter, value.to_string());
        serde_json::to_string(&crate::infrastructure::effects::encode_steps(
            &effects.preview_effects(),
        ))
        .unwrap_or_default()
    };
    let cached = app
        .runtime_state
        .demo
        .as_ref()
        .and_then(|session| session.effect_previews.get(&cache_key))
        .cloned();
    if let Some(path) = cached {
        let stale = app.panels.effects.as_mut().and_then(|effects| effects.begin_fade(path));
        if let Some(path) = stale {
            app.discard_effect_preview(&path);
        }
        app.retick();
        Task::none()
    } else {
        log::warn!("ui command: cached Effects choice '{parameter}={value}' is not ready");
        super::effects::effects_request_preview(app);
        Task::none()
    }
}

fn capture_demo_outputs(app: &mut App) {
    let snapshot = app
        .daemon
        .output_statuses
        .iter()
        .filter(|output| output.is_connected())
        .cloned()
        .collect::<Vec<_>>();
    if let Some(session) = app.runtime_state.demo.as_mut() {
        if session.overridden_outputs.is_empty() {
            session.overridden_outputs = snapshot;
        }
        session.overrides_active = true;
    }
}

fn ui_tune(app: &mut App, path: &str, val: &str) -> Task<Message> {
    if app.runtime_state.demo.is_none() {
        log::warn!("ui command: 'tune' requires an active demo session");
        return Task::none();
    }
    if !stage_demo_tune(app, path, val) {
        return Task::none();
    }
    app.apply_layout();
    app.invalidate_settings();
    Task::none()
}

fn stage_demo_tune(app: &mut App, path: &str, val: &str) -> bool {
    if path.is_empty() || val.is_empty() {
        log::warn!("ui command: usage 'tune <config.path> <value>'");
        return false;
    }
    let parsed = if val == "true" {
        json!(true)
    } else if val == "false" {
        json!(false)
    } else {
        val.parse::<f64>().map_or_else(|_| json!(val), |num| json!(num))
    };
    super::settings_policy::stage_value(app, path, &parsed);
    true
}

fn ui_motion(app: &mut App, raw: &str) -> Task<Message> {
    if app.runtime_state.demo.is_none() {
        log::warn!("ui command: 'motion' requires an active demo session");
        return Task::none();
    }
    let Ok(scale) = raw.parse::<f32>() else {
        log::warn!("ui command: usage 'motion <scale>'");
        return Task::none();
    };
    app.scene.set_motion_scale(scale);
    app.retick();
    Task::none()
}

fn ui_demo_scroll(app: &mut App, raw: &str) -> Task<Message> {
    let Some(session) = app.runtime_state.demo.as_mut() else {
        log::warn!("ui command: 'scroll-demo' requires an active demo session");
        return Task::none();
    };
    let rate = if raw == "stop" {
        0.0
    } else {
        let Ok(rate) = raw.parse::<f32>() else {
            log::warn!("ui command: usage 'scroll-demo <walls-per-second|stop>'");
            return Task::none();
        };
        if !rate.is_finite() {
            log::warn!("ui command: scroll rate must be finite");
            return Task::none();
        }
        rate.clamp(-12.0, 12.0)
    };
    session.scroll_rate = rate;
    app.retick();
    Task::none()
}

fn ui_recolour(app: &mut App, name: &str) -> Task<Message> {
    if app.runtime_state.demo.is_none() {
        log::warn!("ui command: 'recolour' requires an active demo session");
        return Task::none();
    }
    let Some(palette) = crate::app::helpers::local_static_palette(app, name) else {
        log::warn!("ui command: unknown recolour palette '{name}'");
        return Task::none();
    };
    app.config.set_key(skwd_config::keys::theme::POLICY, json!("fixed"));
    app.config.set_key(skwd_config::keys::theme::STATIC_THEME, json!(name));
    if let Some(designer) = app.panels.theme_designer.as_mut()
        && let Some(candidate) = crate::domain::theme::Candidate::from_preset(name)
    {
        designer.start_from_preset(name.to_string(), candidate);
    }
    app.theme.base_palette = palette;
    app.theme.fade_t =
        app.motion_profile().tween(1.0, crate::frontend::animation::MotionTier::Fast);
    app.start_fade(palette);
    app.init_settings_inputs();
    app.chrome.bar.cache.clear();
    app.invalidate_settings();
    app.retick();
    Task::none()
}

fn ui_demo_picker(app: &mut App, action: &str) -> Task<Message> {
    let suppressed = match action {
        "hide" => true,
        "show" => false,
        _ => {
            log::warn!("ui command: usage 'picker-demo <hide|show>'");
            return Task::none();
        }
    };
    if suppressed {
        close_all_overlays(app);
    }
    if let Some(session) = app.runtime_state.demo.as_mut() {
        session.picker_suppressed = suppressed;
    }
    app.retick();
    Task::none()
}

fn ui_demo_apply(app: &mut App, raw: &str) -> Task<Message> {
    if app.runtime_state.demo.is_none() {
        log::warn!("ui command: output-scoped 'apply' requires an active demo session");
        return Task::none();
    }
    let mut args = raw.split_whitespace();
    let output = args.next().unwrap_or("");
    let shader = args.next().unwrap_or("inkwell-drop");
    let duration_ms = args.next().and_then(|raw| raw.parse::<u64>().ok()).unwrap_or(220);
    let override_locks = app
        .runtime_state
        .demo
        .as_mut()
        .is_some_and(|session| std::mem::take(&mut session.override_next_apply));
    if output.is_empty()
        || !crate::frontend::settings::SHADERS.contains(&shader)
        || !(50..=10_000).contains(&duration_ms)
        || args.next().is_some()
    {
        log::warn!("ui command: usage 'apply <output> [transition-shader] [50..10000ms]'");
        return Task::none();
    }
    let Some(catalog_index) = demo_source_index(app) else {
        return Task::none();
    };
    let item = &app.library_session.library.catalog().items[catalog_index];
    if item.kind == crate::domain::library::catalog::WallpaperKind::We {
        log::warn!("ui command: demo transition applies deliberately exclude WE wallpapers");
        return Task::none();
    }
    let mut params = crate::app::helpers::apply_params(item, Vec::new());
    let item_path = item.path.clone();
    params["output"] = json!(output);
    params["transition"] = json!(true);
    params["transition_shader"] = json!(shader);
    params["transition_duration_ms"] = json!(duration_ms);
    params["notify"] = json!(false);
    if override_locks {
        capture_demo_outputs(app);
        params["override_locks"] = json!(true);
    }
    if let Some(session) = app.runtime_state.demo.as_mut()
        && session.opening_blur_source.as_deref() == Some(item_path.as_str())
    {
        session.opening_blur_resolved = true;
    }
    app.daemon.client.call("wall.apply", params);
    Task::none()
}

fn restore_output_snapshots(app: &mut App, outputs: &[crate::contracts::daemon::OutputStatus]) {
    for output in outputs {
        let mut params = json!({
            "type": output.kind.as_key(),
            "output": output.target(),
            "notify": false,
            "no_transition": true,
            "override_locks": true,
        });
        match output.kind {
            crate::contracts::media::MediaKind::WallpaperEngine if !output.we_id.is_empty() => {
                params["we_id"] = json!(output.we_id);
                params["mute"] = json!(output.mute);
                params["volume"] = json!(output.volume);
            }
            crate::contracts::media::MediaKind::Video => {
                let path = if output.current.is_empty() { &output.path } else { &output.current };
                if path.is_empty() {
                    continue;
                }
                params["path"] = json!(path);
                params["mute"] = json!(output.mute);
                params["volume"] = json!(output.volume);
            }
            crate::contracts::media::MediaKind::Static => {
                let path = if output.current.is_empty() { &output.path } else { &output.current };
                if path.is_empty() {
                    continue;
                }
                params["path"] = json!(path);
            }
            crate::contracts::media::MediaKind::Other(_)
            | crate::contracts::media::MediaKind::WallpaperEngine => continue,
        }
        app.daemon.client.call("wall.apply", params);
    }
}

fn restore_demo_overrides(app: &mut App) {
    let outputs = app.runtime_state.demo.as_mut().map_or_else(Vec::new, |session| {
        session.overrides_active = false;
        std::mem::take(&mut session.overridden_outputs)
    });
    restore_output_snapshots(app, &outputs);
}

fn ui_demo_stage_blur(app: &mut App, raw: &str) -> Task<Message> {
    if app.runtime_state.demo.is_none() {
        log::warn!("ui command: 'stage-blur' requires an active demo session");
        return Task::none();
    }
    let Ok(radius) = raw.parse::<f32>() else {
        log::warn!("ui command: usage 'stage-blur <1..50>'");
        return Task::none();
    };
    if !(1.0..=50.0).contains(&radius) {
        log::warn!("ui command: usage 'stage-blur <1..50>'");
        return Task::none();
    }
    let Some(catalog_index) = demo_source_index(app) else {
        return Task::none();
    };
    let item = &app.library_session.library.catalog().items[catalog_index];
    if item.kind != crate::domain::library::catalog::WallpaperKind::Static {
        log::warn!("ui command: 'stage-blur' needs a selected static wallpaper");
        return Task::none();
    }
    let source = item.path.clone();
    let output_dir = std::path::PathBuf::from(app.config.cache_dir()).join("demo");
    if let Err(error) = std::fs::create_dir_all(&output_dir) {
        log::warn!("ui command: could not prepare demo cache for opening wallpaper blur: {error}");
        return Task::none();
    }
    let output = output_dir.join(format!("opening-blur-{}.png", std::process::id()));
    let blurred = match image::open(&source) {
        Ok(image) => image::imageops::blur(&image.into_rgba8(), radius),
        Err(error) => {
            log::warn!("ui command: could not decode opening wallpaper for blur: {error}");
            return Task::none();
        }
    };
    if let Err(error) = blurred.save(&output) {
        log::warn!("ui command: could not save opening wallpaper blur: {error}");
        return Task::none();
    }
    if let Some(previous) = app
        .runtime_state
        .demo
        .as_mut()
        .and_then(|session| session.opening_blur.replace(output.clone()))
        && previous != output
    {
        let _ = std::fs::remove_file(previous);
    }
    if let Some(session) = app.runtime_state.demo.as_mut() {
        session.opening_blur_source = Some(source);
        session.opening_blur_resolved = false;
    }
    app.daemon.client.call(
        "wall.apply",
        json!({
            "type": wall_proto::kind::STATIC,
            "path": output,
            "output": "*",
            "no_transition": true,
            "notify": false,
        }),
    );
    Task::none()
}

fn demo_source_index(app: &mut App) -> Option<usize> {
    let override_key =
        app.runtime_state.demo.as_mut().and_then(|session| session.apply_source.take());
    if let Some(key) = override_key {
        let index =
            app.library_session.library.catalog().items.iter().position(|item| item.key == key);
        if index.is_none() {
            app.show_toast(
                crate::i18n::tr_args!("status-demo-playback-missing", key => key.as_str()),
            );
            app.retick();
        }
        index
    } else {
        app.library_session.filtered.get(app.scene.current).map(|&index| index as usize)
    }
}

fn ui_colour_filter(app: &mut App, raw: &str) -> Task<Message> {
    let bucket = if matches!(raw, "" | "all" | "any" | "off" | "none") {
        -1
    } else {
        let Some(bucket) = crate::frontend::ui::parse_color_bucket(raw) else {
            log::warn!("ui command: unknown colour bucket '{raw}'");
            return Task::none();
        };
        bucket
    };
    super::update_inner(app, Message::SetColorFilter(bucket))
}

fn monitor_name(app: &App, token: &str) -> Option<String> {
    let monitors = app.panels.effects.as_ref()?.monitors();
    if let Ok(index) = token.parse::<usize>() {
        return monitors.get(index).map(|monitor| monitor.name.clone());
    }
    monitors
        .iter()
        .find(|monitor| monitor.name == token)
        .map(|monitor| monitor.name.clone())
        .or_else(|| app.runtime_state.demo.is_some().then(|| token.to_string()))
}

fn ui_display(app: &mut App, arg: &str) -> Task<Message> {
    use crate::frontend::effects::EffectsMsg;
    let mut parts = arg.split_whitespace();
    let action = parts.next().unwrap_or("");
    let target = parts.next().unwrap_or("");
    if app.panels.effects.is_none() {
        log::warn!("ui command: 'display {action}' needs an open displays desk");
        return Task::none();
    }
    match action {
        "stage" => {
            let key = parts.next().unwrap_or("");
            ui_demo_display_stage(app, target, key)
        }
        "all" => super::update_inner(app, Message::Effects(EffectsMsg::MonitorToggleAll)),
        "apply" => {
            let selected = app
                .panels
                .effects
                .as_ref()
                .is_some_and(crate::frontend::effects::Effects::has_selected_outputs);
            if !selected {
                log::warn!("ui command: 'display apply' needs a selected display");
                return Task::none();
            }
            super::update_inner(app, Message::Effects(EffectsMsg::MonitorApplySelected))
        }
        "hover" if target.is_empty() => {
            super::update_inner(app, Message::Effects(EffectsMsg::MonitorHover(None)))
        }
        "toggle" | "hover" => {
            let Some(name) = monitor_name(app, target) else {
                log::warn!("ui command: unknown display '{target}'");
                return Task::none();
            };
            let msg = if action == "toggle" {
                EffectsMsg::MonitorToggle(name)
            } else {
                EffectsMsg::MonitorHover(Some(name))
            };
            super::update_inner(app, Message::Effects(msg))
        }
        other => {
            log::warn!("ui command: unknown display action '{other}'");
            Task::none()
        }
    }
}

fn ui_demo_display_stage(app: &mut App, target: &str, key: &str) -> Task<Message> {
    if app.runtime_state.demo.is_none() {
        log::warn!("ui command: 'display stage' requires an active demo session");
        return Task::none();
    }
    if target.is_empty() || key.is_empty() {
        log::warn!("ui command: usage 'display stage <display> <wallpaper-key>'");
        return Task::none();
    }
    let output = app
        .panels
        .effects
        .as_ref()
        .and_then(|effects| {
            effects
                .monitors()
                .iter()
                .find(|monitor| monitor.target == target || monitor.name == target)
                .map(|monitor| monitor.target.clone())
        })
        .unwrap_or_else(|| target.to_string());
    let Some(store_index) =
        app.library_session.library.catalog().items.iter().position(|item| item.key == key)
    else {
        app.show_toast(crate::i18n::tr_args!("status-demo-missing", key => key));
        app.retick();
        return Task::none();
    };
    let Some(card) = demo_visible_card(app, store_index) else {
        app.show_toast(crate::i18n::tr_args!("status-demo-filtered", key => key));
        app.retick();
        return Task::none();
    };
    let item = &app.library_session.library.catalog().items[store_index];
    let kind = item.kind;
    let path = item.path.clone();
    let we_id = item.we_id.clone();
    let thumb = (!item.thumb.is_empty()).then(|| item.thumb.clone());
    if (kind != crate::domain::library::catalog::WallpaperKind::We && path.is_empty())
        || (kind == crate::domain::library::catalog::WallpaperKind::We && we_id.is_empty())
    {
        log::warn!("ui command: demo wallpaper '{key}' has no applicable source");
        return Task::none();
    }

    let Some(effects) = app.panels.effects.as_mut() else {
        log::warn!("ui command: 'display stage' needs an open displays desk");
        return Task::none();
    };
    effects.replace_source(
        path.clone(),
        thumb.clone(),
        card,
        kind,
        app.config.wallpaper_mute(),
        app.config.wallpaper_volume(),
    );
    effects.select_only_output(&output);
    effects.record_output_source(&output, kind, path.clone(), we_id.clone(), thumb);
    app.scene.set_current(card, app.library_session.filtered.len());
    app.retick();
    Task::none()
}

fn ui_demo_badges(app: &mut App, raw: &str) -> Task<Message> {
    let suppressed = match raw {
        "hide" | "off" => true,
        "show" | "on" => false,
        _ => {
            log::warn!("ui command: usage 'badges <show|hide>'");
            return Task::none();
        }
    };
    if let Some(session) = app.runtime_state.demo.as_mut() {
        session.type_badges_suppressed = suppressed;
    }
    app.chrome.cache.clear();
    app.retick();
    Task::none()
}

fn selected_key(app: &App) -> String {
    app.library_session
        .filtered
        .get(app.scene.current)
        .and_then(|&index| app.library_session.library.catalog().items.get(index as usize))
        .map_or_else(String::new, |wallpaper| wallpaper.key.clone())
}

fn visible_card(app: &App, store_index: usize) -> Option<usize> {
    app.library_session.filtered.iter().position(|&index| index as usize == store_index)
}

fn demo_visible_card(app: &mut App, store_index: usize) -> Option<usize> {
    if let Some(card) = visible_card(app, store_index) {
        return Some(card);
    }
    app.runtime_state.demo.as_ref()?;

    app.refilter_semantic_from_start();
    if let Some(card) = visible_card(app, store_index) {
        return Some(card);
    }

    let kind = app.library_session.library.catalog().items[store_index]
        .effective_kind()
        .as_str()
        .to_owned();
    let sort = app.library_session.filters.sort.clone();
    app.tags.tag_search.clear();
    app.tags.semantic.search.clear();
    app.clear_semantic_search();
    app.library_session.filters = crate::domain::library::filter::Filters {
        kind,
        sort,
        ..crate::domain::library::filter::Filters::default()
    };
    app.chrome.bar.cache.clear();
    app.refilter_semantic_from_start();
    visible_card(app, store_index)
}

fn select_key(app: &mut App, key: &str) -> Task<Message> {
    let catalog = app.library_session.library.catalog();
    let Some(store_index) = catalog.items.iter().position(|wallpaper| wallpaper.key == key) else {
        app.show_toast(crate::i18n::tr_args!("status-demo-missing", key => key));
        app.retick();
        return Task::none();
    };
    let Some(mut filtered_index) = demo_visible_card(app, store_index) else {
        app.show_toast(crate::i18n::tr_args!("status-demo-filtered", key => key));
        app.retick();
        return Task::none();
    };
    app.scene.kb_nav = true;
    let count = app.library_session.filtered.len();
    if app.runtime_state.demo.is_some() && count > 2 {
        let showcase_index = app.demo_showcase_index(count);
        if filtered_index < showcase_index {
            app.library_session.filtered.rotate_right(showcase_index - filtered_index);
        } else if filtered_index > showcase_index {
            app.library_session.filtered.rotate_left(filtered_index - showcase_index);
        }
        filtered_index = showcase_index;
        app.scene.reset_to_index(filtered_index, count);
    } else {
        app.scene.set_current(filtered_index, count);
    }
    app.retick();
    Task::none()
}

fn close_all_overlays(app: &mut App) {
    while app.close_topmost_overlay() {}
}

fn demo_begin(app: &mut App) -> Task<Message> {
    if app.runtime_state.demo.is_none() {
        let output_snapshot = app
            .daemon
            .output_statuses
            .iter()
            .filter(|output| output.is_connected())
            .cloned()
            .collect();
        let session = DemoSession {
            config: app.config.begin_transient(),
            filters: app.library_session.filters.clone(),
            query: app.tags.tag_search.clone(),
            selection_key: selected_key(app),
            filter_bar_visible: app.chrome.filter_bar_visible,
            palette: app.theme.palette,
            base_palette: app.theme.base_palette,
            motion_scale: app.scene.motion_scale(),
            type_badges_suppressed: false,
            opening_blur: None,
            opening_blur_source: None,
            opening_blur_resolved: false,
            apply_source: None,
            effect_previews: std::collections::HashMap::new(),
            overridden_outputs: output_snapshot,
            overrides_active: false,
            override_next_apply: false,
            audio_demo_volume: None,
            scroll_rate: 0.0,
            picker_suppressed: false,
            batch_id: None,
            batch_commands: Vec::new(),
        };
        app.runtime_state.demo = Some(session);
    }
    app.call_tracked("wall.outputs", json!({}), Pending::DemoOutputs);
    close_all_overlays(app);
    app.tags.tag_search.clear();
    app.tags.semantic.search.clear();
    app.library_session.filters = crate::domain::library::filter::Filters::default();
    app.refilter();
    app.chrome.filter_bar_visible = true;
    app.chrome.bar.cache.clear();
    let count = app.library_session.filtered.len();
    app.scene.reset_to_index(app.demo_showcase_index(count), count);
    app.retick();
    Task::none()
}

fn demo_end(app: &mut App) -> Task<Message> {
    let Some(session) = app.runtime_state.demo.take() else {
        return Task::none();
    };
    if !session.opening_blur_resolved
        && let Some(source) = &session.opening_blur_source
    {
        app.daemon.client.call(
            "wall.apply",
            json!({
                "type": wall_proto::kind::STATIC,
                "path": source,
                "output": "*",
                "no_transition": true,
                "notify": false,
            }),
        );
    }
    if let Some(path) = &session.opening_blur {
        let _ = std::fs::remove_file(path);
    }
    close_all_overlays(app);
    restore_output_snapshots(app, &session.overridden_outputs);
    for path in session.effect_previews.values().collect::<std::collections::HashSet<_>>() {
        app.daemon.client.call("effects.discard", json!({ "preview": path }));
    }
    app.config.restore_transient(session.config);
    app.daemon.client.call(
        "wall.set_audio",
        json!({
            "volume": app.config.wallpaper_volume(),
            "mute": app.config.wallpaper_mute(),
        }),
    );
    app.library_session.filters = session.filters;
    app.tags.tag_search = session.query;
    app.tags.semantic.search.clear();
    app.refilter();
    app.apply_layout();
    app.scene.set_motion_scale(session.motion_scale);
    app.theme.palette = session.palette;
    app.theme.base_palette = session.base_palette;
    app.theme.fade_from = session.palette;
    app.theme.fade_to = session.palette;
    app.theme.fade_t =
        app.motion_profile().tween(1.0, crate::frontend::animation::MotionTier::Standard);
    app.chrome.filter_bar_visible = session.filter_bar_visible;
    app.chrome.bar.cache.clear();
    if !session.selection_key.is_empty() {
        let _ = select_key(app, &session.selection_key);
    }
    app.retick();
    Task::none()
}
