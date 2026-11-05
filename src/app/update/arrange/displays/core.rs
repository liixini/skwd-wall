const INVENTORY_UNAVAILABLE: &str =
    "display inventory unavailable; retry when displays are connected";
const DISCONNECTED_CONFIRMATION: &str = "confirm saving the listed disconnected displays";
const DISPLAY_SIZE_CHANGED: &str = "arrange-recipient-resized";
const SOURCE_SIZE_CHANGED: &str = "arrange-source-resized";
const REVIEW_REQUIRED: &str = "review this display before applying";
const NO_REVIEW: &str = "select a display first";
const DISPLAY_SIZE_UNAVAILABLE: &str = "display size unavailable";
const DISPLAY_UNAVAILABLE: &str = "display unavailable";
const DISPLAY_SELECTION_CLOSED: &str = "display selection closed";
const CURRENT_DISPLAY_UNAVAILABLE: &str = "current display unavailable";
const PENDING_REVIEW: &str = "apply or skip the reviewed layout first";
const DISPLAY_CONNECTED: &str = "connected displays cannot be forgotten";
use std::collections::BTreeMap;

use skwd_config::picker_layout::{Layouts, Size};

use crate::app::App;
use crate::app::state::arrange::displays::{DisplayCopy, DisplayDraft, OutputReview};
use crate::contracts::daemon::OutputStatus;
use crate::frontend::arrange::{
    ArrangeMsg, DisplayGeometry, DisplayRecipient, DisplayReview, Target,
};

use super::measure;

const ERROR: &str = "arrange-display-error";
const STEP_COPY: &str = "arrange-step-displays";
const STEP_FORGET: &str = "arrange-step-forget";

pub(in crate::app) fn update(app: &mut App, message: ArrangeMsg) {
    if app.panels.arrange.as_ref().is_none_or(crate::app::state::arrange::ArrangeSession::gesturing)
    {
        return;
    }
    if let Err(error) = change(app, message) {
        app.show_toast(crate::i18n::tr_args!(ERROR, error => error));
    }
    app.scene.touch();
}

fn change(app: &mut App, message: ArrangeMsg) -> Result<(), String> {
    match message {
        ArrangeMsg::OpenDisplays => open(app),
        ArrangeMsg::CloseDisplays => {
            if pending(app) {
                return Err(PENDING_REVIEW.to_owned());
            }
            app.panels.arrange.as_mut().unwrap().display_copy = None;
            Ok(())
        }
        ArrangeMsg::ReviewDisplay(output) => review(app, &output),
        ArrangeMsg::FitDisplay => fit(app),
        ArrangeMsg::AllowDisconnected(allow) => {
            if let Some(review) = app
                .panels
                .arrange
                .as_mut()
                .and_then(|session| session.display_copy.as_mut())
                .and_then(|copy| copy.review.as_mut())
            {
                review.view.save_disconnected = allow;
            }
            Ok(())
        }
        ArrangeMsg::ApplyDisplay => apply(app),
        ArrangeMsg::SkipDisplay => {
            if let Some(copy) =
                app.panels.arrange.as_mut().and_then(|session| session.display_copy.as_mut())
            {
                copy.review = None;
            }
            Ok(())
        }
        ArrangeMsg::ForgetDisplay(output) => forget(app, &output),
        _ => Ok(()),
    }
}

pub(in crate::app::update::arrange) fn pending(app: &App) -> bool {
    app.panels
        .arrange
        .as_ref()
        .and_then(|session| session.display_copy.as_ref())
        .is_some_and(|copy| copy.review.is_some())
}

pub(in crate::app::update::arrange) fn initial_reviews(
    app: &App,
) -> BTreeMap<String, OutputReview> {
    app.config
        .layout_output()
        .map(|output| {
            (
                output.to_owned(),
                OutputReview {
                    viewport: Size { width: app.scene.viewport.0, height: app.scene.viewport.1 },
                    save_disconnected: false,
                },
            )
        })
        .into_iter()
        .collect()
}

pub(in crate::app::update::arrange) fn validate_save(app: &mut App) -> bool {
    if let Some(session) = app.panels.arrange.as_mut() {
        session.disconnected_outputs.clear();
    }
    match save_ready(app) {
        Ok(()) => true,
        Err(error) => {
            app.show_toast(crate::i18n::tr_args!(ERROR, error => error));
            false
        }
    }
}

fn save_ready(app: &mut App) -> Result<(), String> {
    let session = app.panels.arrange.as_ref().ok_or_else(|| DISPLAY_SELECTION_CLOSED.to_owned())?;
    let original = Layouts::read(&super::super::document(session.history.original()))?;
    let current = Layouts::read(app.config.document())?;
    let changed: Vec<_> = current
        .outputs
        .iter()
        .filter(|(output, profile)| original.outputs.get(*output) != Some(*profile))
        .collect();
    if changed.is_empty() {
        return Ok(());
    }
    if app.daemon.output_statuses.is_empty() {
        return Err(INVENTORY_UNAVAILABLE.to_owned());
    }
    let mut disconnected = Vec::new();
    for (output, profile) in changed {
        let review = session.output_reviews.get(output);
        let expected = review.map_or(profile.viewport, |review| review.viewport);
        if let Some(status) = live(app, output) {
            let (width, height) = status.logical_size();
            if width as f32 != expected.width || height as f32 != expected.height {
                let key = if app.config.layout_output() == Some(output.as_str()) {
                    SOURCE_SIZE_CHANGED
                } else {
                    DISPLAY_SIZE_CHANGED
                };
                return Err(crate::i18n::tr(key).to_string());
            }
        } else if review.is_none_or(|review| !review.save_disconnected) {
            disconnected.push(output.clone());
        }
    }
    let pending = !disconnected.is_empty();
    app.panels.arrange.as_mut().unwrap().disconnected_outputs = disconnected;
    if pending { Err(DISCONNECTED_CONFIRMATION.to_owned()) } else { Ok(()) }
}

pub(in crate::app::update::arrange) fn confirm_disconnected(app: &mut App) {
    let current = match Layouts::read(app.config.document()) {
        Ok(current) => current,
        Err(error) => {
            app.show_toast(crate::i18n::tr_args!(ERROR, error => error));
            return;
        }
    };
    let session = app.panels.arrange.as_mut().unwrap();
    for output in session.disconnected_outputs.drain(..) {
        if let Some(profile) = current.outputs.get(&output) {
            session
                .output_reviews
                .entry(output)
                .and_modify(|review| review.save_disconnected = true)
                .or_insert(OutputReview { viewport: profile.viewport, save_disconnected: true });
        }
    }
}

fn open(app: &mut App) -> Result<(), String> {
    if pending(app) {
        return Err(PENDING_REVIEW.to_owned());
    }
    let source = app
        .config
        .layout_output()
        .ok_or_else(|| CURRENT_DISPLAY_UNAVAILABLE.to_owned())?
        .to_owned();
    let viewport = Size { width: app.scene.viewport.0, height: app.scene.viewport.1 };
    let mut profile = app.config.layout_profile(&source, viewport)?;
    let geometry = DisplayGeometry {
        viewport: app.scene.viewport,
        picker: Some(super::super::picker_rect(app)),
        bar: super::super::placement::rect(app, Target::Bar),
        search: super::super::search_rect(app),
        arrange: floating(app, Target::Arrange),
        swatch: floating(app, Target::Swatch),
    };
    measure::edit(&mut profile, &geometry, Target::Picker, super::super::picker_rect(app))?;
    let (recipients, sizes) = recipients(app, &source)?;
    app.panels.arrange.as_mut().unwrap().display_copy =
        Some(DisplayCopy { source, profile, recipients, sizes, review: None });
    Ok(())
}

fn live<'a>(app: &'a App, monitor: &str) -> Option<&'a OutputStatus> {
    app.daemon
        .output_statuses
        .iter()
        .find(|status| status.is_connected() && status.monitor() == monitor)
}

fn recipients(
    app: &App,
    source: &str,
) -> Result<(Vec<DisplayRecipient>, BTreeMap<String, Size>), String> {
    if app.daemon.output_statuses.is_empty() {
        return Err(INVENTORY_UNAVAILABLE.to_owned());
    }
    let connected: Vec<_> =
        app.daemon.output_statuses.iter().filter(|status| status.is_connected()).collect();
    let mut outputs = BTreeMap::new();
    let mut sizes = BTreeMap::new();
    for (output, profile) in Layouts::read(app.config.document())?.outputs {
        let adopted =
            connected.iter().any(|status| status.name == output && status.monitor() != output);
        if adopted {
            continue;
        }
        sizes.insert(output.clone(), profile.viewport);
        outputs.insert(
            output.clone(),
            DisplayRecipient { name: output, connected: false, reviewable: true, staged: false },
        );
    }
    for status in connected {
        let monitor = status.monitor().to_owned();
        let (width, height) = status.logical_size();
        let size = Size { width: width as f32, height: height as f32 };
        if size.valid() {
            sizes.insert(monitor.clone(), size);
        }
        let reviewable = sizes.contains_key(&monitor);
        outputs.insert(
            monitor.clone(),
            DisplayRecipient { name: monitor, connected: true, reviewable, staged: false },
        );
    }
    outputs.remove(source);
    sizes.remove(source);
    let mut outputs: Vec<_> = outputs.into_values().collect();
    outputs.sort_by_key(|recipient| !recipient.connected);
    Ok((outputs, sizes))
}

pub(in crate::app::update::arrange) fn refresh(app: &mut App) -> Result<(), String> {
    let Some(source) = app
        .panels
        .arrange
        .as_ref()
        .and_then(|session| session.display_copy.as_ref())
        .map(|copy| copy.source.clone())
    else {
        return Ok(());
    };
    let (mut refreshed, sizes) = recipients(app, &source)?;
    let copy = app.panels.arrange.as_mut().unwrap().display_copy.as_mut().unwrap();
    for recipient in &mut refreshed {
        recipient.staged =
            copy.recipients.iter().any(|old| old.name == recipient.name && old.staged);
    }
    copy.recipients = refreshed;
    copy.sizes = sizes;
    Ok(())
}

fn review(app: &mut App, output: &str) -> Result<(), String> {
    if pending(app) {
        return Err(PENDING_REVIEW.to_owned());
    }
    refresh(app)?;
    let copy = app
        .panels
        .arrange
        .as_ref()
        .and_then(|session| session.display_copy.as_ref())
        .ok_or_else(|| DISPLAY_SELECTION_CLOSED.to_owned())?;
    let recipient = copy
        .recipients
        .iter()
        .find(|recipient| recipient.name == output)
        .ok_or_else(|| DISPLAY_UNAVAILABLE.to_owned())?;
    let viewport = *copy.sizes.get(output).ok_or_else(|| DISPLAY_SIZE_UNAVAILABLE.to_owned())?;
    let before = app.config.layout_profile(output, viewport)?;
    let mut after = copy.profile.clone();
    after.viewport = viewport;
    let (before_geometry, _) = measure::measure(app, output, &before)?;
    let (after_geometry, compatible) = measure::measure(app, output, &after)?;
    let view = DisplayReview {
        output: output.to_owned(),
        connected: recipient.connected,
        before: before_geometry,
        after: after_geometry,
        compatible,
        save_disconnected: false,
    };
    app.panels.arrange.as_mut().unwrap().display_copy.as_mut().unwrap().review =
        Some(DisplayDraft { profile: after, view });
    Ok(())
}

fn draft(app: &App) -> Result<&DisplayDraft, String> {
    app.panels
        .arrange
        .as_ref()
        .and_then(|session| session.display_copy.as_ref())
        .ok_or_else(|| DISPLAY_SELECTION_CLOSED.to_owned())?
        .review
        .as_ref()
        .ok_or_else(|| NO_REVIEW.to_owned())
}

fn draft_mut(app: &mut App) -> Result<&mut DisplayDraft, String> {
    app.panels
        .arrange
        .as_mut()
        .and_then(|session| session.display_copy.as_mut())
        .ok_or_else(|| DISPLAY_SELECTION_CLOSED.to_owned())?
        .review
        .as_mut()
        .ok_or_else(|| NO_REVIEW.to_owned())
}

fn floating(app: &App, target: Target) -> Option<iced::Rectangle> {
    let present = super::super::placement::present(&app.config, target);
    present.then(|| super::super::placement::rect(app, target))
}

fn fit(app: &mut App) -> Result<(), String> {
    let review = draft(app)?;
    let mut profile = review.profile.clone();
    let mut geometry = review.view.after.clone();
    let output = review.view.output.clone();
    for target in Target::ALL {
        let rect = match target {
            Target::Picker => geometry.picker,
            Target::Bar => Some(geometry.bar),
            Target::Search => Some(geometry.search),
            Target::Arrange => geometry.arrange,
            Target::Swatch => geometry.swatch,
        };
        if let Some(rect) = rect {
            let rect = measure::fit(rect, geometry.viewport, target == Target::Picker);
            measure::edit(&mut profile, &geometry, target, rect)?;
            geometry = measure::measure(app, &output, &profile)?.0;
        }
    }
    let (geometry, compatible) = measure::measure(app, &output, &profile)?;
    let review = draft_mut(app)?;
    review.profile = profile;
    review.view.after = geometry;
    review.view.compatible = compatible;
    Ok(())
}

fn apply(app: &mut App) -> Result<(), String> {
    if app.daemon.output_statuses.is_empty() {
        return Err(INVENTORY_UNAVAILABLE.to_owned());
    }
    let review = draft(app)?;
    let status = live(app, &review.view.output);
    let connected = status.is_some();
    if !review.view.compatible || (!connected && !review.view.save_disconnected) {
        draft_mut(app)?.view.connected = connected;
        return Err(REVIEW_REQUIRED.to_owned());
    }
    if status.is_some_and(|status| {
        let (width, height) = status.logical_size();
        width as f32 != review.profile.viewport.width
            || height as f32 != review.profile.viewport.height
    }) {
        return Err(crate::i18n::tr(DISPLAY_SIZE_CHANGED).to_string());
    }
    let profile = review.profile.clone();
    let output = review.view.output.clone();
    let reviewed = OutputReview {
        viewport: review.profile.viewport,
        save_disconnected: review.view.save_disconnected,
    };
    app.config.set_layout_profile(&output, &profile)?;
    app.panels.arrange.as_mut().unwrap().output_reviews.insert(output, reviewed);
    let copy = app.panels.arrange.as_mut().unwrap().display_copy.as_mut().unwrap();
    let review = copy.review.take().unwrap();
    if let Some(recipient) =
        copy.recipients.iter_mut().find(|recipient| recipient.name == review.view.output)
    {
        recipient.staged = true;
    }
    super::super::record(app, STEP_COPY);
    Ok(())
}

fn forget(app: &mut App, output: &str) -> Result<(), String> {
    let listed = app
        .panels
        .arrange
        .as_ref()
        .and_then(|session| session.display_copy.as_ref())
        .and_then(|copy| copy.recipients.iter().find(|recipient| recipient.name == output))
        .ok_or_else(|| DISPLAY_UNAVAILABLE.to_owned())?;
    if listed.connected || live(app, output).is_some() {
        return Err(DISPLAY_CONNECTED.to_owned());
    }
    if !app.config.remove_layout_profile(output) {
        return Err(DISPLAY_UNAVAILABLE.to_owned());
    }
    let session = app.panels.arrange.as_mut().unwrap();
    session.output_reviews.remove(output);
    if let Some(copy) = session.display_copy.as_mut() {
        copy.review = None;
        copy.recipients.retain(|recipient| recipient.name != output);
        copy.sizes.remove(output);
    }
    super::super::record(app, STEP_FORGET);
    Ok(())
}
