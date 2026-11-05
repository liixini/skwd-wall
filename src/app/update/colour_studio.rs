use iced::Task;
use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers};
use serde_json::{Map, Value, json};
use skwd_config::keys::theme;
use skwd_palette::studio::{Area, Tuning};

use crate::app::{App, Message, Pending};
use crate::domain::library::catalog::{Wallpaper as Item, WallpaperKind};
use crate::domain::studio::{Mode, ModeChoice, Saved, Snapshot, Source, Studio, is_area};
use crate::frontend::colour_studio::{
    ColourStudio, Decoded, Edit, Effect, Load, Row, Schemes, StudioMsg, Wallpaper,
};
use crate::frontend::theme::Palette;
use crate::i18n::{tr, tr_args};
use skwd_palette::studio::Method;

const PICK_KEY: &str = "a";
const FAILED: &str = "colour-studio-failed";
const MISSING: &str = "colour-studio-missing";
const KEPT: &str = "colour-studio-undone-colours";
const NO_SCHEME: &str = "colour-studio-matugen-missing";
const SPACE: &str = " ";

const RESTORED: [&str; 13] = [
    theme::POLICY,
    theme::AUTHORITY,
    theme::ENGINE,
    theme::STUDIO_METHOD,
    theme::STUDIO_MODE,
    theme::STUDIO_AREA,
    theme::STUDIO_AREA_IMAGE,
    theme::STUDIO_WARMTH,
    theme::STUDIO_COLOUR,
    theme::STUDIO_CONTRAST,
    theme::STUDIO_SHADES,
    theme::WALLPAPER_PROFILES,
    theme::MODE,
];

fn mode_key(mode: ModeChoice) -> &'static str {
    match mode {
        ModeChoice::Dark => theme::MODE_DARK,
        ModeChoice::Light => theme::MODE_LIGHT,
        ModeChoice::ByTime => theme::STUDIO_MODE_BY_TIME,
    }
}

fn starting_mode(app: &App, settings: &Map<String, Value>) -> (Mode, bool, ModeChoice) {
    let setting = |path| {
        settings
            .get(path)
            .and_then(Value::as_str)
            .map_or_else(|| app.config.str_path(path), str::to_string)
    };
    match skwd_config::studio_time::StudioMode::from_key(&setting(theme::STUDIO_MODE)) {
        skwd_config::studio_time::StudioMode::Dark => return (Mode::Dark, true, ModeChoice::Dark),
        skwd_config::studio_time::StudioMode::Light => {
            return (Mode::Light, true, ModeChoice::Light);
        }
        skwd_config::studio_time::StudioMode::ByTime => {
            return (crate::infrastructure::theme::studio_time().0, true, ModeChoice::ByTime);
        }
        skwd_config::studio_time::StudioMode::FollowTheme => {}
    }
    let dark = match setting(theme::MODE).as_str() {
        theme::MODE_DARK => true,
        theme::MODE_LIGHT => false,
        _ => app.theme.base_palette.studio().is_dark(),
    };
    let mode = if dark { Mode::Dark } else { Mode::Light };
    (mode, false, mode.into())
}

pub(in crate::app) fn config_writes(
    studio: &Studio,
    area: Option<(Area, String)>,
) -> Vec<(&'static str, Value)> {
    let mut writes = vec![
        (theme::POLICY, json!(theme::POLICY_WALLPAPER)),
        (theme::AUTHORITY, json!(theme::AUTHORITY_SKWD)),
        (theme::ENGINE, json!(wall_proto::STUDIO_ENGINE)),
        (theme::STUDIO_METHOD, json!(studio.source.key())),
        (theme::STUDIO_WARMTH, json!(studio.tuning.warmth)),
        (theme::STUDIO_COLOUR, json!(studio.tuning.colour)),
        (theme::STUDIO_CONTRAST, json!(studio.tuning.contrast)),
        (theme::STUDIO_SHADES, json!(studio.tuning.shades)),
    ];
    if let Some((area, image)) = area.filter(|_| is_area(&studio.source)) {
        writes.push((theme::STUDIO_AREA, json!(area.edges())));
        writes.push((theme::STUDIO_AREA_IMAGE, json!(image)));
    }
    let mode = if studio.mode_picked { mode_key(studio.mode_choice) } else { "" };
    writes.push((theme::STUDIO_MODE, json!(mode)));
    writes
}

fn tuning(app: &App, settings: &Map<String, Value>) -> Tuning {
    let number = |path| {
        settings.get(path).and_then(Value::as_f64).unwrap_or_else(|| app.config.num_path(path))
            as f32
    };
    Tuning::from_parts(
        number(theme::STUDIO_WARMTH),
        number(theme::STUDIO_COLOUR),
        number(theme::STUDIO_CONTRAST),
        settings
            .get(theme::STUDIO_SHADES)
            .and_then(Value::as_array)
            .map_or_else(|| app.config.array_slice(theme::STUDIO_SHADES), Vec::as_slice)
            .iter()
            .map(|step| step.as_i64().unwrap_or_default()),
    )
}

const SAVED_NAME: &str = "name";
const SAVED_METHOD: &str = "method";
const SAVED_AREA: &str = "area";
const SAVED_WARMTH: &str = "warmth";
const SAVED_COLOUR: &str = "colour";
const SAVED_CONTRAST: &str = "contrast";
const SAVED_SHADES: &str = "shades";

pub(in crate::app) fn saved_value(saved: &Saved) -> Value {
    let mut value = json!({
        SAVED_NAME: saved.name,
        SAVED_METHOD: saved.source.key(),
        SAVED_WARMTH: saved.tuning.warmth,
        SAVED_COLOUR: saved.tuning.colour,
        SAVED_CONTRAST: saved.tuning.contrast,
        SAVED_SHADES: saved.tuning.shades,
    });
    if let Some(area) = saved.area {
        value[SAVED_AREA] = json!(area.edges());
    }
    value
}

pub(in crate::app) fn saved_entry(value: &Value) -> Option<Saved> {
    let plain = Tuning::default();
    let number =
        |key: &str, fallback: f32| value[key].as_f64().map_or(fallback, |found| found as f32);
    let shades: Vec<i64> = value[SAVED_SHADES]
        .as_array()
        .map(|steps| steps.iter().filter_map(Value::as_i64).collect())
        .unwrap_or_default();
    let area = value[SAVED_AREA].as_array().and_then(|list| {
        let found: Vec<f32> =
            list.iter().filter_map(Value::as_f64).map(|edge| edge as f32).collect();
        Area::from_edges(&found)
    });
    Some(Saved {
        name: value[SAVED_NAME].as_str()?.to_string(),
        source: Source::parse(value[SAVED_METHOD].as_str()?),
        area,
        tuning: Tuning::from_parts(
            number(SAVED_WARMTH, plain.warmth),
            number(SAVED_COLOUR, plain.colour),
            number(SAVED_CONTRAST, plain.contrast),
            shades,
        ),
    })
}

fn saved_list(app: &App) -> Vec<Saved> {
    app.config.array_slice(theme::STUDIO_SAVED).iter().filter_map(saved_entry).collect()
}

fn config_area(app: &App, settings: &Map<String, Value>) -> Option<Area> {
    let found: Vec<f32> = settings
        .get(theme::STUDIO_AREA)
        .and_then(Value::as_array)
        .map_or_else(|| app.config.array_slice(theme::STUDIO_AREA), Vec::as_slice)
        .iter()
        .filter_map(Value::as_f64)
        .map(|edge| edge as f32)
        .collect();
    Area::from_edges(&found)
}

fn focused(app: &App) -> usize {
    app.scene.flipped().unwrap_or(app.scene.current)
}

fn item_at(app: &App, index: usize) -> Option<Item> {
    let catalog = *app.library_session.filtered.get(index)? as usize;
    app.library_session.library.catalog().items.get(catalog).cloned()
}

fn index_of(app: &App, key: &str) -> Option<usize> {
    app.library_session.filtered.iter().position(|&catalog| {
        app.library_session
            .library
            .catalog()
            .items
            .get(catalog as usize)
            .is_some_and(|item| item.key == key)
    })
}

fn catalog_item(app: &App, key: &str) -> Option<Item> {
    app.library_session.library.catalog().items.iter().find(|item| item.key == key).cloned()
}

fn image_path(item: &Item) -> String {
    if item.kind == WallpaperKind::We { item.thumb.clone() } else { item.path.clone() }
}

fn is_current(app: &App, item: &Item) -> bool {
    app.daemon
        .current_wallpaper_art
        .as_deref()
        .is_some_and(|art| art == item.path || art == item.thumb)
}

fn wallpaper_of(app: &App, item: &Item) -> Wallpaper {
    Wallpaper {
        key: item.key.clone(),
        label: item.name.clone(),
        image: iced::widget::image::Handle::from_path(&item.thumb),
        current: is_current(app, item),
        aspect: (item.width > 0 && item.height > 0).then(|| item.width as f32 / item.height as f32),
    }
}

fn decode_task(key: String, path: String) -> Task<Message> {
    Task::perform(async move { crate::frontend::colour_studio::decode(&path) }, move |decoded| {
        Message::StudioPixels(key.clone(), decoded)
    })
}

fn request(app: &mut App, item: &Item, area: Option<Area>, schemes: bool) -> bool {
    let wire = area.map(|area| wall_proto::StudioArea {
        left: area.left,
        top: area.top,
        right: area.right,
        bottom: area.bottom,
    });
    let id = app.call_tracked(
        wall_proto::rpc::THEME_STUDIO,
        json!(wall_proto::StudioRequest {
            image: Some(image_path(item)),
            area: wire,
            we_id: (item.kind == WallpaperKind::We).then(|| item.we_id.clone()),
            materials: schemes,
        }),
        Pending::Studio { key: item.key.clone(), area, schemes },
    );
    id > 0
}

pub(super) fn open(app: &mut App) -> Task<Message> {
    if app.panels.colour_studio.is_some() {
        app.panels.colour_studio = None;
        app.retick();
        return Task::none();
    }
    if app.menu_capturing() {
        return Task::none();
    }
    let Some(item) = item_at(app, focused(app)) else {
        return Task::none();
    };
    let wallpaper = wallpaper_of(app, &item);
    let settings = super::theme::wallpaper_settings(app, &item.key);
    let restored = (wallpaper.current || settings.is_some())
        && super::theme::wallpaper_theme_backend_for(app, &item.key) == wall_proto::STUDIO_ENGINE;
    let settings = settings.unwrap_or_default();
    let source = settings
        .get(theme::STUDIO_METHOD)
        .and_then(Value::as_str)
        .map_or_else(|| app.config.str_path(theme::STUDIO_METHOD), str::to_string);
    let tuning = if restored { tuning(app, &settings) } else { Tuning::default() };
    let (mode, picked, choice) = starting_mode(app, &settings);
    let mut model = Studio::new(Source::parse(&source), mode, tuning);
    model.mode_picked = picked;
    model.mode_choice = choice;
    model.live = Some(app.theme.base_palette.studio());
    let mut studio = ColourStudio::new(model, wallpaper, saved_list(app));
    let area = config_area(app, &settings).filter(|_| restored);
    studio.set_saved_area(area);
    studio.area_image = area.map(|_| {
        let image = settings
            .get(theme::STUDIO_AREA_IMAGE)
            .and_then(Value::as_str)
            .map_or_else(|| app.config.str_path(theme::STUDIO_AREA_IMAGE), str::to_string);
        (item.key.clone(), image)
    });
    studio.set_time_mode(crate::infrastructure::theme::studio_time().0);
    studio.sync();
    app.panels.colour_studio = Some(studio);
    app.panels.studio_restore = None;
    request(app, &item, None, false);
    if let Some(area) = area {
        request(app, &item, Some(area), false);
    }
    app.retick();
    decode_task(item.key.clone(), item.thumb.clone())
}

fn persist_saved(app: &mut App) {
    let Some(studio) = app.panels.colour_studio.as_ref() else {
        return;
    };
    let list: Vec<Value> = studio.saved.iter().map(saved_value).collect();
    app.config.set_key(theme::STUDIO_SAVED, Value::Array(list));
    app.config.persist();
}

fn request_area(app: &mut App, area: Area) {
    let Some(key) = app.panels.colour_studio.as_ref().map(|studio| studio.wallpaper.key.clone())
    else {
        return;
    };
    if let Some(item) = index_of(app, &key).and_then(|index| item_at(app, index)) {
        request(app, &item, Some(area), false);
    }
}

pub(super) fn update(app: &mut App, msg: &StudioMsg) -> Task<Message> {
    let effect = app.panels.colour_studio.as_mut().map_or(Effect::None, |studio| {
        studio.set_time_mode(crate::infrastructure::theme::studio_time().0);
        studio.update(msg)
    });
    let task = match effect {
        Effect::None => Task::none(),
        Effect::Close => {
            app.panels.colour_studio = None;
            Task::none()
        }
        Effect::Apply => apply(app),
        Effect::Undo => undo(app),
        Effect::Persist => {
            persist_saved(app);
            Task::none()
        }
        Effect::Area(area) => {
            request_area(app, area);
            Task::none()
        }
    };
    fetch_schemes(app);
    if matches!(msg, StudioMsg::Pick(_)) {
        missing_scheme(app);
    }
    app.retick();
    task
}

fn missing_scheme(app: &mut App) {
    let Some(studio) = app.panels.colour_studio.as_ref() else {
        return;
    };
    let material = matches!(studio.model.source, Source::Material(_));
    if material && studio.schemes == Schemes::Read && studio.palette().is_none() {
        app.show_toast(tr(NO_SCHEME));
    }
}

fn naming_key(key: &Key, modifiers: Modifiers) -> Option<StudioMsg> {
    match key {
        Key::Named(Named::Enter) => Some(StudioMsg::Save),
        Key::Named(Named::Escape) => Some(StudioMsg::CancelSave),
        Key::Named(Named::Backspace) => Some(StudioMsg::Name(Edit::Erase)),
        Key::Named(Named::Space) => Some(StudioMsg::Name(Edit::Insert(SPACE.to_string()))),
        Key::Character(text) if !modifiers.control() && !modifiers.alt() && !modifiers.logo() => {
            Some(StudioMsg::Name(Edit::Insert(text.to_string())))
        }
        _ => None,
    }
}

fn studio_key(studio: &ColourStudio, key: &Key, modifiers: Modifiers) -> StudioMsg {
    let row = studio.row;
    let picking = studio.picking.is_some();
    let plain = !modifiers.control() && !modifiers.alt() && !modifiers.logo();
    match key {
        Key::Named(Named::Escape) if picking => StudioMsg::CancelPick,
        Key::Named(Named::PageUp | Named::PageDown) if picking => StudioMsg::SkipIntro,
        Key::Character(text) if plain && text.eq_ignore_ascii_case(PICK_KEY) => {
            StudioMsg::StartPick
        }
        Key::Named(Named::ArrowUp) => StudioMsg::Row(row.cycled(-1)),
        Key::Named(Named::ArrowDown) => StudioMsg::Row(row.cycled(1)),
        Key::Named(Named::ArrowLeft) => StudioMsg::Step(row, -1, modifiers.shift()),
        Key::Named(Named::ArrowRight) => StudioMsg::Step(row, 1, modifiers.shift()),
        Key::Named(Named::Space) if row == Row::Compare => StudioMsg::Compare,
        Key::Named(Named::PageDown) => StudioMsg::SchemeStep(1),
        Key::Named(Named::PageUp) => StudioMsg::SchemeStep(-1),
        Key::Named(Named::Enter) => StudioMsg::Apply,
        Key::Named(Named::Escape) => StudioMsg::Close,
        _ => StudioMsg::SkipIntro,
    }
}

pub(super) fn key(app: &mut App, key: &Key, modifiers: Modifiers) -> Task<Message> {
    let Some(studio) = app.panels.colour_studio.as_ref() else {
        return Task::none();
    };
    let msg = if studio.naming.is_some() {
        match naming_key(key, modifiers) {
            Some(msg) => msg,
            None => return Task::none(),
        }
    } else {
        studio_key(studio, key, modifiers)
    };
    update(app, &msg)
}

pub(in crate::app) fn pixels(app: &mut App, key: &str, decoded: Option<Decoded>) {
    let Some(studio) =
        app.panels.colour_studio.as_mut().filter(|studio| studio.wallpaper.key == key)
    else {
        return;
    };
    studio.set_decoded(decoded.map_or(Load::Failed, Load::Ready));
    app.retick();
}

fn fetch_schemes(app: &mut App) {
    let Some(studio) = app.panels.colour_studio.as_mut() else {
        return;
    };
    let analysed = studio.analysed.as_deref() == Some(studio.wallpaper.key.as_str());
    if studio.schemes != Schemes::Unread || !analysed || !studio.wants_schemes() {
        return;
    }
    studio.schemes = Schemes::Reading;
    let key = studio.wallpaper.key.clone();
    let sent = catalog_item(app, &key).is_some_and(|item| request(app, &item, None, true));
    if !sent && let Some(studio) = app.panels.colour_studio.as_mut() {
        studio.schemes = Schemes::Unread;
    }
}

fn add_schemes(studio: &mut ColourStudio, result: Snapshot) {
    studio.schemes = Schemes::Read;
    match studio.model.snapshot.as_mut().filter(|known| known.image == result.image) {
        Some(known) => known.materials = result.materials,
        None => studio.model.snapshot = Some(result),
    }
    studio.sync();
}

pub(in crate::app) fn loaded(
    app: &mut App,
    key: &str,
    area: Option<Area>,
    schemes: bool,
    mut result: Snapshot,
) {
    let Some(studio) =
        app.panels.colour_studio.as_mut().filter(|studio| studio.wallpaper.key == key)
    else {
        return;
    };
    if schemes {
        add_schemes(studio, result);
        missing_scheme(app);
        app.retick();
        return;
    }
    if let Some(area) = area {
        studio.adopt_area(area, &result);
        app.retick();
        return;
    }
    studio.analysed = Some(key.to_string());
    let known = studio.model.snapshot.as_ref().filter(|known| known.image == result.image);
    if result.materials.is_empty()
        && let Some(known) = known
    {
        result.materials.clone_from(&known.materials);
    }
    if !result.materials.is_empty() {
        studio.schemes = Schemes::Read;
    }
    let owned = studio.area_image.as_ref().is_some_and(|(owner, _)| owner == key);
    let foreign = owned && studio.area_image.take().is_some_and(|(_, image)| image != result.image);
    if foreign {
        studio.set_saved_area(None);
        studio.invalidate();
        studio.refresh_area();
        if is_area(&studio.model.source) {
            studio.model.source = Source::Method(Method::Tonal.key().to_string());
            studio.group = studio.model.group();
        }
    }
    if let Some(dark) = result.dark.filter(|_| !studio.model.mode_picked) {
        studio.model.mode = if dark { Mode::Dark } else { Mode::Light };
    }
    let first = result.methods.first().map(|method| Source::Method(method.key.clone()));
    studio.model.snapshot = Some(result);
    let reading = studio.schemes != Schemes::Read;
    let waiting = (is_area(&studio.model.source) && studio.saved_area().is_some())
        || (reading && matches!(studio.model.source, Source::Material(_)));
    if !waiting
        && studio.model.palette().is_none()
        && let Some(first) = first
    {
        studio.model.source = first;
        studio.group = studio.model.group();
    }
    studio.sync();
    fetch_schemes(app);
    app.retick();
}

pub(in crate::app) fn failed(app: &mut App, key: &str, schemes: bool, error: &str) {
    if let Some(studio) =
        app.panels.colour_studio.as_mut().filter(|studio| studio.wallpaper.key == key)
    {
        if schemes {
            studio.schemes = Schemes::Failed;
        }
        studio.sync();
        app.show_toast(tr_args!(FAILED, error => error));
    }
    app.retick();
}

fn snapshot(app: &App) -> Vec<(&'static str, Option<Value>)> {
    RESTORED
        .iter()
        .map(|path| (*path, skwd_config::get(app.config.root(), path).cloned()))
        .collect()
}

fn apply(app: &mut App) -> Task<Message> {
    let Some(studio) = app.panels.colour_studio.as_ref() else {
        return Task::none();
    };
    let Some(palette) = studio.palette() else {
        return Task::none();
    };
    let scanned = studio.analysed.as_deref() == Some(studio.wallpaper.key.as_str());
    if !scanned && !matches!(studio.model.source, Source::Theme(_)) {
        return Task::none();
    }
    let image = studio.model.snapshot.as_ref().map(|snapshot| snapshot.image.clone());
    let writes = config_writes(&studio.model, studio.saved_area().zip(image));
    let current = studio.wallpaper.current;
    let key = studio.wallpaper.key.clone();
    let index = index_of(app, &key);
    if !current && index.is_none() {
        app.show_toast(tr(MISSING));
        app.retick();
        return Task::none();
    }
    let before = snapshot(app);
    let previous = app.theme.base_palette;
    let wallpaper =
        (!current).then(|| app.daemon.current_wallpaper_art.clone().unwrap_or_default());
    for (path, value) in writes {
        app.config.set_key(path, value);
    }
    crate::app::update::theme::update_pinned_settings_for(app, &key);
    app.config.persist();
    let palette = Palette::from_studio(&palette);
    app.theme.base_palette = palette;
    app.start_fade(&palette);
    app.invalidate_swatch();
    let task = if current {
        retheme(app);
        Task::none()
    } else {
        index.map_or_else(Task::none, |index| crate::app::helpers::apply_task(app, index))
    };
    if app.panels.studio_restore.is_none() {
        app.panels.studio_restore = Some((before, previous, wallpaper));
    }
    if let Some(studio) = app.panels.colour_studio.as_mut() {
        studio.mark_applied();
        studio.wallpaper.current = true;
        studio.model.live = Some(palette.studio());
        studio.sync();
    }
    app.retick();
    task
}

fn retheme(app: &App) {
    app.daemon.client.call(wall_proto::rpc::WALL_RETHEME, Value::Object(serde_json::Map::new()));
}

fn undo(app: &mut App) -> Task<Message> {
    let Some((before, previous, wallpaper)) = app.panels.studio_restore.take() else {
        return Task::none();
    };
    for (path, value) in before {
        match value {
            Some(value) => app.config.set_key(path, value),
            None => app.config.remove_key(path),
        }
    }
    app.config.persist();
    app.theme.base_palette = previous;
    app.start_fade(&previous);
    app.invalidate_swatch();
    let changed = wallpaper.is_some();
    let back = wallpaper.and_then(|art| {
        let items = &app.library_session.library.catalog().items;
        let known = !art.is_empty();
        items.iter().find(|item| known && (item.path == art || item.thumb == art)).cloned()
    });
    let kept = changed && back.is_none();
    let task = if let Some(item) = &back {
        match index_of(app, &item.key) {
            Some(index) => crate::app::helpers::apply_task(app, index),
            None => crate::app::helpers::apply_item(app, item),
        }
    } else {
        retheme(app);
        Task::none()
    };
    if let Some(studio) = app.panels.colour_studio.as_mut() {
        studio.mark_undone();
        studio.model.live = Some(previous.studio());
        if let Some(item) = &back {
            studio.wallpaper.current = item.key == studio.wallpaper.key;
        }
        studio.sync();
    }
    if kept {
        app.show_toast(tr(KEPT));
    }
    task
}
