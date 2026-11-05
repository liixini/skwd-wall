use iced::advanced::widget;
use iced::widget::{button, text};
use iced::{Color, Element, Event, Font, Length, Pixels, Point, Size, Vector, mouse};

use super::{MotionSlot, motion_slot};

#[derive(Debug, Clone, PartialEq)]
enum Message {
    ToggleSettings,
    Stash,
}

fn child(label: &'static str) -> Element<'static, Message> {
    text(label).into()
}

#[test]
fn retains_current_tree() {
    let first: Element<'_, Message> = motion_slot(child("current"), None, 1.0, None).into();
    let mut tree = widget::Tree::empty();
    tree.diff(first.as_widget());
    let current_tag = tree.children[0].tag;
    let rolling: Element<'_, Message> =
        motion_slot(child("current"), Some(child("old")), 0.25, None).into();
    tree.diff(rolling.as_widget());
    assert_eq!(tree.children.len(), 2);
    assert_eq!(tree.children[0].tag, current_tag);
    let settled: Element<'_, Message> = motion_slot(child("current"), None, 1.0, None).into();
    tree.diff(settled.as_widget());
    assert_eq!(tree.children.len(), 1);
    assert_eq!(tree.children[0].tag, current_tag);
}

#[test]
fn clips_roll_until_revealed() {
    let slot = MotionSlot {
        current: child("new"),
        previous: Some(child("old")),
        progress: 0.25,
        travel_limit: None,
        identity: None,
    };
    assert!(!slot.ready());
    assert_eq!(slot.travel(80.0), (Vector::new(0.0, 60.0), Vector::new(0.0, -20.0)));
    let settled = MotionSlot { progress: 1.0, ..slot };
    assert!(settled.ready());
    assert_eq!(settled.travel(80.0), (Vector::ZERO, Vector::new(0.0, -80.0)));
}

#[test]
fn tall_branch_roll() {
    let slot = MotionSlot {
        current: child("new"),
        previous: Some(child("old")),
        progress: 0.5,
        travel_limit: Some(40.0),
        identity: None,
    };
    assert_eq!(slot.travel(400.0), (Vector::new(0.0, 20.0), Vector::new(0.0, -20.0)));
}

#[test]
#[ignore = "manual offscreen render of the settings motion slot"]
fn renders_local_roll() {
    use iced::futures::executor::block_on;

    const SCREEN: Size = Size::new(100.0, 60.0);
    const CELL: Size = Size::new(80.0, 40.0);

    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..wgpu::InstanceDescriptor::default()
    });
    let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
        .expect("adapter");
    let (device, queue) =
        block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).expect("device");
    let engine = iced_wgpu::Engine::new(
        &adapter,
        device,
        queue,
        wgpu::TextureFormat::Rgba8UnormSrgb,
        None,
        iced_wgpu::graphics::Shell::headless(),
    );
    let mut renderer = iced_wgpu::Renderer::new(engine, Font::DEFAULT, Pixels(16.0));
    let surface = |color: Color, message: Message| -> Element<'static, Message> {
        button(text(""))
            .width(Length::Fixed(CELL.width))
            .height(Length::Fixed(CELL.height))
            .padding(0.0)
            .on_press(message)
            .style(move |_, _| button::Style {
                background: Some(color.into()),
                ..Default::default()
            })
            .into()
    };
    let backdrop =
        button(text("")).width(Length::Fill).height(Length::Fill).on_press(Message::Stash).style(
            |_, _| button::Style { background: Some(Color::BLACK.into()), ..Default::default() },
        );
    let mut ui = iced_runtime::UserInterface::build(
        iced::widget::stack![
            backdrop,
            motion_slot(
                surface(Color::from_rgb(0.0, 1.0, 0.0), Message::ToggleSettings),
                Some(surface(Color::from_rgb(1.0, 0.0, 0.0), Message::Stash)),
                0.5,
                None,
            )
        ],
        SCREEN,
        iced_runtime::user_interface::Cache::default(),
        &mut renderer,
    );
    ui.draw(
        &mut renderer,
        &iced::Theme::Dark,
        &iced::advanced::renderer::Style::default(),
        mouse::Cursor::Unavailable,
    );
    let viewport = iced_wgpu::graphics::Viewport::with_physical_size(
        Size::new(SCREEN.width as u32, SCREEN.height as u32),
        1.0,
    );
    let pixels = renderer.screenshot(&viewport, Color::BLACK);
    let pixel = |x: usize, y: usize| {
        let idx = (y * SCREEN.width as usize + x) * 4;
        &pixels[idx..idx + 4]
    };
    let old = pixel(20, 10);
    let new = pixel(20, 30);
    assert!(old[0] > old[1] && old[0] > old[2], "outgoing half");
    assert!(new[1] > new[0] && new[1] > new[2], "incoming half");
    assert_eq!(pixel(20, 50)[..3], [0, 0, 0]);

    let click = [
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
    ];
    let mut messages = Vec::new();
    let cursor = mouse::Cursor::Available(Point::new(20.0, 30.0));
    let (_, statuses) = ui.update(
        &click,
        cursor,
        &mut renderer,
        &mut iced::advanced::clipboard::Null,
        &mut messages,
    );
    assert!(messages.is_empty());
    assert!(statuses.iter().all(|status| *status == iced::event::Status::Captured));

    let mut ui = iced_runtime::UserInterface::build(
        motion_slot(
            surface(Color::from_rgb(0.0, 1.0, 0.0), Message::ToggleSettings),
            Some(surface(Color::from_rgb(1.0, 0.0, 0.0), Message::Stash)),
            1.0,
            None,
        ),
        SCREEN,
        ui.into_cache(),
        &mut renderer,
    );
    ui.update(&click, cursor, &mut renderer, &mut iced::advanced::clipboard::Null, &mut messages);
    assert_eq!(messages.len(), 1);
    assert!(matches!(messages.first(), Some(Message::ToggleSettings)));
}

#[test]
#[ignore = "offscreen render of a layered roll"]
fn layers_roll_inside() {
    use iced::widget::{Space, column, container, stack};

    const SCREEN: Size = Size::new(40.0, 120.0);
    const CELL: f32 = 40.0;

    let layered = |color: Color| -> Element<'static, Message> {
        let base = Space::new().width(Length::Fixed(SCREEN.width)).height(Length::Fixed(CELL));
        let fill = container(Space::new())
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| container::Style::default().background(color));
        stack![base, fill].into()
    };
    let old = Color::from_rgb(1.0, 0.0, 0.0);
    let new = Color::from_rgb(0.0, 1.0, 0.0);
    let view = column![
        Space::new().height(Length::Fixed(CELL)),
        motion_slot(layered(new), Some(layered(old)), 0.5, None)
    ];
    let mut renderer = crate::frontend::headless::renderer(Font::DEFAULT);
    let mut ui = iced_runtime::UserInterface::build(
        view,
        SCREEN,
        iced_runtime::user_interface::Cache::default(),
        &mut renderer,
    );
    ui.draw(
        &mut renderer,
        &iced::Theme::Dark,
        &iced::advanced::renderer::Style::default(),
        mouse::Cursor::Unavailable,
    );
    let viewport = iced_wgpu::graphics::Viewport::with_physical_size(
        Size::new(SCREEN.width as u32, SCREEN.height as u32),
        1.0,
    );
    let pixels = renderer.screenshot(&viewport, Color::BLACK);
    let row = |y: usize| &pixels[y * SCREEN.width as usize * 4..][..3];
    assert_eq!(row(30), [0, 0, 0], "old above slot");
    assert_eq!(row(50), [255, 0, 0], "old in slot");
    assert_eq!(row(70), [0, 255, 0], "new in slot");
    assert_eq!(row(90), [0, 0, 0], "new below slot");
}
