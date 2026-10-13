use std::collections::HashMap;

use iced::widget::canvas::{self, Frame, Path, Stroke};
use iced::widget::{button, column, container, stack, text};
use iced::{Alignment, Element, Length, Point, Rectangle};

use super::super::{
    ActionId, ProfileEntry, ProfilesPage, SettingsMsg, TreeNode, mark_folds, workbench_input_id,
};
use super::control::{Reveals, button_width, fixed_button};
use super::page::{
    NAME_SIZE, Paint, ROW_SPACING, SECTION_GAP, TAG_SIZE, action, input, muted, note, rule, tag,
    text_color,
};
use crate::app::Message;
use crate::contracts::settings::{ProfileState, keys};
use crate::frontend::animation::smoothstep;
use crate::frontend::components::with_alpha;
use crate::frontend::theme::Palette;
use crate::frontend::ui::{TYPE_SMALL, label, legible_type_scale, row};
use crate::i18n::{tr, tr_args};

const LIST_WIDTH: f32 = 250.0;
const LIST_ROW_HEIGHT: f32 = 42.0;
const LIST_ROW_GAP: f32 = 2.0;
const LIST_PADDING: f32 = 10.0;
const MARKER_WIDTH: f32 = 14.0;
const MARKER_RADIUS: f32 = 4.5;
const MARKER_STROKE: f32 = 1.5;
const MARKER_GUIDE_ALPHA: f32 = 0.3;
const PANE_GAP: f32 = 30.0;
const PANE_SLIDE: f32 = 14.0;
const HEADING_SIZE: f32 = 18.0;
const CODE_SIZE: f32 = 11.5;
const CODE_PADDING: [f32; 2] = [9.0, 12.0];
const CODE_FILL_ALPHA: f32 = 0.42;
const CODE_BORDER_ALPHA: f32 = 0.3;
const SEGMENT_WIDTH: f32 = 58.0;
const SEGMENT_HEIGHT: f32 = 26.0;
const SEGMENT_BORDER_ALPHA: f32 = 0.45;
const SEGMENT_IDLE_ALPHA: f32 = 0.7;
const SEGMENT_LOCKED_ALPHA: f32 = 0.55;
const SEGMENT_THUMB_ALPHA: f32 = 0.95;
const TREE_ROW_HEIGHT: f32 = 34.0;
const TREE_INDENT: f32 = 18.0;
const TREE_TWISTY_WIDTH: f32 = 24.0;
const TREE_TAG_ALPHA: f32 = 0.62;
const TREE_LOCKED_ALPHA: f32 = 0.5;
const TREE_NOTE_INDENT: f32 = 30.0;
const HAIRLINE: f32 = crate::frontend::ui::FOLIO_RULE_THICKNESS;
const LEGEND: [(ProfileState, &str); 3] = [
    (ProfileState::Show, "settings-profiles-legend-show"),
    (ProfileState::Hide, "settings-profiles-legend-hide"),
    (ProfileState::Exclude, "settings-profiles-legend-exclude"),
];

pub(super) struct ProfilesView<'a> {
    pub(super) page: ProfilesPage,
    pub(super) filter: &'a str,
    pub(super) enter: f32,
    pub(super) marker: f32,
    pub(super) values: &'a HashMap<String, String>,
    pub(super) reveals: Reveals<'a>,
    pub(super) armed: Option<&'a ActionId>,
}

pub(super) fn profiles_page<'a>(view: ProfilesView<'a>, paint: Paint<'a>) -> Element<'a, Message> {
    let ProfilesView { page, filter, enter, marker, values, reveals, armed } = view;
    let enter = smoothstep(enter.clamp(0.0, 1.0));
    let marker = marker.min(page.profiles.len().saturating_sub(1) as f32);
    let list = profile_list(&page, marker, paint);
    let detail = container(detail_pane(page, filter, values, reveals, armed, paint, enter))
        .width(Length::Fill)
        .padding(iced::Padding::ZERO.left(PANE_SLIDE * paint.scale * (1.0 - enter)));
    row![container(list).width(Length::Fixed(LIST_WIDTH * paint.scale)), detail]
        .spacing(PANE_GAP * paint.scale)
        .align_y(Alignment::Start)
        .into()
}

fn profile_list<'a>(page: &ProfilesPage, marker: f32, paint: Paint<'a>) -> Element<'a, Message> {
    let Paint { scale, palette, fade, .. } = paint;
    let mut items = column![].spacing(LIST_ROW_GAP * scale);
    for (position, entry) in page.profiles.iter().enumerate() {
        items = items.push(profile_item(entry, position, position == page.selected, paint));
    }
    let rail = iced::widget::canvas(ProfileRail { position: marker, palette: *palette, fade })
        .width(Length::Fixed(MARKER_WIDTH * scale))
        .height(Length::Fixed(list_height(page.profiles.len(), scale)));
    let add = action(
        tr("settings-profiles-add-label"),
        Some(Message::Settings(SettingsMsg::Run(ActionId::NewProfile))),
        paint,
    );
    column![
        row![rail, items].spacing(0.0).align_y(Alignment::Start),
        add,
        rule(paint),
        label(tr("settings-profiles-hidden-note-label"), NAME_SIZE, scale, text_color(paint)),
        note(tr("settings-profiles-hidden-note-desc"), paint),
    ]
    .spacing(SECTION_GAP * scale)
    .into()
}

fn list_height(count: usize, scale: f32) -> f32 {
    let rows = count.max(1) as f32;
    (rows * LIST_ROW_HEIGHT + (rows - 1.0) * LIST_ROW_GAP) * scale
}

fn profile_item<'a>(
    entry: &ProfileEntry,
    position: usize,
    selected: bool,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    let Paint { scale, palette, fade, .. } = paint;
    let name = if entry.name.is_empty() { entry.id.clone() } else { entry.name.clone() };
    let mut line = row![
        label(name, NAME_SIZE, scale, text_color(paint)),
        container(text("")).width(Length::Fill)
    ]
    .spacing(ROW_SPACING * scale)
    .align_y(Alignment::Center);
    if entry.active {
        line = line.push(tag(tr("settings-profiles-active-tag"), palette.primary, paint));
    }
    if entry.builtin() {
        line = line.push(tag(tr("settings-profiles-builtin-tag"), palette.surface_text, paint));
    }
    button(container(line).center_y(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fixed(LIST_ROW_HEIGHT * scale))
        .padding([0.0, LIST_PADDING * scale])
        .on_press(Message::Settings(SettingsMsg::SelectProfile(position)))
        .style(move |_theme, status| {
            crate::frontend::ui::folio_line_button_style(selected, false, palette, fade, status)
        })
        .into()
}

struct ProfileRail {
    position: f32,
    palette: Palette,
    fade: f32,
}

impl canvas::Program<Message> for ProfileRail {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let x = bounds.width * 0.5;
        frame.stroke(
            &Path::line(Point::new(x, 0.0), Point::new(x, bounds.height)),
            Stroke::default()
                .with_color(with_alpha(self.palette.outline, MARKER_GUIDE_ALPHA * self.fade))
                .with_width(crate::frontend::ui::FOLIO_RULE_THICKNESS),
        );
        let step = (LIST_ROW_HEIGHT + LIST_ROW_GAP) * bounds.width / MARKER_WIDTH;
        let y = self.position * step + LIST_ROW_HEIGHT * bounds.width / MARKER_WIDTH * 0.5;
        let radius = MARKER_RADIUS * bounds.width / MARKER_WIDTH;
        let diamond = crate::frontend::ui::diamond(Point::new(x, y), radius);
        frame.fill(&diamond, with_alpha(self.palette.background, self.fade));
        frame.stroke(
            &diamond,
            Stroke::default()
                .with_color(with_alpha(self.palette.primary, self.fade))
                .with_width(MARKER_STROKE),
        );
        vec![frame.into_geometry()]
    }
}

fn detail_pane<'a>(
    page: ProfilesPage,
    filter: &'a str,
    values: &'a HashMap<String, String>,
    reveals: Reveals<'a>,
    armed: Option<&ActionId>,
    paint: Paint<'a>,
    enter: f32,
) -> Element<'a, Message> {
    let paint = Paint { fade: paint.fade * enter, ..paint };
    let Paint { scale, palette, fade, motion } = paint;
    let entry = page.current().clone();
    let name: Element<'a, Message> = match entry.name_key() {
        Some(key) => {
            let value = values.get(&key).map_or("", String::as_str);
            let message_key = key.clone();
            input(tr("settings-profiles-name-placeholder"), value, paint)
                .id(workbench_input_id(&key))
                .on_input(move |raw| {
                    Message::Settings(SettingsMsg::Input(message_key.clone(), raw))
                })
                .on_submit(Message::Settings(SettingsMsg::Commit))
                .into()
        }
        None => label(entry.name.clone(), NAME_SIZE, scale, text_color(paint)).into(),
    };
    let activate_label = if entry.active {
        tr("settings-profiles-active-tag")
    } else {
        tr("settings-profiles-make-active")
    };
    let activate = fixed_button(
        activate_label.into(),
        entry.active,
        false,
        (!entry.active).then(|| {
            Message::Settings(SettingsMsg::Pick(
                keys::library::PROFILE.to_owned(),
                entry.id.clone(),
            ))
        }),
        false,
        scale,
        palette,
        fade,
        button_width(tr("settings-profiles-make-active"), scale)
            .max(button_width(tr("settings-profiles-active-tag"), scale)),
        motion,
    );
    let mut pane = column![
        column![
            label(tr("settings-profiles-name-label"), TYPE_SMALL, scale, muted(paint)),
            row![name, activate].spacing(ROW_SPACING * 2.0 * scale).align_y(Alignment::Center),
        ]
        .spacing(ROW_SPACING * scale),
        rule(paint),
        row![
            label(tr("settings-profiles-tree-label"), HEADING_SIZE, scale, text_color(paint)),
            container(text("")).width(Length::Fill),
            label(page.summary.clone(), TYPE_SMALL, scale, with_alpha(palette.primary, fade)),
        ]
        .spacing(ROW_SPACING * scale)
        .align_y(Alignment::End),
        legend(paint),
        note(
            if entry.builtin() {
                tr("settings-profiles-everything-desc")
            } else {
                tr("settings-profiles-tree-desc")
            },
            paint,
        ),
        input(tr("settings-profiles-filter-placeholder"), filter, paint)
            .on_input(|raw| Message::Settings(SettingsMsg::ProfileFilter(raw))),
        tree(page.tree, !filter.trim().is_empty(), entry.builtin(), reveals, paint),
    ]
    .spacing(SECTION_GAP * scale);
    if page.no_match {
        pane = pane.push(note(tr("settings-profiles-no-match"), paint));
    }
    pane = pane.push(note(tr("settings-profiles-tree-note"), paint));
    if let Some(index) = entry.index {
        pane = pane.push(switch_section(&entry, index, armed, paint));
    }
    pane.into()
}

fn switch_section<'a>(
    entry: &ProfileEntry,
    index: usize,
    armed: Option<&ActionId>,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    let Paint { scale, palette, fade, motion } = paint;
    let mut section = column![
        rule(paint),
        label(tr("settings-profiles-switch-label"), HEADING_SIZE, scale, text_color(paint)),
        note(tr("settings-profiles-switch-desc"), paint),
    ]
    .spacing(SECTION_GAP * scale);
    let [helm, flag] = super::super::profile_commands(&entry.id);
    for (command, helm_command, desc) in [
        (helm, true, tr("settings-profiles-helm-desc")),
        (flag, false, tr("settings-profiles-flag-desc")),
    ] {
        let code = container(
            text(command)
                .font(iced::Font::MONOSPACE)
                .size(CODE_SIZE * legible_type_scale(scale))
                .color(text_color(paint)),
        )
        .width(Length::Fill)
        .padding([CODE_PADDING[0] * scale, CODE_PADDING[1] * scale])
        .style(move |_| {
            crate::frontend::ui::box_style(
                with_alpha(iced::Color::BLACK, CODE_FILL_ALPHA * fade),
                with_alpha(palette.outline, CODE_BORDER_ALPHA * fade),
            )
        });
        let copy = action(
            tr("settings-profiles-copy-action"),
            Some(Message::Settings(SettingsMsg::Run(ActionId::CopyProfileCommand(
                index as u16,
                helm_command,
            )))),
            paint,
        );
        section = section.push(
            column![
                row![code, copy].spacing(ROW_SPACING * scale).align_y(Alignment::Center),
                note(desc, paint)
            ]
            .spacing(ROW_SPACING * scale),
        );
    }
    let delete_id = ActionId::DeleteProfile(index as u16);
    let confirming = armed == Some(&delete_id);
    let delete_label = tr("settings-profiles-delete-action");
    let confirmation = tr_args!("settings-control-confirm", label => delete_label);
    let mut delete_row = row![fixed_button(
        if confirming { confirmation.clone() } else { delete_label.to_owned() },
        confirming,
        confirming,
        Some(Message::Settings(SettingsMsg::Run(delete_id))),
        false,
        scale,
        palette,
        fade,
        button_width(&confirmation, scale).max(button_width(delete_label, scale)),
        motion,
    )]
    .spacing(ROW_SPACING * 2.0 * scale)
    .align_y(Alignment::Center);
    if confirming {
        delete_row = delete_row.push(label(
            tr("settings-profiles-delete-hint"),
            TYPE_SMALL,
            scale,
            with_alpha(palette.primary, fade),
        ));
    }
    section.push(delete_row).into()
}

fn legend(paint: Paint<'_>) -> Element<'_, Message> {
    let Paint { scale, palette, fade, .. } = paint;
    let mut line = row![].spacing(SECTION_GAP * scale);
    for (state, key) in LEGEND {
        line = line.push(
            row![
                label(state_label(state), TYPE_SMALL, scale, thumb_color(state, palette, fade)),
                note(tr(key), paint),
            ]
            .spacing(ROW_SPACING * 0.5 * scale),
        );
    }
    line.into()
}

fn state_label(state: ProfileState) -> &'static str {
    super::super::STATES
        .iter()
        .find(|(candidate, _)| *candidate == state)
        .map_or("", |(_, key)| tr(key))
}

fn thumb_color(state: ProfileState, palette: &Palette, fade: f32) -> iced::Color {
    match state {
        ProfileState::Show => with_alpha(palette.primary, fade),
        ProfileState::Hide => with_alpha(palette.tertiary, fade),
        ProfileState::Exclude => with_alpha(palette.outline, fade),
    }
}

fn tree<'a>(
    mut nodes: Vec<TreeNode>,
    filtering: bool,
    locked: bool,
    reveals: Reveals<'a>,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    if !filtering {
        mark_folds(&mut nodes, reveals.folds);
    }
    let amounts: Vec<f32> = nodes
        .iter()
        .map(|node| {
            let settled = if node.collapsed { 0.0 } else { 1.0 };
            if filtering {
                return 1.0;
            }
            reveals.bars.get(&node.id).map_or(settled, |tween| tween.x)
        })
        .collect();
    let (block, _) = tree_block(&nodes, &amounts, 0, nodes.len(), locked, reveals, paint);
    block.into()
}

fn tree_block<'a>(
    nodes: &[TreeNode],
    amounts: &[f32],
    start: usize,
    end: usize,
    locked: bool,
    reveals: Reveals<'a>,
    paint: Paint<'a>,
) -> (iced::widget::Column<'a, Message>, f32) {
    let scale = paint.scale;
    let mut block = column![].spacing(0.0);
    let mut rows = 0.0;
    let mut index = start;
    while index < end {
        let node = &nodes[index];
        block = block.push(tree_row(node, locked, reveals, paint));
        rows += 1.0;
        if node.id.ends_with(super::super::WORKSHOP_TAG) {
            block = block.push(
                container(note(tr("settings-profiles-workshop-desc"), paint))
                    .padding(iced::Padding::ZERO.left(TREE_NOTE_INDENT * scale)),
            );
        }
        let sub_end = (index + 1..end).find(|next| nodes[*next].depth <= node.depth).unwrap_or(end);
        if sub_end > index + 1 {
            let amount = smoothstep(amounts[index]);
            if amount > 0.0 {
                let (inner, inner_rows) =
                    tree_block(nodes, amounts, index + 1, sub_end, locked, reveals, paint);
                let height = inner_rows * TREE_ROW_HEIGHT * scale * amount;
                block = block.push(
                    container(inner).width(Length::Fill).height(Length::Fixed(height)).clip(true),
                );
                rows += inner_rows * amount;
            }
        }
        index = sub_end;
    }
    (block, rows)
}

fn tree_row<'a>(
    node: &TreeNode,
    locked: bool,
    reveals: Reveals<'a>,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    let Paint { scale, palette, fade, .. } = paint;
    let indent = container(text(""))
        .width(Length::Fixed(TREE_INDENT * scale * f32::from(node.depth)))
        .height(Length::Fixed(HAIRLINE));
    let twisty: Element<'a, Message> = if node.children {
        let glyph = if node.collapsed {
            crate::frontend::ui::TWISTY_CLOSED
        } else {
            crate::frontend::ui::TWISTY_OPEN
        };
        button(container(label(glyph, TYPE_SMALL, scale, text_color(paint))).center(Length::Fill))
            .width(Length::Fixed(TREE_TWISTY_WIDTH * scale))
            .height(Length::Fixed(TREE_ROW_HEIGHT * scale))
            .padding(0)
            .on_press(Message::Settings(SettingsMsg::ToggleFold(
                node.id.clone(),
                node.open_by_default(),
            )))
            .style(button::text)
            .into()
    } else {
        container(text("")).width(Length::Fixed(TREE_TWISTY_WIDTH * scale)).into()
    };
    let name_color = match node.state {
        None => with_alpha(palette.surface_text, TREE_LOCKED_ALPHA * fade),
        Some(ProfileState::Hide) => with_alpha(palette.tertiary, fade),
        Some(ProfileState::Exclude) => with_alpha(palette.surface_text, TREE_TAG_ALPHA * fade),
        Some(ProfileState::Show) => with_alpha(palette.surface_text, fade),
    };
    let mut line = row![indent, twisty, label(node.label.clone(), TYPE_SMALL, scale, name_color)]
        .spacing(ROW_SPACING * scale)
        .align_y(Alignment::Center);
    let tag_text = if node.state.is_none() {
        Some(tr("settings-profiles-never-scanned").to_owned())
    } else if node.collapsed && node.hidden_inside > 0 {
        Some(tr_args!("settings-profiles-hidden-inside", count => node.hidden_inside))
    } else if node.mixed {
        Some(tr("settings-profiles-mixed").to_owned())
    } else {
        None
    };
    if let Some(tag_text) = tag_text {
        line = line.push(label(
            tag_text,
            TAG_SIZE,
            scale,
            with_alpha(palette.primary, TREE_TAG_ALPHA * fade),
        ));
    }
    line = line.push(container(text("")).width(Length::Fill));
    if let Some(state) = node.state {
        let position = reveals
            .bars
            .get(&format!("{}{}", super::super::SEGMENT_PREFIX, node.id))
            .map_or(state_position(state), |tween| tween.x);
        line =
            line.push(segment(&node.path, state, position, locked || node.path.is_empty(), paint));
    }
    container(line)
        .width(Length::Fill)
        .height(Length::Fixed(TREE_ROW_HEIGHT * scale))
        .align_y(Alignment::Center)
        .into()
}

pub fn state_position(state: ProfileState) -> f32 {
    super::super::STATES
        .iter()
        .position(|(candidate, _)| *candidate == state)
        .map_or(0.0, |index| index as f32)
}

fn segment<'a>(
    path: &str,
    state: ProfileState,
    position: f32,
    locked: bool,
    paint: Paint<'a>,
) -> Element<'a, Message> {
    let Paint { scale, palette, fade, .. } = paint;
    let fade = if locked { fade * SEGMENT_LOCKED_ALPHA } else { fade };
    let width = SEGMENT_WIDTH * scale;
    let thumb = iced::widget::canvas(SegmentThumb {
        position,
        color: thumb_color(state, palette, SEGMENT_THUMB_ALPHA * fade),
        border: with_alpha(palette.outline, SEGMENT_BORDER_ALPHA * fade),
    })
    .width(Length::Fixed(width * super::super::STATES.len() as f32))
    .height(Length::Fixed(SEGMENT_HEIGHT * scale));
    let mut options = row![].spacing(0.0);
    for (candidate, key) in super::super::STATES {
        let active = candidate == state;
        let color = if active {
            with_alpha(palette.background, fade)
        } else {
            with_alpha(palette.surface_text, SEGMENT_IDLE_ALPHA * fade)
        };
        let mut option =
            button(container(label(tr(key), TYPE_SMALL, scale, color)).center(Length::Fill))
                .width(Length::Fixed(width))
                .height(Length::Fixed(SEGMENT_HEIGHT * scale))
                .padding(0)
                .style(button::text);
        if !locked && !active {
            option = option.on_press(Message::Settings(SettingsMsg::Pick(
                path.to_owned(),
                candidate.key().to_owned(),
            )));
        }
        options = options.push(option);
    }
    stack![thumb, options].into()
}

struct SegmentThumb {
    position: f32,
    color: iced::Color,
    border: iced::Color,
}

impl canvas::Program<Message> for SegmentThumb {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let width = bounds.width / super::super::STATES.len() as f32;
        frame.fill_rectangle(
            Point::new(self.position * width, 0.0),
            iced::Size::new(width, bounds.height),
            self.color,
        );
        frame.stroke(
            &Path::rectangle(
                Point::new(HAIRLINE * 0.5, HAIRLINE * 0.5),
                iced::Size::new(bounds.width - HAIRLINE, bounds.height - HAIRLINE),
            ),
            Stroke::default().with_color(self.border).with_width(HAIRLINE),
        );
        vec![frame.into_geometry()]
    }
}
