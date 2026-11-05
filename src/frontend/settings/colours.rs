#[cfg(test)]
#[path = "colours_tests.rs"]
mod tests;

use std::collections::HashSet;

use crate::contracts::settings::{LockScreenTarget, SettingsSource, keys, schema};
use crate::i18n::{tr, tr_args};

use super::category::{INTEGRATIONS, THEME};
use super::places::{
    Asleep, Bind, Choice, LIST_KEY, NO_SLOTS, PlaceSource, SUMMARY_JOINER, bind, column, indexed,
    listed, looked, noted, rows, spoken, take, unfold,
};
use super::tabs::{colours_asleep_rows, take_control_paths};
use super::wallpapers::acted;
use super::{
    ActionId, Card, Cell, Control, Group, GroupKind, Look, More, Note, Row, Say, Section, SectionId,
};

use keys::matugen;
use keys::theme;

pub(super) const ORIGIN: &str = "settings-colours-source";
pub(super) const GENERATOR: &str = "settings-colours-generator";
pub(super) const LOOK: &str = "settings-colours-look";
pub(super) const DESTINATIONS: &str = "settings-colours-go";
pub(super) const OUTPUTS: &str = "settings-app-themes-custom";
pub(super) const SHELLS: &str = "settings-colours-shells";
pub(super) const LOCKS: &str = "settings-colours-lock";

const ORIGIN_DESC: &str = "settings-colours-source-desc";
const ORIGIN_SAY: &str = "settings-colours-source-say";
const ORIGIN_MADE: &str = "settings-colours-source-made";
const POLICY_PHRASES: [(&str, &str); 3] = [
    (WALLPAPER, "settings-colours-source-follow"),
    (FIXED, "settings-colours-source-fixed"),
    (OFF, "settings-colours-source-off"),
];
const SHELL_NAMES: [(&str, &str); 4] = [
    ("caelestia", "settings-integrations-caelestia-card"),
    (NOCTALIA, "settings-integrations-noctalia-card"),
    (DMS, "settings-integrations-dms-card"),
    ("end4", "settings-colours-end4"),
];
const CONTROL_MENU: &str = "settings-colours-control-menu";
const GENERATOR_MENU: &str = "settings-colours-generator-menu";
const THEME_MENU: &str = "settings-colours-theme-menu";
const ALL_PRESETS: &str = "settings-colours-all-presets";
const GENERATOR_DESC: &str = "settings-colours-generator-desc";
const GENERATOR_SAY: &str = "settings-colours-generator-say";
const GENERATOR_SEEDED: &str = "settings-colours-generator-seeded";
const GENERATOR_SHELL: &str = "settings-colours-generator-shell";
const PALETTE: &str = "settings-colours-palette";
const PALETTE_DESC: &str = "settings-colours-palette-desc";
const PALETTE_SAY: &str = "settings-colours-palette-say";
const IDLE: &str = "settings-colours-idle";
const NATIVE_NOTE: &str = "settings-theme-native-card-desc";
const INDEX_HINT: &str = "settings-colours-index-hint";
const CONTRAST_HINT: &str = "settings-colours-contrast-hint";
const WALLUST_HINT: &str = "settings-colours-wallust-hint";
const SEEDS_HINT: &str = "settings-colours-seeds-hint";
const CONFIG_HINT: &str = "settings-colours-config-hint";
const COMMAND_HINT: &str = "settings-colours-command-hint";
const AUTO: &str = "settings-colours-auto";
const NO_VALUE: &str = "settings-control-none";
const DEFAULT_MATUGEN_CONFIG: &str = "~/.config/matugen/config.toml";
const LOOK_DESC: &str = "settings-colours-look-desc";
const LOOK_SAY: &str = "settings-colours-look-say";
const LOOK_IMPORTED: &str = "settings-colours-look-imported";
const LOOK_SHELL: &str = "settings-colours-look-shell";
const LOOK_FIXED: &str = "settings-colours-look-fixed";
const NO_DESIGNER: &str = "settings-colours-no-designer";
const DESIGNER_HINT: &str = "settings-colours-designer-hint";
const FINISH_PLAIN: &str = "settings-theme-finish-natural-plain";
const GO_DESC: &str = "settings-colours-go-desc";
const GO_SAY: &str = "settings-colours-go-say";
const GO_LOADING: &str = "settings-app-themes-loading";
const FIND_HINT: &str = "settings-colours-find-hint";
const INSTALLED: &str = "settings-app-themes-installed";
const CONFIGURED: &str = "settings-app-themes-configured";
const OUTPUTS_DESC: &str = "settings-app-themes-custom-desc";
const OUTPUTS_SAY: &str = "settings-colours-outputs-say";
const RELOAD_HINT: &str = "settings-colours-reload-hint";
const MATUGEN_HINT: &str = "settings-colours-matugen-hint";
const ADD_LABEL: &str = "settings-matugen-add-label";
const OUTPUT_HEADING: &str = "settings-colours-output-heading";
const SHELLS_DESC: &str = "settings-colours-shells-desc";
const SHELLS_SAY: &str = "settings-colours-shells-say";
const NOCTALIA_HINT: &str = "settings-integrations-noctalia-card";
const DMS_HINT: &str = "settings-colours-dms-hint";
const MODE_HINT: &str = "settings-colours-mode-hint";
const BIN_HINT: &str = "settings-colours-bin-hint";
const LOCKS_DESC: &str = "settings-colours-lock-desc";
const LOCKS_SAY: &str = "settings-colours-lock-say";
const LOCKS_STATIC: &str = "settings-colours-lock-say-static";
const LOCK_FOLLOW: &str = "settings-colours-lock-follow";
const LOCK_STILL: &str = "settings-colours-lock-still";
const SDDM_HINT: &str = "settings-colours-sddm-hint";

const WALLPAPER: &str = "wallpaper";
const FIXED: &str = "fixed";
const OFF: &str = "off";
const SKWD: &str = "skwd";
const MATUGEN: &str = "matugen";
const NATIVE: &str = "native";
const NATURAL: &str = "natural";
const NOCTALIA: &str = "noctalia";
const DMS: &str = "dms";
const CHANGED: &str = "changed";
const NONE: &str = "none";
const ONE: &str = "one";
const SOME: &str = "some";
const SINGLE: &str = "single";
const MANY: &str = "many";
const CUSTOM_FOLD: &str = "app-themes.custom";
const APP_FOLD: &str = "app-profile:";
const SHELL_ARG: &str = "shell";
const POLICY_ARG: &str = "policy";
const AUTHORITY_ARG: &str = "authority";
const FINISH_ARG: &str = "finish";
const SCHEME_ARG: &str = "scheme";
const MODE_ARG: &str = "mode";
const STATE_ARG: &str = "state";
const COUNT_ARG: &str = "count";
const NAMES_ARG: &str = "names";
const ON_ARG: &str = "on";
const ALL_ARG: &str = "all";
const FOLLOWERS_ARG: &str = "followers";
const CHANGED_ARG: &str = "changed";
const NAME_ARG: &str = "name";
const SHORT_PRESETS: usize = 3;
const HEADING_GAP: &str = "\u{a0}";
const SEED_JOINER: char = ',';
const CUSTOM_THEME: &str = "custom";
const APPS_LEFT: usize = 5;
const SHELL_STANCES: [&str; 2] = ["keeps", "follows"];
const NOCTALIA_SAYS: Choice = Choice { arg: NOCTALIA, values: SHELL_STANCES };
const DMS_SAYS: Choice = Choice { arg: DMS, values: SHELL_STANCES };
const CAELESTIA_SAYS: Choice = Choice { arg: "caelestia", values: ["no", "yes"] };

const NAME_LEAF: &str = "name";
const TEMPLATE_LEAF: &str = "template";
const OUTPUT_LEAF: &str = "output";
const RELOAD_LEAF: &str = "reload";
const LIVE_LEAF: &str = "livePreview";
const ENABLED_LEAF: &str = "enabled";

const ENGINE_SLOT: Bind<'_> = Bind::new("engine", theme::ENGINE);
const INDEX_SLOT: Bind<'_> = Bind::new("index", matugen::COLOR_INDEX);
const CONTRAST_SLOT: Bind<'_> = Bind::new("contrast", matugen::CONTRAST);
const THEME_SLOT: Bind<'_> = Bind::new("theme", theme::STATIC_THEME);
const VARIANT_SLOT: Bind<'_> = Bind::new("variant", theme::MODE);
const IMAGE_SLOT: Bind<'_> =
    Bind { blank: Some(NO_VALUE), ..Bind::new("image", keys::lock_screens::IMAGE) };

struct Flow {
    policy: String,
    authority: Option<String>,
    engine: Option<String>,
}

impl Flow {
    fn of(pool: &[Row]) -> Self {
        Self {
            policy: current(pool, theme::POLICY).unwrap_or_else(|| WALLPAPER.to_string()),
            authority: current(pool, theme::AUTHORITY),
            engine: current(pool, theme::ENGINE),
        }
    }

    fn generating(&self) -> bool {
        self.engine.is_some()
    }

    fn engine_is(&self, engine: &str) -> bool {
        self.engine.as_deref() == Some(engine)
    }

    fn shell(&self) -> Option<&str> {
        self.authority.as_deref().filter(|authority| *authority != SKWD)
    }

    fn fixed(&self) -> bool {
        self.policy == FIXED
    }
}

fn current(pool: &[Row], path: &str) -> Option<String> {
    pool.iter().find_map(|row| match &row.control {
        Control::Dropdown { path: at, current, .. } if at == path => Some(current.clone()),
        _ => None,
    })
}

fn flat(cards: Vec<(Card, Vec<Row>)>) -> Vec<Row> {
    cards.into_iter().flat_map(|(_, rows)| rows).collect()
}

pub(super) fn compose(source: &PlaceSource<'_>) -> Vec<Section> {
    let cfg = source.cfg;
    let mut pool = unfold(flat(source.cards(THEME)), |id| id == CUSTOM_FOLD);
    let mut linked = unfold(flat(source.cards(INTEGRATIONS)), |id| id == keys::lock_screens::MODE);
    let flow = Flow::of(&pool);
    vec![
        origin(&mut pool, &flow),
        generator(cfg, &mut pool, &flow),
        look(&mut pool, &flow),
        destinations(source, &mut pool),
        outputs(cfg, &mut pool),
        shells(cfg, &mut linked),
        locks(&mut linked),
    ]
}

pub(super) fn asleep(sections: &[Section], source: &PlaceSource<'_>) -> Vec<Asleep> {
    let mut seen: HashSet<String> = sections
        .iter()
        .flat_map(Section::rows)
        .filter_map(|row| row.control.path().map(str::to_string))
        .collect();
    let cause = [theme::ENGINE, theme::AUTHORITY]
        .into_iter()
        .find(|path| seen.contains(*path))
        .unwrap_or(theme::POLICY);
    colours_asleep_rows(source.cfg, source.backends)
        .into_iter()
        .filter_map(|row| {
            let path = row.control.path()?.to_string();
            if !seen.insert(path.clone()) {
                return None;
            }
            let wakes = match path.as_str() {
                keys::lock_screens::IMAGE => keys::lock_screens::MODE,
                theme::STATIC_THEME | theme::CUSTOM_COLORS => theme::POLICY,
                _ => cause,
            };
            Some(Asleep { row, wakes })
        })
        .collect()
}

fn phrase(table: &[(&str, &'static str)], value: Option<&str>) -> Option<&'static str> {
    table.iter().find(|(key, _)| Some(*key) == value).map(|(_, phrase)| *phrase)
}

fn shown(rows: Vec<Row>, empty: &str) -> Vec<Row> {
    rows.into_iter()
        .map(|row| Row { look: Look { blank: Some(empty.to_string()), ..row.look }, ..row })
        .collect()
}

fn menu_titled(mut rows: Vec<Row>, key: &'static str) -> Vec<Row> {
    for row in &mut rows {
        row.look.heading = Some(tr(key).to_string());
    }
    rows
}

fn origin(pool: &mut Vec<Row>, flow: &Flow) -> Section {
    let mut head = take_control_paths(pool, &[theme::POLICY]);
    head.extend(menu_titled(take_control_paths(pool, &[theme::AUTHORITY]), CONTROL_MENU));
    let section = Section {
        head,
        ..Section::new(SectionId::new(THEME, ORIGIN), tr(ORIGIN), tr(ORIGIN_DESC))
    };
    let policy = Bind {
        blank: phrase(&POLICY_PHRASES, Some(&flow.policy)),
        ..Bind::new(POLICY_ARG, theme::POLICY)
    };
    let authority = Bind {
        blank: phrase(&SHELL_NAMES, flow.authority.as_deref()),
        ..Bind::new(AUTHORITY_ARG, theme::AUTHORITY)
    };
    spoken(section, |section| {
        if flow.authority.is_some() {
            bind(section, ORIGIN_MADE, &[policy, authority], &[])
        } else {
            bind(section, ORIGIN_SAY, &[policy], &[])
        }
    })
}

fn theme_more(mut rows: Vec<Row>, presets: &[(String, String)]) -> Vec<Row> {
    for row in &mut rows {
        let Control::Dropdown { options, current, .. } = &row.control else {
            continue;
        };
        let first = presets.iter().take(SHORT_PRESETS);
        let short: Vec<String> = options
            .iter()
            .map(|(value, _)| value)
            .filter(|value| {
                *value == current
                    || first.clone().any(|(preset, _)| preset == *value)
                    || !presets.iter().any(|(preset, _)| preset == *value)
            })
            .cloned()
            .collect();
        if short.len() < options.len() {
            let label = tr_args!(ALL_PRESETS, count => presets.len());
            row.look.more = Some(More { short, label });
        }
    }
    rows
}

fn generator(cfg: &dyn SettingsSource, pool: &mut Vec<Row>, flow: &Flow) -> Section {
    let mut head = menu_titled(take_control_paths(pool, &[theme::ENGINE]), GENERATOR_MENU);
    let presets = cfg.palette_presets();
    let mut picked = menu_titled(take_control_paths(pool, &[theme::STATIC_THEME]), THEME_MENU);
    for row in &mut picked {
        if let Control::Dropdown { palettes, .. } = &mut row.control {
            let seeds = cfg.text(theme::CUSTOM_COLORS);
            let seeds = seeds.split(SEED_JOINER).map(|hex| hex.trim().to_string()).collect();
            palettes.push((CUSTOM_THEME.to_string(), seeds));
        }
    }
    head.extend(theme_more(picked, &presets));
    let index = take_control_paths(pool, &[matugen::COLOR_INDEX]);
    let contrast = take_control_paths(pool, &[matugen::CONTRAST]);
    let mut groups = Vec::new();
    let echoed = |rows: &[Row], hint| {
        let mut group = column(None, Vec::new());
        group.echoes = noted(rows.to_vec(), Some(hint)).into_iter().map(|row| (0, row)).collect();
        group
    };
    if !index.is_empty() {
        groups.push(echoed(&index, INDEX_HINT));
        groups.push(echoed(&contrast, CONTRAST_HINT));
    }
    head.extend(index);
    head.extend(contrast);
    let single = [
        (theme::WALLUST_PALETTE, WALLUST_HINT, tr(AUTO)),
        (theme::CUSTOM_COLORS, SEEDS_HINT, tr(NO_VALUE)),
    ];
    for (path, hint, empty) in single {
        let row = shown(rows(pool, &[(path, Some(hint))]), empty);
        if !row.is_empty() {
            groups.push(Group::new(GroupKind::Wide, None, row));
        }
    }
    let config = rows(pool, &[(matugen::DEFAULT_CONFIG, Some(CONFIG_HINT))]);
    let command = rows(pool, &[(keys::system::EXTERNAL_MATUGEN_COMMAND, Some(COMMAND_HINT))]);
    let fallback = schema::text_default(keys::system::EXTERNAL_MATUGEN_COMMAND).unwrap_or_default();
    groups.push(column(None, shown(config, DEFAULT_MATUGEN_CONFIG)));
    groups.push(column(None, shown(command, fallback)));
    let (title, desc) =
        if flow.fixed() { (PALETTE, PALETTE_DESC) } else { (GENERATOR, GENERATOR_DESC) };
    let section = Section {
        head,
        groups,
        foot: flow.engine_is(NATIVE).then(|| tr(NATIVE_NOTE).to_string()),
        ..Section::new(SectionId::new(THEME, GENERATOR), tr(title), tr(desc))
    };
    let shell = shell_name(flow);
    spoken(section, |section| {
        if flow.fixed() {
            bind(section, PALETTE_SAY, &[THEME_SLOT], &[])
        } else if flow.engine_is(MATUGEN) {
            bind(section, GENERATOR_SEEDED, &[ENGINE_SLOT, INDEX_SLOT, CONTRAST_SLOT], &[])
        } else if flow.generating() {
            bind(section, GENERATOR_SAY, &[ENGINE_SLOT], &[])
        } else if flow.shell().is_some() {
            bind(section, GENERATOR_SHELL, &NO_SLOTS, &[(SHELL_ARG, shell)])
        } else {
            Some(Say { text: tr(IDLE).to_string(), ..Say::default() })
        }
    })
}

fn shell_name(flow: &Flow) -> &'static str {
    phrase(&SHELL_NAMES, flow.shell()).map_or("", tr)
}

fn look(pool: &mut Vec<Row>, flow: &Flow) -> Section {
    let scheme = [theme::SCHEME, matugen::SCHEME_TYPE]
        .into_iter()
        .find(|path| pool.iter().any(|row| row.control.path() == Some(path)));
    let style = current(pool, theme::STYLE);
    let mut head = Vec::new();
    if let Some(scheme) = scheme {
        head.extend(take_control_paths(pool, &[scheme]));
        head.extend(take_control_paths(pool, &[theme::STYLE]));
    }
    head.extend(take_control_paths(pool, &[theme::MODE]));
    let designer = noted(acted(pool, |id| *id == ActionId::OpenThemeDesigner), Some(DESIGNER_HINT));
    let idle = designer.is_empty();
    let section = Section {
        head,
        groups: if idle { Vec::new() } else { vec![Group::new(GroupKind::Wide, None, designer)] },
        foot: idle.then(|| tr(NO_DESIGNER).to_string()),
        ..Section::new(SectionId::new(THEME, LOOK), tr(LOOK), tr(LOOK_DESC))
    };
    let shell = shell_name(flow);
    let finish = Bind {
        blank: (style.as_deref() == Some(NATURAL)).then_some(FINISH_PLAIN),
        ..Bind::new(FINISH_ARG, theme::STYLE)
    };
    spoken(section, |section| {
        if let Some(scheme) = scheme {
            let slots = [Bind::new(SCHEME_ARG, scheme), finish, VARIANT_SLOT];
            return bind(section, LOOK_SAY, &slots, &[]);
        }
        if flow.fixed() {
            return Some(Say { text: tr(LOOK_FIXED).to_string(), ..Say::default() });
        }
        if flow.policy == OFF {
            return Some(Say { text: tr(IDLE).to_string(), ..Say::default() });
        }
        if section.head.is_empty() {
            return bind(section, LOOK_SHELL, &NO_SLOTS, &[(SHELL_ARG, shell)]);
        }
        bind(section, LOOK_IMPORTED, &[VARIANT_SLOT], &[(SHELL_ARG, shell)])
    })
}

fn app_row(fold: Row) -> Vec<Row> {
    let Control::Details { rows: inner, .. } = fold.control else {
        return Vec::new();
    };
    let mut inner = inner.into_iter();
    let (Some(profile), Some(mut app)) = (inner.next(), inner.next()) else {
        return Vec::new();
    };
    if let Control::AppTheme { app: status, .. } = &app.control {
        let label = super::folio::state_label(&status.state, &status.detail);
        let note = if label == tr(CONFIGURED) && status.installed {
            format!("{label}{SUMMARY_JOINER}{}", tr(INSTALLED))
        } else {
            label.to_string()
        };
        app.look.note = Note::Own(note);
    }
    vec![app, looked(vec![profile], Cell::Joined).remove(0)]
}

fn destinations(source: &PlaceSource<'_>, pool: &mut Vec<Row>) -> Section {
    let folds = take(
        pool,
        |control| matches!(control, Control::Details { id, .. } if id.starts_with(APP_FOLD)),
    );
    let find = noted(acted(pool, |id| *id == ActionId::RefreshAppThemes), Some(FIND_HINT));
    let mut left: Vec<Row> = folds.into_iter().flat_map(app_row).collect();
    let mut right = left.split_off(left.len().min(APPS_LEFT * 2));
    right.extend(find);
    let section = Section {
        groups: vec![column(None, left), column(None, right)],
        ..Section::new(SectionId::new(THEME, DESTINATIONS), tr(DESTINATIONS), tr(GO_DESC))
    };
    let Some(result) = source.runtime.app_themes else {
        return spoken(section, |_| {
            Some(Say { text: tr(GO_LOADING).to_string(), ..Say::default() })
        });
    };
    let installed = result.apps.iter().filter(|app| app.installed).count();
    let on = result.apps.iter().filter(|app| app.installed && app.enabled).count();
    let changed: Vec<&str> = result
        .apps
        .iter()
        .filter(|app| app.state == CHANGED)
        .map(|app| app.name.as_str())
        .collect();
    let state = match changed.len() {
        0 => NONE,
        1 => ONE,
        _ => SOME,
    };
    let name = changed.first().copied().unwrap_or_default();
    let count = changed.len().to_string();
    let (on_text, all) = (on.to_string(), installed.to_string());
    let followers = if on == 1 { ONE } else { SOME };
    spoken(section, |section| {
        bind(
            section,
            GO_SAY,
            &NO_SLOTS,
            &[
                (ON_ARG, &on_text),
                (ALL_ARG, &all),
                (FOLLOWERS_ARG, followers),
                (CHANGED_ARG, state),
                (NAME_ARG, name),
                (COUNT_ARG, &count),
            ],
        )
    })
}

fn output_rows(pool: &mut Vec<Row>, at: usize, many: bool) -> [Group; 2] {
    let leaf = |leaf: &str| indexed(keys::integrations::LIST, at, leaf);
    let (name, template, output) = (leaf(NAME_LEAF), leaf(TEMPLATE_LEAF), leaf(OUTPUT_LEAF));
    let (reload, live) = (leaf(RELOAD_LEAF), leaf(LIVE_LEAF));
    let profile = leaf(keys::integrations::PROFILE);
    let mut left =
        shown(rows(pool, &[(&name, None), (&template, None), (&output, None)]), tr(NO_VALUE));
    left.extend(looked(noted(take_control_paths(pool, &[&profile]), None), Cell::Menu));
    let mut right = shown(rows(pool, &[(&reload, Some(RELOAD_HINT))]), tr(NO_VALUE));
    right.extend(rows(pool, &[(&live, None)]));
    let mut lead = column(None, left);
    let mut trail = column(None, right);
    if many {
        lead.heading = Some(tr_args!(OUTPUT_HEADING, count => at + 1));
        trail.heading = Some(HEADING_GAP.to_string());
    }
    [lead, trail]
}

fn removing(pool: &mut Vec<Row>, at: usize) -> Vec<Row> {
    let index = u16::try_from(at).unwrap_or(u16::MAX);
    noted(acted(pool, |id| *id == ActionId::RemoveIntegration(index)), None)
}

fn outputs(cfg: &dyn SettingsSource, pool: &mut Vec<Row>) -> Section {
    let count = cfg.array_len(keys::integrations::LIST);
    let mut add = noted(acted(pool, |id| *id == ActionId::AddIntegration), None);
    for row in &mut add {
        if let Control::ActionBtn { label, .. } = &mut row.control {
            *label = tr(ADD_LABEL).to_string();
        }
    }
    let feature = rows(pool, &[(keys::features::MATUGEN, Some(MATUGEN_HINT))]);
    let mut groups = Vec::new();
    for at in 0..count {
        let [lead, mut trail] = output_rows(pool, at, count > 1);
        if at == 0 {
            trail.rows.extend(feature.iter().cloned());
        }
        groups.push(lead);
        groups.push(trail);
        let enabled = indexed(keys::integrations::LIST, at, ENABLED_LEAF);
        let mut buttons = rows(pool, &[(&enabled, None)]);
        buttons.extend(removing(pool, at));
        if at + 1 == count {
            buttons.append(&mut add);
        }
        groups.push(Group::new(GroupKind::Lines, None, buttons));
    }
    if count == 0 {
        groups.push(column(None, feature));
        groups.push(Group::new(GroupKind::Lines, None, add));
    }
    let section = Section {
        groups,
        ..Section::new(SectionId::new(THEME, OUTPUTS), tr(OUTPUTS), tr(OUTPUTS_DESC))
    };
    let state = match count {
        0 => NONE,
        1 => SINGLE,
        _ => MANY,
    };
    let shown = count.to_string();
    spoken(section, |section| {
        bind(section, OUTPUTS_SAY, &NO_SLOTS, &[(STATE_ARG, state), (COUNT_ARG, &shown)])
    })
}

fn follows(pool: &mut Vec<Row>, shell: &str, hint: &'static str) -> Vec<Row> {
    let rows = take(pool, |control| {
        matches!(
            control,
            Control::ToggleAction { id: ActionId::SetThemeTarget(target, _), .. } if *target == shell
        )
    });
    noted(rows, Some(hint))
}

fn on(rows: &[Row]) -> bool {
    rows.iter().any(|row| row.control.switched_on() == Some(true))
}

fn shells(cfg: &dyn SettingsSource, pool: &mut Vec<Row>) -> Section {
    let mut left = follows(pool, NOCTALIA, NOCTALIA_HINT);
    let mut right = follows(pool, DMS, DMS_HINT);
    let (noctalia, dms) = (on(&left), on(&right));
    left.extend(rows(
        pool,
        &[(keys::noctalia::THEME_MODE, Some(MODE_HINT)), (keys::noctalia::HOVER_PREVIEW, None)],
    ));
    right.extend(rows(
        pool,
        &[
            (keys::dms::HOVER_PREVIEW, None),
            (keys::paths::NOCTALIA_BIN, Some(BIN_HINT)),
            (keys::caelestia::WALLPAPER_SYNC, None),
        ],
    ));
    let section = Section {
        groups: vec![column(None, left), column(None, right)],
        ..Section::new(SectionId::new(INTEGRATIONS, SHELLS), tr(SHELLS), tr(SHELLS_DESC))
    };
    let caelestia = CAELESTIA_SAYS.pick(cfg.flag(keys::caelestia::WALLPAPER_SYNC));
    spoken(section, |section| {
        bind(
            section,
            SHELLS_SAY,
            &NO_SLOTS,
            &[NOCTALIA_SAYS.pick(noctalia), DMS_SAYS.pick(dms), caelestia],
        )
    })
}

fn locks(pool: &mut Vec<Row>) -> Section {
    let source = current(pool, keys::lock_screens::MODE);
    let head = take_control_paths(pool, &[keys::lock_screens::MODE, keys::lock_screens::IMAGE]);
    let still = head.iter().any(|row| row.control.path() == Some(keys::lock_screens::IMAGE));
    let targets = acted(pool, |id| matches!(id, ActionId::SetLockScreen(..)));
    let set: Vec<String> = targets
        .iter()
        .filter(|row| {
            matches!(row.control, Control::ActionBtn { id: ActionId::SetLockScreen(_, false), .. })
        })
        .map(|row| row.title.clone())
        .collect();
    let mut cells: Vec<Row> = targets
        .into_iter()
        .map(|row| {
            let sign_in = matches!(
                row.control,
                Control::ActionBtn { id: ActionId::SetLockScreen(LockScreenTarget::Sddm, _), .. }
            );
            noted(vec![row], sign_in.then_some(SDDM_HINT)).remove(0)
        })
        .collect();
    let chooser = take(pool, |control| matches!(control, Control::ActionChips { .. }));
    cells.extend(looked(noted(chooser, None), Cell::Menu));
    let right = cells.split_off(cells.len().div_ceil(2));
    let section = Section {
        head,
        groups: vec![column(None, cells), column(None, right)],
        ..Section::new(SectionId::new(INTEGRATIONS, LOCKS), tr(LOCKS), tr(LOCKS_DESC))
    };
    let phrase = if source.as_deref() == Some(keys::lock_screens::STATIC) {
        LOCK_STILL
    } else {
        LOCK_FOLLOW
    };
    let mode = Bind { blank: Some(phrase), ..Bind::new(MODE_ARG, keys::lock_screens::MODE) };
    let names: Vec<&str> = set.iter().map(String::as_str).collect();
    let names = if names.is_empty() { String::new() } else { listed(LIST_KEY, &names) };
    let state = match set.len() {
        0 => NONE,
        1 => ONE,
        _ => SOME,
    };
    spoken(section, |section| {
        let args = [(STATE_ARG, state), (NAMES_ARG, names.as_str())];
        if still {
            bind(section, LOCKS_STATIC, &[mode, IMAGE_SLOT], &args)
        } else {
            bind(section, LOCKS_SAY, &[mode], &args)
        }
    })
}
