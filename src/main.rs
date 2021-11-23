mod renderer;
mod texture;

use anyhow::Result;
use image::io::Reader;
use renderer::Renderer;
use std::path::Path;
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Icon, Window, WindowBuilder},
};

fn main() -> Result<()> {
    let event_loop = EventLoop::new();

    let image = Reader::open("assets/icon.png".to_string())?
        .decode()?
        .into_rgba8();
    let (width, height) = image.dimensions();
    let icon = Icon::from_rgba(image.into_raw(), width, height)?;

    let mut window = WindowBuilder::new()
        .with_title("Dragonglass Renderer")
        .with_inner_size(PhysicalSize::new(800, 600))
        .with_window_icon(Some(icon))
        .build(&event_loop)?;

    let logical_size = window.inner_size();
    let window_dimensions = [logical_size.width, logical_size.height];
    let mut renderer = pollster::block_on(Renderer::new(&window, &window_dimensions))?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        if let Err(error) = step(event, control_flow, &mut window, &mut renderer) {
            eprintln!("Error: {}", error);
            *control_flow = ControlFlow::Exit
        }
    });
}

fn step(
    event: Event<()>,
    control_flow: &mut ControlFlow,
    window: &mut Window,
    renderer: &mut Renderer,
) -> Result<()> {
    *control_flow = ControlFlow::Poll;

    let logical_size = window.inner_size();
    let window_dimensions = [logical_size.width, logical_size.height];

    match event {
        Event::MainEventsCleared => handle_main_events_cleared(renderer, &window_dimensions),
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => {
            if event == &WindowEvent::CloseRequested {
                *control_flow = ControlFlow::Exit
            }
            handle_window_event(event, renderer)
        }
        Event::LoopDestroyed => handle_loop_destroyed(renderer),
        _ => return Ok(()),
    }
}

fn handle_main_events_cleared(renderer: &mut Renderer, window_dimensions: &[u32; 2]) -> Result<()> {
    renderer.render(window_dimensions)?;
    Ok(())
}

fn handle_loop_destroyed(renderer: &mut Renderer) -> Result<()> {
    renderer.cleanup()?;
    Ok(())
}

fn handle_window_event(window_event: &WindowEvent, renderer: &mut Renderer) -> Result<()> {
    match window_event {
        WindowEvent::Resized(physical_size) => handle_resize(*physical_size, renderer),
        WindowEvent::ScaleFactorChanged {
            ref new_inner_size, ..
        } => handle_scale_factor_changed(new_inner_size, renderer),
        WindowEvent::DroppedFile(ref path) => handle_file_dropped(path),
        WindowEvent::MouseInput { button, state, .. } => handle_mouse_input(*button, *state),
        WindowEvent::KeyboardInput {
            input:
                KeyboardInput {
                    state,
                    virtual_keycode: Some(keycode),
                    ..
                },
            ..
        } => handle_keyboard_input(*state, *keycode),
        _ => Ok(()),
    }
}

fn handle_resize(physical_size: PhysicalSize<u32>, renderer: &mut Renderer) -> Result<()> {
    renderer.resize([physical_size.width, physical_size.height]);
    Ok(())
}

fn handle_scale_factor_changed(
    new_inner_size: &&mut PhysicalSize<u32>,
    renderer: &mut Renderer,
) -> Result<()> {
    let size = **new_inner_size;
    renderer.resize([size.width, size.height]);
    Ok(())
}

fn handle_file_dropped(path: &Path) -> Result<()> {
    // TODO
    Ok(())
}

fn handle_mouse_input(button: MouseButton, button_state: ElementState) -> Result<()> {
    // TODO
    Ok(())
}

fn handle_keyboard_input(keystate: ElementState, keycode: VirtualKeyCode) -> Result<()> {
    // TODO
    Ok(())
}
