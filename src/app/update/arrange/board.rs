use iced::{Color, Rectangle};
use paper_widgets::Tones;
use skwd_config::corners::{CornerStyle, Corners};

use super::controls::{SPACING_STEPS, corner_step, step};
use super::{picker_rect, placement, search_rect, spacing_key};
use crate::app::App;
use crate::domain::strand::Rect;
use crate::frontend::arrange::{
    Bead, Board, Button, Copy, CornerStep, Family, Grid, Prompt, Shelf, Style, Target, layout_name,
};
use crate::frontend::scene::layout::Mode;
use crate::frontend::strand::raster::ink;
use crate::frontend::ui::{BarItem, BarModel, BarShow, build_bar_with_tasks};

const TYPE_PREFIX: &str = "type:";
const SORT_PREFIX: &str = "sort:";
const COLOR_PREFIX: &str = "color:";
const FAMILIES: [Family; 4] = [Family::Types, Family::Sort, Family::Colours, Family::Tools];

fn rect(rect: Rectangle) -> Rect {
    Rect::new(rect.x, rect.y, rect.width, rect.height)
}

fn rgb(color: Color) -> [u8; 3] {
    let [red, green, blue, _] = color.into_rgba8();
    [red, green, blue]
}

pub(super) fn tones(app: &App) -> Tones {
    let palette = &app.theme.palette;
    let colours = paper_control::Palette {
        text: rgb(palette.surface_text),
        accent: rgb(palette.primary),
        shade: rgb(palette.surface),
    };
    ink(&colours, paper_widgets::luminance(colours.shade))
}

fn family(id: &str) -> Family {
    if id.starts_with(TYPE_PREFIX) {
        Family::Types
    } else if id.starts_with(SORT_PREFIX) {
        Family::Sort
    } else if id.starts_with(COLOR_PREFIX) {
        Family::Colours
    } else {
        Family::Tools
    }
}

fn available(app: &App) -> BarModel {
    let mut all = BarShow::all();
    all.resolution_presets = app.config.resolution_presets();
    let session = &app.library_session;
    build_bar_with_tasks(
        &session.filters,
        &session.folder_options,
        false,
        session.visible_count,
        session.library.catalog().items.len(),
        app.config.ui_scale(),
        app.config.wallhaven_enabled() || app.config.steam_enabled(),
        app.panels.audio_active,
        !app.panels.audio_playing,
        &all,
        app.scene.viewport.0,
        false,
        false,
        None,
        &[],
    )
}

fn shelves(app: &App, shown: &[String], known: Option<&[Shelf]>) -> Vec<Shelf> {
    if let Some(known) = known {
        return known
            .iter()
            .map(|shelf| Shelf {
                family: shelf.family,
                beads: shelf
                    .beads
                    .iter()
                    .map(|bead| Bead { shown: shown.contains(&bead.id), ..bead.clone() })
                    .collect(),
            })
            .collect();
    }
    let model = available(app);
    let mut items: Vec<(String, String)> = model
        .items
        .iter()
        .filter(|item| item.hideable())
        .filter_map(|item| item.edit_id().map(|id| (id, item.edit_title())))
        .collect();
    let swatch = crate::app::swatch_available(&app.config);
    items.extend(
        Target::ALL
            .into_iter()
            .filter(|target| swatch || *target != Target::Swatch)
            .filter_map(|target| Some((target.edit_id()?, target.title_key()?)))
            .map(|(id, title)| (id.to_owned(), crate::i18n::tr(title).to_owned())),
    );
    FAMILIES
        .into_iter()
        .map(|kind| Shelf {
            family: kind,
            beads: items
                .iter()
                .filter(|(id, _)| family(id) == kind)
                .map(|(id, title)| Bead {
                    id: id.clone(),
                    title: title.clone(),
                    shown: shown.contains(id),
                })
                .collect(),
        })
        .filter(|shelf| !shelf.beads.is_empty())
        .collect()
}

fn corner_label(corners: &Corners) -> CornerStep {
    if corners.style != CornerStyle::Straight && !corners.same {
        return CornerStep::Each;
    }
    CornerStep::Step(corner_step(corners))
}

pub(super) fn board(app: &App, known: Option<&[Shelf]>) -> Board {
    let (width, height) = app.scene.viewport;
    let (model, bar) = crate::app::view::filter_bar_target(app, width, height);
    let buttons: Vec<Button> = model
        .items
        .iter()
        .filter_map(|item: &BarItem| {
            item.edit_id().map(|id| Button {
                id,
                rect: Rect::new(bar.x + item.x, bar.y + item.y, item.w, item.h),
            })
        })
        .collect();
    let present = Target::ALL.map(|target| placement::present(&app.config, target));
    let floating = Target::ALL
        .into_iter()
        .filter(|target| present[target.index()])
        .filter_map(Target::edit_id)
        .map(str::to_owned);
    let shown: Vec<String> =
        buttons.iter().map(|button| button.id.clone()).chain(floating).collect();
    let attached = Target::ALL.map(|target| {
        app.config.position(placement::component(target)).is_some_and(|position| {
            position.reference == skwd_config::picker_layout::Reference::Picker
        })
    });
    let mode = app.scene.mode;
    let gridded = mode == Mode::Grid;
    let style = Style {
        layout: layout_name(mode.as_key()),
        spacing: gridded.then(|| step(&SPACING_STEPS, app.config.num_path(spacing_key(mode)))),
        corners: mode
            .corners()
            .filter(|_| gridded || mode == Mode::Depth)
            .map(|keys| corner_label(&app.config.corners(keys))),
    };
    let grid = gridded.then(|| {
        let (columns, rows) = app.scene.grid_capacity();
        Grid {
            cell: [app.config.grid_thumb_width(), app.config.grid_thumb_height()],
            gap: [app.config.grid_gap_x(), app.config.grid_gap_y()],
            cells: [columns, rows],
        }
    });
    let session = app.panels.arrange.as_ref();
    let prompt = session.and_then(|session| {
        if session.conflict.is_some() {
            Some(Prompt::Conflict)
        } else if session.disconnected_outputs.is_empty() {
            None
        } else {
            Some(Prompt::Disconnected(session.disconnected_outputs.clone()))
        }
    });
    let copy = session.and_then(|session| session.display_copy.as_ref()).map(|copy| Copy {
        recipients: copy.recipients.clone(),
        review: copy.review.as_ref().map(|review| review.view.clone()),
    });
    Board {
        rects: [
            rect(picker_rect(app)),
            rect(bar),
            rect(search_rect(app)),
            rect(placement::rect(app, Target::Arrange)),
            rect(placement::rect(app, Target::Swatch)),
        ],
        attached,
        present,
        buttons,
        shelves: shelves(app, &shown, known),
        style,
        grid,
        steps: session.map_or(0, |session| session.history.index()),
        displays: app.config.layout_output().is_some(),
        copy,
        prompt,
    }
}
