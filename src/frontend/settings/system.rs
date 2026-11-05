#[cfg(test)]
#[path = "system_tests.rs"]
mod tests;

use crate::contracts::settings::{SettingsSource, keys};
use crate::i18n::{SYSTEM_LANGUAGE, text, tr, tr_args};

use super::category::APP;
use super::places::{Bind, Choice, PlaceSource, bind, lead, legacy, lone, rows, spoken, switched};
use super::tabs::{take_card, take_control_paths};
use super::{Card, Control, Group, GroupKind, Row, Section, SectionId};

pub(super) const POWER: &str = "settings-performance-power-card";
const GPU: &str = "settings-performance-gpu-label";
pub(super) const LANGUAGE: &str = "settings-language-choice-label";
const BUG_REPORT: &str = "settings-system-bug-title";
const TRANSFER: &str = "settings-configuration-transfer-card";
const EFFICIENCY: &str = "settings-section-wallpaper-efficiency";
const POOLED: [&str; 3] =
    ["settings-section-power", "settings-section-picker-rendering", EFFICIENCY];
const LANGUAGE_CARD: &str = "settings-language-card";
const BUG_REPORT_CARD: &str = "settings-performance-bug-report-label";
const POWER_SOURCE: &str = "settings-system-power-source";
const POWER_SAY: &str = "settings-system-power-say";
const POWER_OFF: &str = "settings-system-power-off";
const GPU_SAY: &str = "settings-system-gpu-say";
const LANGUAGE_SAY: &str = "settings-system-language-say";
const EFFECTIVE: &str = "settings-performance-effective-label";
const EFFECTIVE_LIMITS: &str = "settings-performance-effective";
const EFFECTIVE_NO_IDLE: &str = "settings-performance-effective-no-idle";
pub(super) const CONTROLS: &str = "settings-section-controls";

const ALWAYS: [(&str, Option<&str>); 3] = [
    (keys::general::MAX_FPS, Some("settings-system-max-fps-hint")),
    (keys::paper::IDLE_PAUSE_SECONDS, Some("settings-system-idle-hint")),
    (keys::paper::PERFORMANCE_MODE, None),
];
const ON_BATTERY: [(&str, Option<&str>); 3] = [
    (keys::performance::BATTERY_FPS, Some("settings-system-battery-fps-hint")),
    (keys::performance::BATTERY_VIDEO_IDLE_SECONDS, Some("settings-system-battery-idle-hint")),
    (
        keys::performance::BATTERY_WALLPAPER_PERFORMANCE,
        Some("settings-system-battery-wallpaper-hint"),
    ),
];
const POWER_SLOTS: [Bind<'_>; 2] = [
    Bind::new("fps", keys::performance::BATTERY_FPS),
    Bind::new("seconds", keys::performance::BATTERY_VIDEO_IDLE_SECONDS),
];
const FPS_CAP: Choice = Choice { arg: "cap", values: ["none", "cap"] };
const VIDEO_IDLE: Choice = Choice { arg: "idle", values: ["off", "on"] };
const BATTERY_DETAIL: Choice = Choice { arg: "detail", values: ["full", "performance"] };
const LANGUAGE_FOLLOWS: Choice = Choice { arg: "follow", values: ["chosen", "desktop"] };
const GPU_SLOT: Bind<'_> = Bind::new("gpu", keys::performance::GPU_PREFERENCE);
const LANGUAGE_SLOT: Bind<'_> = Bind::new("language", keys::general::LANGUAGE);
const GPU_MODE: &str = "mode";
const CUSTOM_GPU: &str = "custom";
const POWER_SOURCES: [&str; 3] = [
    "settings-performance-power-external",
    "settings-performance-power-battery-unlimited",
    "settings-performance-power-battery-limited",
];
const EFFECTIVE_REASONS: [&str; 3] = [
    "settings-performance-effective-reason-external",
    "settings-performance-effective-reason-disabled",
    "settings-performance-effective-reason-battery",
];

pub(super) fn compose(source: &PlaceSource<'_>) -> Vec<Section> {
    let cfg = source.cfg;
    let mut cards = source.cards(APP);
    let mut pool = Vec::new();
    for card in POOLED {
        pool.extend(take_card(&mut cards, text(card)));
    }
    let mut sections = vec![
        power(cfg, &mut pool),
        gpu(take_control_paths(&mut pool, &[keys::performance::GPU_PREFERENCE])),
        language(take_card(&mut cards, text(LANGUAGE_CARD))),
        said(
            lone(
                SectionId::new(APP, BUG_REPORT),
                tr("settings-system-bug-desc"),
                take_card(&mut cards, text(BUG_REPORT_CARD)),
            ),
            tr("settings-system-bug-say"),
        ),
        said(
            lone(SectionId::new(APP, TRANSFER), "", take_card(&mut cards, text(TRANSFER))),
            tr("settings-configuration-transfer-card-desc"),
        ),
    ];
    take_control_paths(&mut pool, &super::wallpapers::BORROWED);
    take_card(&mut cards, text(CONTROLS));
    take_card(&mut cards, text(super::layout_place::GENERAL_CARD));
    sections.push(legacy(
        SectionId::new(APP, EFFICIENCY),
        &Card::new(text(EFFICIENCY), tr("settings-paper-performance-card-desc")),
        pool,
    ));
    sections.extend(
        cards.into_iter().map(|(card, rows)| legacy(SectionId::new(APP, card.key), &card, rows)),
    );
    sections
}

fn said(mut section: Section, text: &str) -> Section {
    section.say.text = text.to_string();
    section
}

fn status(cfg: &dyn SettingsSource) -> (String, String) {
    let saver = cfg.flag(keys::performance::BATTERY_SAVER);
    let state = if cfg.on_battery_power() { 1 + usize::from(saver) } else { 0 };
    let reason = tr(EFFECTIVE_REASONS[state]);
    let fps = cfg.effective_picker_fps();
    let idle = cfg.effective_video_idle_seconds();
    let limits = if idle == 0 {
        tr_args!(EFFECTIVE_NO_IDLE, fps => fps, reason => reason)
    } else {
        tr_args!(EFFECTIVE_LIMITS, fps => fps, seconds => idle, reason => reason)
    };
    let source = tr_args!(POWER_SOURCE, source => tr(POWER_SOURCES[state]));
    (source, lead(tr(EFFECTIVE), &limits))
}

fn power(cfg: &dyn SettingsSource, pool: &mut Vec<Row>) -> Section {
    let (source, effective) = status(cfg);
    let section = Section {
        master: take_control_paths(pool, &[keys::performance::BATTERY_SAVER]).pop(),
        groups: vec![
            Group {
                note: Some(effective),
                ..Group::new(
                    GroupKind::Column,
                    Some(tr("settings-system-always").into()),
                    rows(pool, &ALWAYS),
                )
            },
            Group {
                follows_master: true,
                ..Group::new(
                    GroupKind::Column,
                    Some(tr("settings-system-on-battery").into()),
                    rows(pool, &ON_BATTERY),
                )
            },
        ],
        ..Section::new(SectionId::new(APP, POWER), tr(POWER), tr("settings-system-power-desc"))
    };
    let fps = cfg.number(keys::performance::BATTERY_FPS) > 0.0;
    let idle = cfg.number(keys::performance::BATTERY_VIDEO_IDLE_SECONDS) > 0.0;
    let detail = cfg.flag(keys::performance::BATTERY_WALLPAPER_PERFORMANCE);
    let args = [FPS_CAP.pick(fps), VIDEO_IDLE.pick(idle), BATTERY_DETAIL.pick(detail)];
    let mut section = spoken(section, |section| {
        switched(section, POWER_OFF, || bind(section, POWER_SAY, &POWER_SLOTS, &args))
    });
    section.say.aside = Some(source);
    section
}

fn gpu(head: Vec<Row>) -> Section {
    spoken(lone(SectionId::new(APP, GPU), tr("settings-system-gpu-desc"), head), |section| {
        let menu = &section.head.first()?.control;
        let Control::Dropdown { current, .. } = menu else {
            return None;
        };
        let mode = if menu.current_label().is_some() { current.clone() } else { CUSTOM_GPU.into() };
        bind(section, GPU_SAY, &[GPU_SLOT], &[(GPU_MODE, &mode)])
    })
}

fn language(head: Vec<Row>) -> Section {
    let section = lone(SectionId::new(APP, LANGUAGE), tr("settings-system-language-desc"), head);
    spoken(section, |section| {
        let Control::Dropdown { current, .. } = &section.head.first()?.control else {
            return None;
        };
        let follows = LANGUAGE_FOLLOWS.pick(current == SYSTEM_LANGUAGE);
        bind(section, LANGUAGE_SAY, &[LANGUAGE_SLOT], &[follows])
    })
}
