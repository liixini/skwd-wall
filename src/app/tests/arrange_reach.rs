use super::*;
use crate::app::update::arrange::placement;
use crate::domain::strand::{HOVER, Item, Rect, thread_y};
use crate::frontend::arrange::{ArrangeMsg, Hit, Pointer, Prompt, Target};
use crate::frontend::strand::labels::group_width;
use crate::frontend::strand::tokens::{FRAME, GROUP, PLATE};
use iced::keyboard::{Key, Modifiers, key::Named};
use skwd_config::picker_layout::Size;

const LOWEST: (f32, f32) = (1366.0, 768.0);
const DESIGN: (f32, f32) = (1600.0, 900.0);
const TICK: f32 = 1.0 / 60.0;
const SETTLE: usize = 150;
const OUT: &str = "SKWD_REACH_OUT";
const COMMANDS: [&str; 4] = ["preview", "displays", "cancel", "done"];
const LARGE: f32 = 1.5;
const HIGHER: [((f32, f32), bool); 4] = [
    ((1920.0, 1080.0), false),
    ((1920.0, 1200.0), false),
    ((2560.0, 1440.0), true),
    ((3840.0, 2160.0), true),
];

const TOGETHER: [&[&str]; 4] =
    [&["bead", "gem"], &["label"], &["slot", "choice"], &["toast", "undo"]];

type Drag = (Target, [f32; 2]);

const SCENES: [(&str, &[Drag]); 9] = [
    ("default", &[]),
    ("search-right", &[(Target::Search, [960.0, 690.0])]),
    ("search-left", &[(Target::Search, [400.0, 690.0])]),
    ("picker-low", &[(Target::Picker, [683.0, 530.0])]),
    ("bar-row", &[(Target::Bar, [683.0, 726.0])]),
    ("entry-row", &[(Target::Arrange, [1310.0, 726.0])]),
    ("strip-row", &[(Target::Swatch, [1250.0, 726.0])]),
    ("floats-both", &[(Target::Arrange, [1300.0, 700.0]), (Target::Swatch, [1200.0, 650.0])]),
    (
        "all-low",
        &[
            (Target::Search, [960.0, 690.0]),
            (Target::Arrange, [150.0, 726.0]),
            (Target::Swatch, [300.0, 726.0]),
            (Target::Bar, [683.0, 20.0]),
        ],
    ),
];

fn edit(app: &mut App, message: ArrangeMsg) {
    let _ = update(app, Message::Arrange(message));
    refresh_arrange_outputs(app);
}

fn settle(app: &mut App) {
    for _ in 0..SETTLE {
        crate::app::update::arrange::tick(app, TICK);
        app.chrome.tick_filter_bar(TICK);
        if app.panels.arrange.is_none() {
            return;
        }
    }
}

fn open_at(viewport: (f32, f32), scale: f32) -> App {
    let mut app = test_app();
    app.scene.viewport = viewport;
    app.config.set_key(skwd_config::keys::general::UI_SCALE, json!(scale));
    app.config.select_layout_output(
        Some(String::from("DP-1")),
        Size { width: viewport.0, height: viewport.1 },
    );
    app.runtime_state.picker_output = Some(String::from("DP-1"));
    app.daemon.output_statuses = vec![crate::contracts::daemon::OutputStatus {
        name: String::from("DP-1"),
        target: String::from("DP-1"),
        connected: true,
        width: viewport.0 as i32,
        height: viewport.1 as i32,
        logical_width: viewport.0 as i32,
        logical_height: viewport.1 as i32,
        ..crate::contracts::daemon::OutputStatus::default()
    }];
    edit(&mut app, ArrangeMsg::Begin);
    settle(&mut app);
    app
}

fn rect(app: &App, target: Target) -> Rect {
    let rect = placement::rect(app, target);
    Rect::new(rect.x, rect.y, rect.width, rect.height)
}

fn grab(app: &App, target: Target) -> [f32; 2] {
    let rect = rect(app, target);
    let centre = rect.centre();
    if target == Target::Picker || target == Target::Search {
        return centre;
    }
    [rect.x + 2.0, centre[1]]
}

fn press_away(app: &mut App) {
    edit(app, ArrangeMsg::Pointer(Pointer::Press([1.0, 1.0])));
    edit(app, ArrangeMsg::Pointer(Pointer::Release([1.0, 1.0])));
    settle(app);
}

fn drag(app: &mut App, target: Target, centre: [f32; 2]) {
    let from = grab(app, target);
    let middle = rect(app, target).centre();
    let to = [centre[0] + from[0] - middle[0], centre[1] + from[1] - middle[1]];
    edit(app, ArrangeMsg::Pointer(Pointer::Press(from)));
    for step in 1..=8 {
        let share = step as f32 / 8.0;
        let at = [from[0] + (to[0] - from[0]) * share, from[1] + (to[1] - from[1]) * share];
        edit(app, ArrangeMsg::Pointer(Pointer::Move(at)));
        crate::app::update::arrange::tick(app, TICK);
    }
    edit(app, ArrangeMsg::Pointer(Pointer::Release(to)));
    press_away(app);
}

fn staged(viewport: (f32, f32), drags: &[Drag]) -> App {
    let mut app = open_at(viewport, 1.0);
    for (target, centre) in drags {
        drag(&mut app, *target, *centre);
    }
    app
}

fn controls(app: &App) -> Vec<(String, Rect)> {
    let editor = &app.panels.arrange.as_ref().unwrap().editor;
    let mut found: Vec<(String, Rect)> = editor
        .plates()
        .into_iter()
        .filter(|(id, _)| !id.starts_with("bead:") && !id.starts_with("group:"))
        .map(|(id, bounds)| {
            let gem = id.starts_with("gem:");
            (id, if gem { Rect::centred(bounds.centre(), [2.0 * FRAME.gem; 2]) } else { bounds })
        })
        .collect();
    found.extend(
        editor.panel().iter().enumerate().map(|(at, slot)| (format!("slot:{at}"), slot.rect)),
    );
    if let Some(toast) = editor.spool.toast_plates(editor.now) {
        found.push((
            "toast".to_owned(),
            Rect::new(toast.x, toast.y, toast.label_width, PLATE.height),
        ));
    }
    let laid = editor.spool.laid.as_ref().unwrap();
    for placed in laid.plan.items.iter().filter(|placed| !placed.off) {
        let x = editor.spool.item_x(editor.now, placed.item);
        let y = editor.spool.thread_at(x);
        match placed.item {
            Item::Group(kind) => {
                let width = group_width(editor.spool.beads[kind].len());
                let top = y - HOVER - GROUP.stem - GROUP.height;
                found.push((
                    format!("label:{kind}"),
                    Rect::new(x - width / 2.0, top, width, GROUP.height),
                ));
            }
            Item::Bead(kind, style) => {
                found.push((
                    format!("bead:{kind}:{style}"),
                    Rect::centred([x, y], [2.0 * HOVER; 2]),
                ));
            }
            _ => {}
        }
    }
    found
}

fn audit(app: &App, scene: &str, state: &str, faults: &mut Vec<String>) {
    let (width, height) = app.scene.viewport;
    let editor = &app.panels.arrange.as_ref().unwrap().editor;
    let shown = controls(app);
    let tag = crate::i18n::active_tag();
    let overlay = |id: &str| {
        ["tool:", "gem:", "slot:", "choice:", "toast", "undo"]
            .iter()
            .any(|mark| id.starts_with(mark))
    };
    for (id, bounds) in &shown {
        if bounds.x < 0.0 || bounds.y < 0.0 || bounds.right() > width || bounds.bottom() > height {
            faults.push(format!("{tag} {scene} {state}: {id} off screen {bounds:?}"));
        }
        let crossing = state.starts_with("open") && !COMMANDS.contains(&id.as_str());
        for target in editor.targets().into_iter().filter(|_| !overlay(id) && !crossing) {
            if bounds.overlaps(rect(app, target)) {
                faults.push(format!("{tag} {scene} {state}: {id} under {target:?} {bounds:?}"));
            }
        }
    }
    for (index, (id, bounds)) in shown.iter().enumerate() {
        for (other, near) in &shown[index + 1..] {
            let fault = format!("{scene} {state}: {id} on {other}");
            if clash(id, other) && bounds.overlaps(*near) {
                faults.push(format!("{tag} {fault}"));
            }
        }
    }
    for id in COMMANDS {
        let Some((_, bounds)) = shown.iter().find(|(name, _)| name == id) else { continue };
        let hit = editor.hit(bounds.centre());
        if !matches!(hit, Some(Hit::Item(Item::Extra(_) | Item::Cancel | Item::Done))) {
            faults.push(format!("{tag} {scene} {state}: {id} not hit, got {hit:?}"));
        }
    }
}

fn clash(one: &str, other: &str) -> bool {
    let root = |id: &str| id.split(':').next().unwrap_or_default().to_owned();
    let (one, other) = (root(one), root(other));
    !TOGETHER.iter().any(|set| set.contains(&one.as_str()) && set.contains(&other.as_str()))
}

fn sweep(app: &mut App, scene: &str, faults: &mut Vec<String>) {
    audit(app, scene, "rest", faults);
    let targets = app.panels.arrange.as_ref().unwrap().editor.targets();
    for target in targets {
        app.panels.arrange.as_mut().unwrap().editor.select(target);
        settle(app);
        audit(app, scene, &format!("select-{target:?}"), faults);
    }
    app.panels.arrange.as_mut().unwrap().editor.deselect();
    let kinds: Vec<String> = controls(app)
        .into_iter()
        .filter(|(id, _)| id.starts_with("family:"))
        .map(|(id, _)| id)
        .collect();
    for id in kinds {
        let Some(at) = spot_of(app, &id) else { continue };
        edit(app, ArrangeMsg::Pointer(Pointer::Move(at)));
        settle(app);
        audit(app, scene, &format!("open-{id}"), faults);
    }
    press_away(app);
    prompts(app, scene, faults);
    hide_toast(app, scene, faults);
    preview_back(app, scene, faults);
}

fn spot_of(app: &App, id: &str) -> Option<[f32; 2]> {
    controls(app).into_iter().find(|(name, _)| name == id).map(|(_, bounds)| bounds.centre())
}

fn prompts(app: &mut App, scene: &str, faults: &mut Vec<String>) {
    let names = vec![String::from("DP-2"), String::from("HDMI-A-1"), String::from("eDP-1")];
    for (state, prompt) in
        [("conflict", Prompt::Conflict), ("disconnected", Prompt::Disconnected(names))]
    {
        let editor = &mut app.panels.arrange.as_mut().unwrap().editor;
        editor.board.prompt = Some(prompt);
        settle(app);
        audit(app, scene, state, faults);
        let editor = &mut app.panels.arrange.as_mut().unwrap().editor;
        editor.board.prompt = None;
        settle(app);
    }
}

fn hide_toast(app: &mut App, scene: &str, faults: &mut Vec<String>) {
    if !app.panels.arrange.as_ref().unwrap().editor.targets().contains(&Target::Swatch) {
        return;
    }
    app.panels.arrange.as_mut().unwrap().editor.select(Target::Swatch);
    settle(app);
    let Some(hide) = spot_of(app, "tool:Hide") else {
        faults.push(format!("{} {scene}: hide missing", crate::i18n::active_tag()));
        return;
    };
    edit(app, ArrangeMsg::Pointer(Pointer::Press(hide)));
    edit(app, ArrangeMsg::Pointer(Pointer::Release(hide)));
    for _ in 0..SETTLE / 2 {
        crate::app::update::arrange::tick(app, TICK);
    }
    audit(app, scene, "toast", faults);
    settle(app);
    settle(app);
    let _ = update(app, Message::Arrange(ArrangeMsg::Undo));
    settle(app);
}

fn preview_back(app: &mut App, scene: &str, faults: &mut Vec<String>) {
    let Some(at) = spot_of(app, "preview") else { return };
    edit(app, ArrangeMsg::Pointer(Pointer::Press(at)));
    edit(app, ArrangeMsg::Pointer(Pointer::Release(at)));
    settle(app);
    let (width, height) = app.scene.viewport;
    let editor = &app.panels.arrange.as_ref().unwrap().editor;
    let back = editor.back_rect();
    let tag = crate::i18n::active_tag();
    if back.x < 0.0 || back.y < 0.0 || back.right() > width || back.bottom() > height {
        faults.push(format!("{tag} {scene}: back off screen {back:?}"));
    }
    if editor.hit(back.centre()) != Some(Hit::Back) {
        faults.push(format!("{tag} {scene}: back not hit"));
    }
    let centre = back.centre();
    edit(app, ArrangeMsg::Pointer(Pointer::Press(centre)));
    edit(app, ArrangeMsg::Pointer(Pointer::Release(centre)));
    settle(app);
}

fn click_done(app: &mut App) {
    use iced::advanced::clipboard::Null;
    use iced::mouse::{Button, Cursor, Event as Mouse};
    use iced_runtime::user_interface::{Cache, UserInterface};
    let at = spot_of(app, "done").expect("done plate");
    let screen = iced::Size::new(app.scene.viewport.0, app.scene.viewport.1);
    let mut renderer = crate::frontend::headless::renderer(crate::frontend::ui::UI_FONT);
    let mut messages = Vec::new();
    {
        let view = crate::app::view::view_single(app);
        let mut ui = UserInterface::build(view, screen, Cache::default(), &mut renderer);
        let press = [
            iced::Event::Mouse(Mouse::ButtonPressed(Button::Left)),
            iced::Event::Mouse(Mouse::ButtonReleased(Button::Left)),
        ];
        let cursor = Cursor::Available(iced::Point::new(at[0], at[1]));
        let _ = ui.update(&press, cursor, &mut renderer, &mut Null, &mut messages);
    }
    for message in messages {
        let _ = update(app, message);
        refresh_arrange_outputs(app);
    }
    settle(app);
}

fn load(script: crate::i18n::Script) {
    iced_wgpu::graphics::text::font_system()
        .write()
        .expect("font system")
        .load_font(crate::shell::ui_font_bytes(script).into());
}

fn reaches_done(app: App, scene: &str, faults: &mut Vec<String>, drags: &[Drag]) {
    let tag = crate::i18n::active_tag();
    let mut app = app;
    if spot_of(&app, "done").is_none() {
        faults.push(format!("{tag} {scene}: done missing"));
        return;
    }
    click_done(&mut app);
    if app.panels.arrange.is_some() {
        faults.push(format!("{tag} {scene}: click did not reach done"));
    }
    let mut app = staged(LOWEST, drags);
    let _ = update(&mut app, Message::KeyPressed(Key::Named(Named::Enter), Modifiers::CTRL));
    settle(&mut app);
    if app.panels.arrange.is_some() {
        faults.push(format!("{tag} {scene}: control enter did not save"));
    }
}

#[test]
#[ignore = "offscreen layout probe of the Arrange editor in every locale"]
fn done_reachable_locales() {
    let mut faults = Vec::new();
    for language in crate::i18n::LANGUAGES {
        load(language.script);
        crate::i18n::with_language(language.tag, || {
            for (scene, drags) in SCENES {
                let mut app = staged(LOWEST, drags);
                sweep(&mut app, scene, &mut faults);
                reaches_done(staged(LOWEST, drags), scene, &mut faults, drags);
            }
            let mut large = open_at(LOWEST, LARGE);
            sweep(&mut large, "large", &mut faults);
        });
    }
    assert!(faults.is_empty(), "{} faults\n{}", faults.len(), faults.join("\n"));
}

fn framed(rect: Rect) -> Rect {
    rect.padded(FRAME.widget_pad + FRAME.gem * FRAME.gem_grow)
}

#[test]
fn floats_clear_panel() {
    let mut faults = Vec::new();
    for language in crate::i18n::LANGUAGES {
        load(language.script);
        crate::i18n::with_language(language.tag, || {
            for scale in [1.0, LARGE] {
                let app = open_at(LOWEST, scale);
                let mut found = vec![
                    ("arrange", rect(&app, Target::Arrange)),
                    ("swatch", rect(&app, Target::Swatch)),
                    ("search", rect(&app, Target::Search)),
                ];
                let done = controls(&app).into_iter().find(|(id, _)| id == "done");
                found.extend(done.map(|(_, bounds)| ("done", bounds)));
                for (index, (name, one)) in found.iter().enumerate() {
                    for (other, two) in &found[index + 1..] {
                        if framed(*one).overlaps(framed(*two)) {
                            faults.push(format!("{} x{scale}: {name} on {other}", language.tag));
                        }
                    }
                }
            }
        });
    }
    assert!(faults.is_empty(), "{} faults\n{}", faults.len(), faults.join("\n"));
}

#[test]
fn higher_routes_fit() {
    for (viewport, inline) in HIGHER {
        let mut app = open_at(viewport, 1.0);
        let editor = &app.panels.arrange.as_ref().unwrap().editor;
        let laid = editor.spool.laid.as_ref().unwrap();
        assert!(laid.plan.fits, "{viewport:?}");
        if !inline {
            let mut faults = Vec::new();
            audit(&app, "higher", "docked", &mut faults);
            let tools = spot_of(&app, "family:3").unwrap();
            edit(&mut app, ArrangeMsg::Pointer(Pointer::Move(tools)));
            settle(&mut app);
            assert!(
                app.panels.arrange.as_ref().unwrap().editor.spool.laid.as_ref().unwrap().plan.fits
            );
            let plates = app.panels.arrange.as_ref().unwrap().editor.plates();
            for id in ["bead:colour-studio", "bead:widgets"] {
                assert!(plates.iter().any(|(name, _)| name == id));
            }
            audit(&app, "higher", "tools", &mut faults);
            assert!(faults.is_empty(), "{}", faults.join("\n"));
            continue;
        }
        assert!(editor.spool.docked().is_empty(), "{viewport:?}");
        assert!(laid.spans.is_empty(), "{viewport:?}");
        assert_eq!(editor.spool.lane.target(), thread_y(viewport.1));
        assert!(!laid.plan.folded.iter().any(|folded| *folded), "{viewport:?}");
    }
}

fn shoot(app: &mut App, out: &str, name: &str) {
    std::sync::Arc::make_mut(&mut app.scene.render).vis = 1.0;
    let size = iced::Size::new(app.scene.viewport.0, app.scene.viewport.1);
    let path = format!("{out}/{name}.png");
    let app = &*app;
    crate::frontend::headless::capture(size, crate::frontend::ui::UI_FONT, &path, || {
        crate::app::view::view_single(app)
    });
}

#[test]
#[ignore = "manual offscreen render of the Arrange editor"]
fn render_reach() {
    let out = std::env::var(OUT).expect(OUT);
    for tag in ["en-US", "ar-SA"] {
        let language = crate::i18n::LANGUAGES.iter().find(|language| language.tag == tag).unwrap();
        load(language.script);
        crate::i18n::with_language(tag, || {
            for (scene, drags) in SCENES {
                let mut app = staged(LOWEST, drags);
                shoot(&mut app, &out, &format!("{tag}-{scene}"));
            }
            let mut app = open_at(LOWEST, 1.0);
            app.panels.arrange.as_mut().unwrap().editor.select(Target::Picker);
            settle(&mut app);
            shoot(&mut app, &out, &format!("{tag}-picker-selected"));
            let mut app = open_at(LOWEST, LARGE);
            shoot(&mut app, &out, &format!("{tag}-large"));
        });
    }
    crate::i18n::with_language("en-US", || {
        let mut app = open_at(DESIGN, 1.0);
        shoot(&mut app, &out, "en-US-design");
    });
}
