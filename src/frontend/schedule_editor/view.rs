use iced::widget::{button, column, container, mouse_area, scrollable, text};
use iced::{Alignment, Background, Element, Length, Padding, mouse};

use crate::app::Message;
use crate::domain::schedule::{
    Block, ConditionKind, ConditionNode, GroupOperator, RuleRow, WEATHER, WEEKDAYS,
};
use crate::frontend::schedule_editor::{BLOCK_KINDS, Editing, SchedMsg, ScheduleEditor};
use crate::frontend::theme::Palette;
use crate::frontend::ui::{
    UI_FONT, folio_horizontal_rule, label, legible_type_scale, logical_padding, row, with_alpha,
};
use crate::i18n::{tr, tr_args};

const SCHEDULE_INDEX_WIDTH: f32 = 360.0;
const SCHEDULE_ROOMY_SCALE: f32 = 1.18;
const SCHEDULE_ROOMY_MIN_WIDTH: f32 = 1280.0;
const SCHEDULE_ROOMY_MIN_HEIGHT: f32 = 760.0;
const SCHEDULE_DEFINITION_SHARE: f32 = 0.38;

fn schedule_scale(viewport: (f32, f32), scale: f32) -> f32 {
    if viewport.0 >= SCHEDULE_ROOMY_MIN_WIDTH && viewport.1 >= SCHEDULE_ROOMY_MIN_HEIGHT {
        scale * SCHEDULE_ROOMY_SCALE
    } else {
        scale
    }
}

fn vertical_divider<'a>(palette: &Palette) -> Element<'a, Message> {
    let color = with_alpha(palette.outline, 0.36);
    container(text(""))
        .width(Length::Fixed(1.0))
        .height(Length::Fill)
        .style(move |_| crate::frontend::ui::bg_style(Background::Color(color)))
        .into()
}

fn schedule_section<'a>(
    title: &'a str,
    description: &'a str,
    body: Element<'a, Message>,
    scale: f32,
    palette: &'a Palette,
) -> Element<'a, Message> {
    container(
        column![
            folio_horizontal_rule(with_alpha(palette.outline, 0.56)),
            label(title, 14.0, scale, palette.surface_text),
            label(
                description,
                crate::frontend::ui::TYPE_SMALL,
                scale,
                with_alpha(palette.surface_text, 0.5)
            )
            .line_height(iced::widget::text::LineHeight::Relative(1.4)),
            body,
        ]
        .spacing(11.0 * scale),
    )
    .width(Length::Fill)
    .padding(Padding { top: 0.0, right: 9.0 * scale, bottom: 8.0 * scale, left: 9.0 * scale })
    .into()
}

fn action<'a>(
    label: impl text::IntoFragment<'a>,
    label_width: &str,
    active: bool,
    message: Message,
    scale: f32,
    palette: &'a Palette,
) -> Element<'a, Message> {
    crate::frontend::ui::folio_action(
        label,
        active,
        Some(message),
        Length::Fixed(crate::frontend::ui::folio_action_width(label_width, scale)),
        scale * 0.9,
        palette,
    )
}

fn block_kind_name(block: &Block) -> &'static str {
    match block {
        Block::Weekday(_) => tr("schedule-kind-weekday"),
        Block::TimeWindow(..) => tr("schedule-kind-time-range"),
        Block::TimeCmp(op, _) if matches!(op.as_str(), ">=" | ">") => {
            tr("schedule-kind-after-time")
        }
        Block::TimeCmp(..) => tr("schedule-kind-before-time"),
        Block::Date(_) => tr("schedule-kind-date"),
        Block::Year(..) => tr("schedule-kind-year"),
        Block::Weather(_) => tr("schedule-kind-weather"),
        Block::Power(_) => tr("schedule-kind-power"),
        Block::Battery(..) => tr("schedule-kind-battery"),
        Block::Output(_) => tr("schedule-kind-output"),
        Block::OutputCount(..) => tr("schedule-kind-output-count"),
        Block::Raw(_) => tr("schedule-kind-unsupported"),
    }
}

fn weekday_label(day: &str) -> &str {
    match day {
        "mon" => tr("schedule-day-mon"),
        "tue" => tr("schedule-day-tue"),
        "wed" => tr("schedule-day-wed"),
        "thu" => tr("schedule-day-thu"),
        "fri" => tr("schedule-day-fri"),
        "sat" => tr("schedule-day-sat"),
        "sun" => tr("schedule-day-sun"),
        _ => day,
    }
}

fn weather_label(tag: &str) -> &str {
    match tag {
        "clear" => tr("schedule-weather-clear"),
        "sunny" => tr("schedule-weather-sunny"),
        "cloudy" => tr("schedule-weather-cloudy"),
        "rainy" => tr("schedule-weather-rainy"),
        "snowy" => tr("schedule-weather-snowy"),
        "stormy" => tr("schedule-weather-stormy"),
        "foggy" => tr("schedule-weather-foggy"),
        "windy" => tr("schedule-weather-windy"),
        _ => tag,
    }
}

fn time_label(value: &str) -> String {
    for (name, label_key) in [("sunrise", "schedule-sunrise"), ("sunset", "schedule-sunset")] {
        if let Some(rest) = value.strip_prefix(name) {
            let Ok(offset) = rest.parse::<i32>() else {
                return tr(label_key).to_string();
            };
            if rest.is_empty() || offset == 0 {
                return tr(label_key).to_string();
            }
            return if offset < 0 {
                tr_args!(
                    "schedule-solar-before",
                    minutes => offset.unsigned_abs(),
                    solar => tr(label_key),
                )
            } else {
                tr_args!(
                    "schedule-solar-after",
                    minutes => offset.unsigned_abs(),
                    solar => tr(label_key),
                )
            };
        }
    }
    value.to_string()
}

fn comparison_label(op: &str) -> &'static str {
    match op {
        ">=" => tr("schedule-relation-at-least"),
        "<=" => tr("schedule-relation-at-most"),
        ">" => tr("schedule-relation-more-than"),
        "<" => tr("schedule-relation-fewer-than"),
        _ => tr("schedule-relation-exactly"),
    }
}

fn title_case(value: &str) -> String {
    let mut chars = value.chars();
    chars.next().map_or_else(String::new, |first| first.to_uppercase().chain(chars).collect())
}

fn friendly_list(values: &[String]) -> String {
    match values {
        [] => tr("schedule-list-nothing").to_string(),
        [value] => title_case(value),
        [first, second] => {
            tr_args!("schedule-list-pair", first => title_case(first), second => title_case(second))
        }
        values => {
            let mut parts = values.iter().map(|value| title_case(value)).collect::<Vec<_>>();
            let last = parts.pop().unwrap_or_default();
            tr_args!("schedule-list-many", items => parts.join(", "), last => &last)
        }
    }
}

fn labelled_list(values: &[String], value_label: fn(&str) -> &str) -> String {
    let labels: Vec<String> = values.iter().map(|value| value_label(value).to_string()).collect();
    friendly_list(&labels)
}

fn condition_sentence(block: &Block) -> String {
    match block {
        Block::Weekday(days) => {
            tr_args!("schedule-sentence-day", list => labelled_list(days, weekday_label))
        }
        Block::TimeWindow(from, to) => {
            let from = time_label(from);
            let to = time_label(to);
            tr_args!("schedule-sentence-time-between", from => &from, to => &to)
        }
        Block::TimeCmp(op, at) => {
            let at = time_label(at);
            tr_args!(
                "schedule-sentence-time-cmp",
                relation => if matches!(op.as_str(), ">=" | ">") {
                    tr("schedule-relation-after")
                } else {
                    tr("schedule-relation-before")
                },
                at => &at,
            )
        }
        Block::Date(date) => tr_args!(
            "schedule-sentence-date",
            date => date.replace("..", &format!(" {} ", tr("schedule-date-range-joiner"))),
        ),
        Block::Year(op, year) => {
            let relation = match op.as_str() {
                ">=" => tr("schedule-relation-at-least"),
                "<=" => tr("schedule-relation-at-most"),
                ">" => tr("schedule-relation-after"),
                "<" => tr("schedule-relation-before"),
                _ => tr("schedule-relation-exactly"),
            };
            tr_args!("schedule-sentence-year", relation => relation, year => year)
        }
        Block::Weather(tags) => {
            tr_args!("schedule-sentence-weather", list => labelled_list(tags, weather_label))
        }
        Block::Power(source) => tr_args!(
            "schedule-sentence-power",
            source => if source == "battery" {
                tr("schedule-power-battery")
            } else {
                tr("schedule-power-external")
            },
        ),
        Block::Battery(op, percent) => tr_args!(
            "schedule-sentence-battery",
            relation => comparison_label(op),
            percent => percent,
        ),
        Block::Output(name) => tr_args!("schedule-sentence-output", output => name),
        Block::OutputCount(op, count) => tr_args!(
            "schedule-sentence-output-count",
            relation => comparison_label(op),
            count => count,
        ),
        Block::Raw(raw) if raw.starts_with("condition:") => {
            tr("schedule-sentence-needs-setup").to_string()
        }
        Block::Raw(raw) => tr_args!("schedule-sentence-unsupported", raw => raw),
    }
}

fn invalid_condition(node: &ConditionNode) -> bool {
    match &node.kind {
        ConditionKind::Predicate(Block::Raw(raw)) => raw.starts_with("condition:"),
        ConditionKind::Predicate(_) => false,
        ConditionKind::Group { children, .. } => children.iter().any(invalid_condition),
    }
}

fn path_message(path: &[usize]) -> Vec<u16> {
    path.iter().map(|index| *index as u16).collect()
}

fn expression_count(node: &ConditionNode) -> usize {
    match &node.kind {
        ConditionKind::Predicate(_) => 1,
        ConditionKind::Group { children, .. } => children.iter().map(expression_count).sum(),
    }
}

fn condition_summary(rule: &RuleRow) -> String {
    if invalid_condition(&rule.condition) {
        return tr("schedule-summary-needs-setup").to_string();
    }
    match &rule.condition.kind {
        ConditionKind::Predicate(block) => {
            let sentence = condition_sentence(block);
            if rule.condition.negated {
                tr_args!("schedule-summary-negated", summary => &sentence)
            } else {
                sentence
            }
        }
        ConditionKind::Group { operator, children } if children.is_empty() => {
            match (operator, rule.condition.negated) {
                (GroupOperator::All, false) | (GroupOperator::Any, true) => {
                    tr("schedule-summary-always").to_string()
                }
                _ => tr("schedule-summary-never").to_string(),
            }
        }
        ConditionKind::Group { operator, .. } => {
            let count = expression_count(&rule.condition);
            let summary = crate::i18n::schedule_group_summary(
                match operator {
                    GroupOperator::All => tr("schedule-summary-match-all"),
                    GroupOperator::Any => tr("schedule-summary-match-any"),
                },
                count,
            );
            if rule.condition.negated {
                tr_args!("schedule-summary-negated", summary => &summary)
            } else {
                summary
            }
        }
    }
}

fn rule_name(rule: &RuleRow) -> &str {
    if rule.name.trim().is_empty() { tr("schedule-unnamed-rule") } else { rule.name.trim() }
}

impl ScheduleEditor {
    fn priority_index<'a>(&'a self, scale: f32, palette: &'a Palette) -> Element<'a, Message> {
        let mut items = column![].spacing(4.0 * scale);
        for (position, rule) in self.rows.iter().enumerate() {
            let selected = self.selected == position;
            let dragging = self.drag == Some(position);
            let rule_enabled = rule.enabled;
            let detail = crate::frontend::ui::ellipsize_text(
                &format!(
                    "{}  ·  {}",
                    condition_summary(rule),
                    if rule.set.trim().is_empty() {
                        tr("schedule-random-wallpaper")
                    } else {
                        rule.set.trim()
                    }
                ),
                8.5 * legible_type_scale(scale),
                190.0 * scale,
            );
            let content = column![
                label(
                    crate::frontend::ui::ellipsize_text(
                        rule_name(rule),
                        12.0 * legible_type_scale(scale),
                        190.0 * scale,
                    ),
                    12.0,
                    scale,
                    if rule_enabled {
                        palette.surface_text
                    } else {
                        with_alpha(palette.surface_text, 0.5)
                    },
                ),
                label(
                    detail,
                    8.5,
                    scale,
                    with_alpha(palette.surface_text, if rule_enabled { 0.44 } else { 0.28 }),
                ),
            ]
            .spacing(3.0 * scale)
            .width(Length::Fill);
            let drag = container(
                mouse_area(
                    container(label(
                        "≡",
                        11.0,
                        scale,
                        if dragging {
                            palette.primary
                        } else {
                            with_alpha(palette.surface_text, 0.58)
                        },
                    ))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center(Length::Fill),
                )
                .on_press(Message::Sched(SchedMsg::DragStart(position as u16)))
                .interaction(mouse::Interaction::Grab),
            )
            .width(Length::Fixed(26.0 * scale))
            .height(Length::Fill);
            let options = row![
                label(tr("schedule-rule-state"), 8.0, scale, with_alpha(palette.primary, 0.82)),
                container(text("")).width(Length::Fill),
                crate::frontend::ui::folio_action(
                    if rule_enabled { tr("schedule-rule-on") } else { tr("schedule-rule-off") },
                    self.enabled && rule_enabled,
                    Some(Message::Sched(SchedMsg::SetRuleEnabled(position as u16, !rule_enabled,))),
                    Length::Fixed(58.0 * scale),
                    scale * 0.78,
                    palette,
                ),
            ]
            .spacing(8.0 * scale)
            .align_y(Alignment::Center);
            let rule_row = crate::frontend::ui::folio_stack_bar(
                content.into(),
                selected || dragging,
                self.foldout == Some(position),
                if self.foldout_visible == Some(position) { self.foldout_reveal.x } else { 0.0 },
                Message::Sched(SchedMsg::ToggleRuleOptions(position as u16)),
                Some(drag.into()),
                options.into(),
                42.0,
                45.0,
                scale,
                palette,
            );
            items = items.push(
                mouse_area(rule_row)
                    .on_enter(Message::Sched(SchedMsg::DragOver(position as u16)))
                    .on_release(Message::Sched(SchedMsg::DragEnd)),
            );
        }

        if self.rows.is_empty() {
            items = items.push(
                container(
                    column![
                        label(tr("schedule-index-empty-title"), 13.0, scale, palette.surface_text),
                        label(
                            tr("schedule-index-empty-hint"),
                            9.5,
                            scale,
                            with_alpha(palette.surface_text, 0.46),
                        ),
                    ]
                    .spacing(5.0 * scale),
                )
                .width(Length::Fill)
                .padding(14.0 * scale),
            );
        }

        let create = column![
            folio_horizontal_rule(with_alpha(palette.outline, 0.5)),
            label(tr("schedule-new-rule"), 9.0, scale, palette.primary),
            crate::frontend::ui::folio_action(
                tr("schedule-create-rule"),
                false,
                Some(Message::Sched(SchedMsg::AddRule)),
                Length::Fill,
                scale,
                palette,
            ),
        ]
        .spacing(9.0 * scale);

        let heading = column![
            label(tr("schedule-index-title"), 23.0, scale, palette.surface_text),
            label(
                tr("schedule-index-subtitle"),
                11.0,
                scale,
                with_alpha(palette.surface_text, 0.58),
            )
            .line_height(iced::widget::text::LineHeight::Relative(1.45)),
        ]
        .spacing(8.0 * scale);
        let state = column![
            folio_horizontal_rule(with_alpha(palette.outline, 0.5)),
            label(tr("schedule-state-title"), 9.0, scale, palette.primary),
            crate::frontend::ui::folio_action_wrap(
                vec![
                    (
                        tr("schedule-state-enabled").to_string(),
                        self.enabled,
                        Message::Sched(SchedMsg::SetEnabled(true)),
                    ),
                    (
                        tr("schedule-state-disabled").to_string(),
                        !self.enabled,
                        Message::Sched(SchedMsg::SetEnabled(false)),
                    ),
                ],
                280.0 * scale,
                scale,
                palette,
            ),
            label(
                if self.enabled {
                    tr("schedule-state-enabled-desc")
                } else {
                    tr("schedule-state-disabled-desc")
                },
                9.0,
                scale,
                with_alpha(palette.surface_text, 0.5),
            )
            .line_height(iced::widget::text::LineHeight::Relative(1.35)),
        ]
        .spacing(8.0 * scale);
        container(
            column![
                heading,
                state,
                scrollable(items)
                    .direction(crate::frontend::ui::thin_vbar())
                    .style(crate::frontend::ui::scroll_style(with_alpha(palette.outline, 0.72)))
                    .height(Length::Fill),
                create,
            ]
            .spacing(18.0 * scale),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(logical_padding(28.0 * scale, 23.0 * scale, 23.0 * scale, 25.0 * scale))
        .style(move |_| {
            crate::frontend::ui::box_style(
                with_alpha(palette.background, 0.92),
                with_alpha(palette.outline, 0.5),
            )
        })
        .into()
    }

    fn time_endpoint_editor<'a>(
        rule_idx: usize,
        path: &[usize],
        which: u8,
        value: &'a str,
        available: f32,
        scale: f32,
        palette: &'a Palette,
    ) -> Element<'a, Message> {
        let message_path = path_message(path);
        let actions = [("sunrise", tr("schedule-sunrise")), ("sunset", tr("schedule-sunset"))]
            .into_iter()
            .map(|(word, word_label)| {
                (
                    word_label.to_string(),
                    value == word,
                    Message::Sched(SchedMsg::TimePart(
                        rule_idx as u16,
                        message_path.clone(),
                        which,
                        word.to_string(),
                    )),
                )
            })
            .collect();
        column![
            crate::frontend::ui::field_input(
                value,
                tr("schedule-time-placeholder"),
                {
                    let message_path = message_path.clone();
                    move |value| {
                        Message::Sched(SchedMsg::TimePart(
                            rule_idx as u16,
                            message_path.clone(),
                            which,
                            value,
                        ))
                    }
                },
                Message::Sched(SchedMsg::Submit),
                Length::Fixed(142.0 * scale),
                scale,
                palette,
            ),
            crate::frontend::ui::folio_action_wrap(actions, available, scale, palette),
            label(
                tr("schedule-time-hint"),
                crate::frontend::ui::TYPE_SMALL,
                scale,
                with_alpha(palette.surface_text, 0.42),
            ),
        ]
        .spacing(6.0 * scale)
        .into()
    }

    fn block_editor<'a>(
        rule_idx: usize,
        path: &[usize],
        block: &'a Block,
        negated: bool,
        available: f32,
        scale: f32,
        palette: &'a Palette,
    ) -> Element<'a, Message> {
        let message_path = path_message(path);
        let mut body = column![].spacing(9.0 * scale);
        let toggles = |selected: &[String], allowlist: &[&str], value_label: fn(&str) -> &str| {
            let actions = allowlist
                .iter()
                .map(|value| {
                    (
                        value_label(value).to_string(),
                        selected.iter().any(|entry| entry == value),
                        Message::Sched(SchedMsg::ToggleVal(
                            rule_idx as u16,
                            message_path.clone(),
                            (*value).to_string(),
                        )),
                    )
                })
                .collect();
            crate::frontend::ui::folio_action_wrap(actions, available, scale, palette)
        };
        let comparisons = |selected: &str| {
            let actions = [
                ("", tr("schedule-comparison-exact")),
                (">=", tr("schedule-comparison-at-least")),
                ("<=", tr("schedule-comparison-at-most")),
            ]
            .into_iter()
            .map(|(value, action_label)| {
                (
                    action_label.to_string(),
                    selected == value,
                    Message::Sched(SchedMsg::Comparison(
                        rule_idx as u16,
                        message_path.clone(),
                        value.to_string(),
                    )),
                )
            })
            .collect();
            crate::frontend::ui::folio_action_wrap(actions, available, scale, palette)
        };
        match block {
            Block::Weekday(days) => {
                body = body.push(toggles(days, &WEEKDAYS, weekday_label));
            }
            Block::Weather(tags) => {
                body = body.push(toggles(tags, &WEATHER, weather_label)).push(label(
                    tr("schedule-weather-hint"),
                    8.5,
                    scale,
                    with_alpha(palette.surface_text, 0.42),
                ));
            }
            Block::TimeWindow(from, to) => {
                body = body
                    .push(
                        column![
                            label(
                                tr("schedule-from"),
                                9.0,
                                scale,
                                with_alpha(palette.surface_text, 0.52)
                            ),
                            Self::time_endpoint_editor(
                                rule_idx, path, 0, from, available, scale, palette,
                            ),
                        ]
                        .spacing(5.0 * scale),
                    )
                    .push(
                        column![
                            label(
                                tr("schedule-to"),
                                9.0,
                                scale,
                                with_alpha(palette.surface_text, 0.52)
                            ),
                            Self::time_endpoint_editor(
                                rule_idx, path, 1, to, available, scale, palette,
                            ),
                        ]
                        .spacing(5.0 * scale),
                    );
            }
            Block::TimeCmp(op, at) => {
                let actions = [(">=", tr("schedule-cmp-after")), ("<", tr("schedule-cmp-before"))]
                    .into_iter()
                    .map(|(value, label)| {
                        (
                            label.to_string(),
                            matches!((value, op.as_str()), (">=", ">=" | ">") | ("<", "<" | "<=")),
                            Message::Sched(SchedMsg::TimePart(
                                rule_idx as u16,
                                message_path.clone(),
                                3,
                                value.to_string(),
                            )),
                        )
                    })
                    .collect();
                body = body
                    .push(crate::frontend::ui::folio_action_wrap(
                        actions, available, scale, palette,
                    ))
                    .push(Self::time_endpoint_editor(
                        rule_idx, path, 2, at, available, scale, palette,
                    ));
            }
            Block::Date(date) => {
                body = body
                    .push(crate::frontend::ui::field_input(
                        date,
                        tr("schedule-date-placeholder"),
                        {
                            let message_path = message_path.clone();
                            move |value| {
                                Message::Sched(SchedMsg::TextBlock(
                                    rule_idx as u16,
                                    message_path.clone(),
                                    value,
                                ))
                            }
                        },
                        Message::Sched(SchedMsg::Submit),
                        Length::Fill,
                        scale,
                        palette,
                    ))
                    .push(label(
                        tr("schedule-date-hint"),
                        8.5,
                        scale,
                        with_alpha(palette.surface_text, 0.42),
                    ));
            }
            Block::Year(op, year) => {
                let actions = [
                    ("", tr("schedule-year-exact")),
                    (">=", tr("schedule-year-at-least")),
                    ("<=", tr("schedule-year-at-most")),
                    (">", tr("schedule-cmp-after")),
                    ("<", tr("schedule-cmp-before")),
                ]
                .into_iter()
                .map(|(value, label)| {
                    (
                        label.to_string(),
                        op == value,
                        Message::Sched(SchedMsg::TextBlock(
                            rule_idx as u16,
                            message_path.clone(),
                            format!("{value}{year}"),
                        )),
                    )
                })
                .collect();
                let current_op = op.clone();
                body = body
                    .push(crate::frontend::ui::folio_action_wrap(
                        actions, available, scale, palette,
                    ))
                    .push(crate::frontend::ui::field_input(
                        year,
                        "2026",
                        {
                            let message_path = message_path.clone();
                            move |value| {
                                Message::Sched(SchedMsg::TextBlock(
                                    rule_idx as u16,
                                    message_path.clone(),
                                    format!("{current_op}{value}"),
                                ))
                            }
                        },
                        Message::Sched(SchedMsg::Submit),
                        Length::Fixed(120.0 * scale),
                        scale,
                        palette,
                    ));
            }
            Block::Power(source) => {
                let actions = [
                    ("battery", tr("schedule-power-battery")),
                    ("external", tr("schedule-power-external")),
                ]
                .into_iter()
                .map(|(value, action_label)| {
                    (
                        action_label.to_string(),
                        source == value,
                        Message::Sched(SchedMsg::BlockChoice(
                            rule_idx as u16,
                            message_path.clone(),
                            value.to_string(),
                        )),
                    )
                })
                .collect();
                body = body.push(crate::frontend::ui::folio_action_wrap(
                    actions, available, scale, palette,
                ));
            }
            Block::Battery(op, percent) => {
                body = body
                    .push(comparisons(op))
                    .push(crate::frontend::ui::field_input(
                        percent,
                        "30",
                        {
                            let message_path = message_path.clone();
                            move |value| {
                                Message::Sched(SchedMsg::TextBlock(
                                    rule_idx as u16,
                                    message_path.clone(),
                                    value,
                                ))
                            }
                        },
                        Message::Sched(SchedMsg::Submit),
                        Length::Fixed(100.0 * scale),
                        scale,
                        palette,
                    ))
                    .push(label(
                        tr("schedule-battery-hint"),
                        8.5,
                        scale,
                        with_alpha(palette.surface_text, 0.42),
                    ));
            }
            Block::Output(output) => {
                body = body
                    .push(crate::frontend::ui::field_input(
                        output,
                        tr("schedule-output-placeholder"),
                        {
                            let message_path = message_path.clone();
                            move |value| {
                                Message::Sched(SchedMsg::TextBlock(
                                    rule_idx as u16,
                                    message_path.clone(),
                                    value,
                                ))
                            }
                        },
                        Message::Sched(SchedMsg::Submit),
                        Length::Fill,
                        scale,
                        palette,
                    ))
                    .push(label(
                        tr("schedule-output-hint"),
                        8.5,
                        scale,
                        with_alpha(palette.surface_text, 0.42),
                    ));
            }
            Block::OutputCount(op, count) => {
                body = body
                    .push(comparisons(op))
                    .push(crate::frontend::ui::field_input(
                        count,
                        "2",
                        {
                            let message_path = message_path.clone();
                            move |value| {
                                Message::Sched(SchedMsg::TextBlock(
                                    rule_idx as u16,
                                    message_path.clone(),
                                    value,
                                ))
                            }
                        },
                        Message::Sched(SchedMsg::Submit),
                        Length::Fixed(100.0 * scale),
                        scale,
                        palette,
                    ))
                    .push(label(
                        tr("schedule-output-count-hint"),
                        8.5,
                        scale,
                        with_alpha(palette.surface_text, 0.42),
                    ));
            }
            Block::Raw(raw) => {
                body = body
                    .push(crate::frontend::ui::field_input(
                        raw,
                        tr("schedule-raw-placeholder"),
                        {
                            let message_path = message_path.clone();
                            move |value| {
                                Message::Sched(SchedMsg::TextBlock(
                                    rule_idx as u16,
                                    message_path.clone(),
                                    value,
                                ))
                            }
                        },
                        Message::Sched(SchedMsg::Submit),
                        Length::Fill,
                        scale,
                        palette,
                    ))
                    .push(label(
                        tr("schedule-raw-hint"),
                        8.5,
                        scale,
                        with_alpha(palette.surface_text, 0.42),
                    ));
            }
        }
        body = body
            .push(crate::frontend::ui::folio_action_wrap(
                vec![
                    (
                        tr("schedule-must-match").to_string(),
                        !negated,
                        Message::Sched(SchedMsg::SetNegated(
                            rule_idx as u16,
                            message_path.clone(),
                            false,
                        )),
                    ),
                    (
                        tr("schedule-must-not-match").to_string(),
                        negated,
                        Message::Sched(SchedMsg::SetNegated(
                            rule_idx as u16,
                            message_path.clone(),
                            true,
                        )),
                    ),
                ],
                available,
                scale,
                palette,
            ))
            .push(crate::frontend::ui::folio_destructive_action(
                tr("schedule-remove-condition"),
                false,
                Some(Message::Sched(SchedMsg::RemoveNode(rule_idx as u16, message_path))),
                Length::Fixed(150.0 * scale),
                scale * 0.9,
                palette,
            ));
        body.into()
    }

    fn condition_node<'a>(
        &'a self,
        rule_idx: usize,
        node: &'a ConditionNode,
        path: &[usize],
        depth: usize,
        scale: f32,
        palette: &'a Palette,
    ) -> Element<'a, Message> {
        let selected = matches!(
            self.editing.as_ref(),
            Some(Editing::Node(row, selected_path)) if *row == rule_idx && selected_path == path
        );
        let (eyebrow, title, detail) = match &node.kind {
            ConditionKind::Predicate(block) => (
                tr("schedule-node-condition"),
                condition_sentence(block),
                if node.negated {
                    tr("schedule-node-must-not-match").to_string()
                } else {
                    tr("schedule-node-must-match").to_string()
                },
            ),
            ConditionKind::Group { operator, children } => {
                let empty_matches = matches!(operator, GroupOperator::All) != node.negated;
                let title = match (children.is_empty(), empty_matches, operator, node.negated) {
                    (true, true, _, _) => tr("schedule-node-always"),
                    (true, false, _, _) => tr("schedule-node-never"),
                    (false, _, GroupOperator::All, false) => tr("schedule-node-match-all"),
                    (false, _, GroupOperator::Any, false) => tr("schedule-node-match-any"),
                    (false, _, GroupOperator::All, true) => tr("schedule-node-not-all"),
                    (false, _, GroupOperator::Any, true) => tr("schedule-node-match-none"),
                };
                (
                    if path.is_empty() {
                        tr("schedule-node-root")
                    } else {
                        tr("schedule-node-nested")
                    },
                    title.to_string(),
                    match (children.is_empty(), empty_matches, operator) {
                        (true, true, _) => tr("schedule-node-empty-always").to_string(),
                        (true, false, _) => tr("schedule-node-empty-never").to_string(),
                        (false, _, GroupOperator::All) => {
                            crate::i18n::schedule_group_all_detail(children.len())
                        }
                        (false, _, GroupOperator::Any) => {
                            crate::i18n::schedule_group_any_detail(children.len())
                        }
                    },
                )
            }
        };
        let content = row![
            column![
                label(
                    eyebrow,
                    7.8,
                    scale,
                    if selected {
                        with_alpha(palette.primary_text, 0.72)
                    } else {
                        with_alpha(palette.primary, 0.5)
                    },
                ),
                label(
                    title,
                    11.0,
                    scale,
                    if selected { palette.primary_text } else { palette.surface_text },
                ),
                label(
                    detail,
                    8.6,
                    scale,
                    if selected {
                        with_alpha(palette.primary_text, 0.68)
                    } else {
                        with_alpha(palette.surface_text, 0.44)
                    },
                ),
            ]
            .spacing(2.0 * scale)
            .width(Length::Fill),
            label(
                if selected {
                    tr("schedule-node-editing").to_string()
                } else {
                    format!("{}  ›", tr("schedule-node-edit"))
                },
                9.0,
                scale,
                if selected {
                    palette.primary_text
                } else {
                    with_alpha(palette.surface_text, 0.68)
                },
            ),
        ]
        .spacing(8.0 * scale)
        .align_y(Alignment::Center);
        let mut branch = column![
            container(
                button(content)
                    .width(Length::Fill)
                    .padding([8.0 * scale, 9.0 * scale])
                    .on_press(Message::Sched(SchedMsg::EditNode(
                        rule_idx as u16,
                        path_message(path),
                    )))
                    .style(move |_theme, status| {
                        crate::frontend::ui::folio_button_style(selected, false, palette, status)
                    }),
            )
            .padding(logical_padding(0.0, 0.0, 0.0, depth as f32 * 13.0 * scale)),
        ]
        .spacing(4.0 * scale);
        if let ConditionKind::Group { operator, children } = &node.kind {
            for (index, child) in children.iter().enumerate() {
                let mut child_path = path.to_owned();
                child_path.push(index);
                branch = branch.push(
                    row![
                        label(
                            match (index, operator) {
                                (0, _) => tr("schedule-if"),
                                (_, GroupOperator::All) => tr("schedule-and"),
                                (_, GroupOperator::Any) => tr("schedule-or"),
                            },
                            8.0,
                            scale,
                            with_alpha(palette.primary, 0.52),
                        ),
                        self.condition_node(
                            rule_idx,
                            child,
                            &child_path,
                            depth + 1,
                            scale,
                            palette
                        ),
                    ]
                    .spacing(5.0 * scale)
                    .align_y(Alignment::Center),
                );
            }
            let adding = matches!(
                self.editing.as_ref(),
                Some(Editing::Add(row, group_path)) if *row == rule_idx && group_path == path
            );
            branch = branch.push(
                container(crate::frontend::ui::folio_action(
                    tr("schedule-add-condition-or-group"),
                    adding,
                    Some(Message::Sched(SchedMsg::EditAdd(rule_idx as u16, path_message(path)))),
                    Length::Fill,
                    scale * 0.9,
                    palette,
                ))
                .padding(logical_padding(
                    3.0 * scale,
                    0.0,
                    2.0 * scale,
                    (depth as f32 + 1.0) * 13.0 * scale,
                )),
            );
        }
        branch.into()
    }

    fn condition_list<'a>(
        &'a self,
        rule_idx: usize,
        rule: &'a RuleRow,
        scale: f32,
        palette: &'a Palette,
    ) -> Element<'a, Message> {
        if invalid_condition(&rule.condition) {
            return column![
                label(tr("schedule-setup-title"), 11.0, scale, palette.surface_text),
                label(
                    tr("schedule-setup-desc"),
                    8.8,
                    scale,
                    with_alpha(palette.surface_text, 0.5),
                )
                .line_height(iced::widget::text::LineHeight::Relative(1.35)),
                crate::frontend::ui::folio_action(
                    tr("schedule-setup-action"),
                    false,
                    Some(Message::Sched(SchedMsg::ResetCondition(rule_idx as u16))),
                    Length::Fill,
                    scale * 0.9,
                    palette,
                ),
            ]
            .spacing(8.0 * scale)
            .into();
        }
        column![
            self.condition_node(rule_idx, &rule.condition, &[], 0, scale, palette),
            label(
                tr("schedule-groups-hint"),
                crate::frontend::ui::TYPE_SMALL,
                scale,
                with_alpha(palette.surface_text, 0.42),
            ),
        ]
        .spacing(7.0 * scale)
        .into()
    }

    fn condition_editor<'a>(
        &'a self,
        rule_idx: usize,
        available: f32,
        scale: f32,
        palette: &'a Palette,
    ) -> Element<'a, Message> {
        match self.editing.as_ref() {
            Some(Editing::Add(row_idx, group_path)) if *row_idx == rule_idx => {
                let mut actions: Vec<(String, bool, Message)> = BLOCK_KINDS
                    .iter()
                    .enumerate()
                    .map(|(kind_idx, (_, name))| {
                        (
                            tr(name).to_string(),
                            false,
                            Message::Sched(SchedMsg::AddBlock(
                                rule_idx as u16,
                                path_message(group_path),
                                kind_idx as u8,
                            )),
                        )
                    })
                    .collect();
                actions.push((
                    tr("schedule-add-group-all").to_string(),
                    false,
                    Message::Sched(SchedMsg::AddGroup(
                        rule_idx as u16,
                        path_message(group_path),
                        false,
                    )),
                ));
                actions.push((
                    tr("schedule-add-group-any").to_string(),
                    false,
                    Message::Sched(SchedMsg::AddGroup(
                        rule_idx as u16,
                        path_message(group_path),
                        true,
                    )),
                ));
                column![
                    label(tr("schedule-add-title"), 10.0, scale, palette.surface_text),
                    label(
                        tr("schedule-add-desc"),
                        8.6,
                        scale,
                        with_alpha(palette.surface_text, 0.48),
                    )
                    .line_height(iced::widget::text::LineHeight::Relative(1.35)),
                    crate::frontend::ui::folio_action_wrap(actions, available, scale, palette),
                ]
                .spacing(8.0 * scale)
                .into()
            }
            Some(Editing::Node(row_idx, path)) if *row_idx == rule_idx => {
                let Some(node) = self.node(rule_idx, path) else {
                    return label(
                        tr("schedule-choose-expression"),
                        10.0,
                        scale,
                        with_alpha(palette.surface_text, 0.46),
                    )
                    .into();
                };
                match &node.kind {
                    ConditionKind::Predicate(block) => Self::block_editor(
                        rule_idx,
                        path,
                        block,
                        node.negated,
                        available,
                        scale,
                        palette,
                    ),
                    ConditionKind::Group { operator, .. } => {
                        let message_path = path_message(path);
                        let mut body = column![
                            crate::frontend::ui::folio_action_wrap(
                                vec![
                                    (
                                        tr("schedule-group-match-all").to_string(),
                                        matches!(operator, GroupOperator::All),
                                        Message::Sched(SchedMsg::SetGroupOperator(
                                            rule_idx as u16,
                                            message_path.clone(),
                                            false,
                                        )),
                                    ),
                                    (
                                        tr("schedule-group-match-any").to_string(),
                                        matches!(operator, GroupOperator::Any),
                                        Message::Sched(SchedMsg::SetGroupOperator(
                                            rule_idx as u16,
                                            message_path.clone(),
                                            true,
                                        )),
                                    ),
                                    (
                                        tr("schedule-group-include").to_string(),
                                        !node.negated,
                                        Message::Sched(SchedMsg::SetNegated(
                                            rule_idx as u16,
                                            message_path.clone(),
                                            false,
                                        )),
                                    ),
                                    (
                                        tr("schedule-group-exclude").to_string(),
                                        node.negated,
                                        Message::Sched(SchedMsg::SetNegated(
                                            rule_idx as u16,
                                            message_path.clone(),
                                            true,
                                        )),
                                    ),
                                ],
                                available,
                                scale,
                                palette,
                            ),
                            crate::frontend::ui::folio_action(
                                tr("schedule-group-add-inside"),
                                false,
                                Some(Message::Sched(SchedMsg::EditAdd(
                                    rule_idx as u16,
                                    message_path.clone(),
                                ))),
                                Length::Fill,
                                scale * 0.9,
                                palette,
                            ),
                        ]
                        .spacing(9.0 * scale);
                        if !path.is_empty() {
                            body = body.push(crate::frontend::ui::folio_destructive_action(
                                tr("schedule-group-remove"),
                                false,
                                Some(Message::Sched(SchedMsg::RemoveNode(
                                    rule_idx as u16,
                                    message_path,
                                ))),
                                Length::Fixed(140.0 * scale),
                                scale * 0.9,
                                palette,
                            ));
                        }
                        body.into()
                    }
                }
            }
            _ => label(
                tr("schedule-editor-idle-hint"),
                10.0,
                scale,
                with_alpha(palette.surface_text, 0.46),
            )
            .into(),
        }
    }

    fn definition_column<'a>(
        rule_idx: usize,
        rule: &'a RuleRow,
        available: f32,
        scale: f32,
        palette: &'a Palette,
    ) -> Element<'a, Message> {
        let identity = schedule_section(
            tr("schedule-identity-title"),
            tr("schedule-identity-desc"),
            crate::frontend::ui::field_input(
                &rule.name,
                tr("schedule-rule-name-placeholder"),
                move |value| Message::Sched(SchedMsg::NameInput(rule_idx as u16, value)),
                Message::Sched(SchedMsg::Submit),
                Length::Fill,
                scale,
                palette,
            ),
            scale,
            palette,
        );
        let wallpaper = schedule_section(
            tr("schedule-wallpaper-title"),
            tr("schedule-wallpaper-desc"),
            column![
                crate::frontend::ui::field_input(
                    &rule.set,
                    tr("schedule-wallpaper-placeholder"),
                    move |value| Message::Sched(SchedMsg::SetInput(rule_idx as u16, value)),
                    Message::Sched(SchedMsg::Submit),
                    Length::Fill,
                    scale,
                    palette,
                ),
                action(
                    tr("schedule-random-action"),
                    tr("schedule-random-action"),
                    rule.set.trim().is_empty() || rule.set == "random",
                    Message::Sched(SchedMsg::SetInput(rule_idx as u16, "random".to_string())),
                    scale,
                    palette,
                ),
            ]
            .spacing(7.0 * scale)
            .into(),
            scale,
            palette,
        );
        let theme_actions = [
            ("", tr("schedule-theme-keep")),
            ("light", tr("schedule-theme-light")),
            ("dark", tr("schedule-theme-dark")),
        ]
        .into_iter()
        .map(|(value, label)| {
            (
                label.to_string(),
                rule.mode == value,
                Message::Sched(SchedMsg::Mode(rule_idx as u16, value.to_string())),
            )
        })
        .collect();
        let theme = schedule_section(
            tr("schedule-theme-title"),
            tr("schedule-theme-desc"),
            crate::frontend::ui::folio_action_wrap(theme_actions, available, scale, palette),
            scale,
            palette,
        );
        column![identity, wallpaper, theme].spacing(15.0 * scale).into()
    }

    fn conditions_column<'a>(
        &'a self,
        rule_idx: usize,
        rule: &'a RuleRow,
        available: f32,
        scale: f32,
        palette: &'a Palette,
    ) -> Element<'a, Message> {
        let conditions = schedule_section(
            tr("schedule-conditions-title"),
            tr("schedule-conditions-desc"),
            self.condition_list(rule_idx, rule, scale, palette),
            scale,
            palette,
        );
        let editing_here = matches!(
            self.editing.as_ref(),
            Some(Editing::Add(row_idx, _) | Editing::Node(row_idx, _)) if *row_idx == rule_idx
        );
        if invalid_condition(&rule.condition) {
            return column![conditions].into();
        }
        let editor_title = match self.editing.as_ref() {
            Some(Editing::Add(row_idx, _)) if *row_idx == rule_idx => {
                tr("schedule-editor-add-title")
            }
            Some(Editing::Node(row_idx, path)) if *row_idx == rule_idx => self
                .node(rule_idx, path)
                .map_or(tr("schedule-editor-expression"), |node| match &node.kind {
                    ConditionKind::Predicate(block) => block_kind_name(block),
                    ConditionKind::Group { .. } => tr("schedule-editor-group-title"),
                }),
            _ => tr("schedule-editor-fallback-title"),
        };
        let editor_description =
            if editing_here { tr("schedule-editor-desc") } else { tr("schedule-editor-idle-desc") };
        let editor = schedule_section(
            editor_title,
            editor_description,
            self.condition_editor(rule_idx, available, scale, palette),
            scale,
            palette,
        );
        column![conditions, editor].spacing(15.0 * scale).into()
    }

    fn reading_surface<'a>(
        &'a self,
        available: f32,
        scale: f32,
        palette: &'a Palette,
    ) -> Element<'a, Message> {
        let Some(rule) = self.rows.get(self.selected) else {
            return container(
                column![
                    label(tr("schedule-empty-eyebrow"), 9.0, scale, palette.primary),
                    label(tr("schedule-empty-title"), 31.0, scale, palette.surface_text),
                    label(
                        tr("schedule-empty-desc"),
                        10.5,
                        scale,
                        with_alpha(palette.surface_text, 0.58),
                    )
                    .line_height(iced::widget::text::LineHeight::Relative(1.4)),
                ]
                .spacing(8.0 * scale),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(36.0 * scale)
            .into();
        };
        let rule_idx = self.selected;
        let target =
            if rule.set.trim().is_empty() { tr("schedule-random-action") } else { rule.set.trim() };
        let intro = column![
            row![
                label(tr("schedule-rule-eyebrow"), 9.0, scale, palette.primary),
                container(text("")).width(Length::Fill),
                label(
                    if self.demo {
                        tr("schedule-status-demo")
                    } else if self.migrated {
                        tr("schedule-status-migrated")
                    } else {
                        tr("schedule-status-saved")
                    },
                    9.0,
                    scale,
                    with_alpha(palette.surface_text, 0.46),
                ),
            ]
            .align_y(Alignment::Center),
            label(
                crate::frontend::ui::ellipsize_text(
                    rule_name(rule),
                    31.0 * legible_type_scale(scale),
                    available * 0.72,
                ),
                31.0,
                scale,
                palette.surface_text,
            ),
            row![
                label(tr("schedule-summary-when-label"), 8.5, scale, palette.primary),
                label(
                    condition_summary(rule),
                    crate::frontend::ui::TYPE_SMALL,
                    scale,
                    with_alpha(palette.surface_text, 0.74)
                ),
            ]
            .spacing(10.0 * scale)
            .align_y(Alignment::Center),
            row![
                label(tr("schedule-summary-then-label"), 8.5, scale, palette.primary),
                label(
                    crate::frontend::ui::ellipsize_text(
                        &tr_args!("schedule-apply-label", target => target),
                        10.5 * legible_type_scale(scale),
                        available * 0.34,
                    ),
                    10.5,
                    scale,
                    with_alpha(palette.surface_text, 0.74),
                ),
                text("·").font(UI_FONT).color(with_alpha(palette.surface_text, 0.42)),
                label(
                    match rule.mode.as_str() {
                        "light" => tr("schedule-theme-light"),
                        "dark" => tr("schedule-theme-dark"),
                        _ => tr("schedule-theme-keep"),
                    },
                    10.5,
                    scale,
                    with_alpha(palette.surface_text, 0.74),
                ),
                container(text("")).width(Length::Fill),
                crate::frontend::ui::folio_destructive_action(
                    tr("schedule-delete-rule"),
                    false,
                    Some(Message::Sched(SchedMsg::RemoveRule(rule_idx as u16))),
                    Length::Fixed(116.0 * scale),
                    scale * 0.9,
                    palette,
                ),
            ]
            .spacing(10.0 * scale)
            .align_y(Alignment::Center),
        ]
        .spacing(8.0 * scale);

        let side_padding = 34.0 * scale;
        let content_width = (available - side_padding * 2.0).max(240.0);
        let column_gap = 22.0 * scale;
        let wide = content_width >= 760.0 * scale.max(0.9);
        let form_width = (content_width - column_gap - 1.0).max(240.0);
        let definition_width =
            if wide { form_width * SCHEDULE_DEFINITION_SHARE } else { content_width };
        let conditions_width = if wide { form_width - definition_width } else { content_width };
        let definition = Self::definition_column(rule_idx, rule, definition_width, scale, palette);
        let conditions = self.conditions_column(rule_idx, rule, conditions_width, scale, palette);
        let form: Element<'a, Message> = if wide {
            row![
                container(definition).width(Length::Fixed(definition_width)),
                vertical_divider(palette),
                container(conditions).width(Length::Fill),
            ]
            .spacing(column_gap)
            .width(Length::Fill)
            .into()
        } else {
            column![definition, conditions].spacing(18.0 * scale).into()
        };

        scrollable(column![intro, form].spacing(28.0 * scale).padding(Padding {
            top: 32.0 * scale,
            right: side_padding,
            bottom: 42.0 * scale,
            left: side_padding,
        }))
        .id(iced::widget::Id::new("folio-schedule-editor"))
        .direction(crate::frontend::ui::thin_vbar())
        .style(crate::frontend::ui::scroll_style(with_alpha(palette.outline, 0.72)))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    pub fn view<'a>(
        &'a self,
        viewport: (f32, f32),
        scale: f32,
        palette: &'a Palette,
    ) -> Element<'a, Message> {
        let scale = schedule_scale(viewport, scale);
        let (panel_width, _) = crate::frontend::ui::folio_sheet_dims(viewport, scale);
        let index_width = SCHEDULE_INDEX_WIDTH * scale.max(0.9);
        let selected =
            self.rows.get(self.selected).map_or(tr("schedule-masthead-fallback"), rule_name);
        let masthead = crate::frontend::ui::folio_masthead(
            tr_args!("schedule-masthead", selected => selected),
            Message::Sched(SchedMsg::Close),
            scale,
            palette,
        );
        crate::frontend::ui::folio_sheet(
            masthead,
            self.priority_index(scale, palette),
            self.reading_surface(panel_width - index_width, scale, palette),
            Message::Sched(SchedMsg::Close),
            viewport,
            scale,
            1.0,
            SCHEDULE_INDEX_WIDTH,
            palette,
        )
    }
}

#[cfg(test)]
mod tests;
