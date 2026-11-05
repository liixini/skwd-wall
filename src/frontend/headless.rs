#![cfg(test)]

use std::cell::RefCell;

use iced::advanced::widget::Tree;
use iced::futures::executor::block_on;
use iced::{Color, Element, Font, Pixels, Rectangle, Size, mouse};

const TEXT_SIZE: Pixels = Pixels(16.0);
const DECODE_WAIT: std::time::Duration = std::time::Duration::from_millis(400);
const PASSES: usize = 5;

pub(crate) fn renderer(font: Font) -> iced_wgpu::Renderer {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
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
        Some(iced_wgpu::graphics::Antialiasing::MSAAx4),
        iced_wgpu::graphics::Shell::headless(),
    );
    iced_wgpu::Renderer::new(engine, font, TEXT_SIZE)
}

pub(crate) fn capture<'a, Message>(
    screen: Size,
    font: Font,
    out: &str,
    view: impl Fn() -> Element<'a, Message>,
) {
    let mut renderer = renderer(font);
    let style = iced::advanced::renderer::Style { text_color: Color::WHITE };
    let physical = Size::new(screen.width as u32, screen.height as u32);
    let viewport = iced_wgpu::graphics::Viewport::with_physical_size(physical, 1.0);
    let mut pixels = Vec::new();
    for pass in 0..PASSES {
        iced::advanced::Renderer::reset(&mut renderer, Rectangle::with_size(screen));
        let mut ui = iced_runtime::UserInterface::build(
            view(),
            screen,
            iced_runtime::user_interface::Cache::default(),
            &mut renderer,
        );
        let redraw =
            iced::Event::Window(iced::window::Event::RedrawRequested(std::time::Instant::now()));
        let _ = ui.update(
            &[redraw],
            mouse::Cursor::Unavailable,
            &mut renderer,
            &mut iced::advanced::clipboard::Null,
            &mut Vec::new(),
        );
        ui.draw(&mut renderer, &iced::Theme::Dark, &style, mouse::Cursor::Unavailable);
        pixels = renderer.screenshot(&viewport, Color::BLACK);
        if pass + 1 < PASSES {
            std::thread::sleep(DECODE_WAIT);
        }
    }
    ::image::RgbaImage::from_raw(physical.width, physical.height, pixels)
        .expect("frame")
        .save(out)
        .expect("png");
}

thread_local! {
    static SHARED: RefCell<Option<(Font, iced_wgpu::Renderer)>> = const { RefCell::new(None) };
}

pub(crate) struct Shared;

impl Drop for Shared {
    fn drop(&mut self) {
        SHARED.take();
    }
}

pub(crate) fn share(font: Font) -> Shared {
    SHARED.set(Some((font, renderer(font))));
    Shared
}

fn with_shared<T>(font: Font, run: impl FnOnce(&mut iced_wgpu::Renderer) -> T) -> T {
    SHARED.with_borrow_mut(|shared| match shared {
        Some((kept, renderer)) if *kept == font => run(renderer),
        _ => run(&mut renderer(font)),
    })
}

pub(crate) fn operate<Message>(
    screen: Size,
    font: Font,
    view: Element<'_, Message>,
    operation: &mut dyn iced::advanced::widget::Operation,
) {
    with_shared(font, |renderer| {
        let mut ui = iced_runtime::UserInterface::build(
            view,
            screen,
            iced_runtime::user_interface::Cache::default(),
            renderer,
        );
        ui.operate(renderer, operation);
    });
}

pub(crate) fn laid_tree<Message>(screen: Size, font: Font, mut view: Element<'_, Message>) -> Tree {
    with_shared(font, |renderer| {
        let mut tree = Tree::new(view.as_widget());
        let limits = iced::advanced::layout::Limits::new(Size::ZERO, screen);
        let node = view.as_widget_mut().layout(&mut tree, renderer, &limits);
        let bounds = Rectangle::with_size(screen);
        if let Some(mut over) = view.as_widget_mut().overlay(
            &mut tree,
            iced::advanced::Layout::new(&node),
            renderer,
            &bounds,
            iced::Vector::ZERO,
        ) {
            over.as_overlay_mut().layout(renderer, screen);
        }
        tree
    })
}
