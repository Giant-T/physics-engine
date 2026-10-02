use std::{sync::Arc, time::Instant};

use cgmath::Vector2;
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, KeyEvent, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

use super::{
    input_state::InputState,
    renderer::{CameraController, Renderer},
};

pub struct State {
    renderer: Renderer,
    camera_controller: CameraController,
    window: Arc<Window>,
    last_frame_time: Instant,
    cursor_locked: bool,
    input_state: InputState,
}

impl State {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let renderer = Renderer::new(window.clone()).await?;
        let camera_controller = CameraController::new(10.0, 0.005);

        Ok(Self {
            window,
            renderer,
            camera_controller,
            last_frame_time: Instant::now(),
            cursor_locked: false,
            input_state: Default::default(),
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.renderer.resize(width, height);
    }

    fn handle_key(&mut self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        if code == KeyCode::Escape && is_pressed {
            event_loop.exit();
        } else {
            self.input_state.update_keys(code, is_pressed);
        }
    }

    fn toggle_cursor(&mut self) {
        if !self.cursor_locked {
            if let Ok(()) = self
                .window
                .set_cursor_grab(winit::window::CursorGrabMode::Locked)
            {
                self.window.set_cursor_visible(false);
                self.cursor_locked = true;
            }
        } else {
            if let Ok(()) = self
                .window
                .set_cursor_grab(winit::window::CursorGrabMode::None)
            {
                self.window.set_cursor_visible(true);
                self.cursor_locked = false;
            }
        }
    }

    fn handle_mouse_input(&mut self, button: MouseButton, is_pressed: bool) {
        if button == MouseButton::Middle && is_pressed {
            self.toggle_cursor();
        }
    }

    fn update(&mut self) {
        let now = Instant::now();
        let delta_time = (now - self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;

        self.camera_controller.update_camera(
            &self.input_state,
            &mut self.renderer.camera,
            delta_time,
            self.cursor_locked,
        );
        self.renderer.update();
    }

    pub fn render(&mut self) -> anyhow::Result<()> {
        self.window.request_redraw();

        self.input_state.reset_mouse_delta();

        self.renderer.render()
    }

    fn handle_mouse_move(&mut self, mouse_delta: (f64, f64)) {
        let mouse_delta = Vector2::new(mouse_delta.0 as f32, mouse_delta.1 as f32);
        self.input_state.handle_mouse_move(mouse_delta);
    }
}

pub struct App {
    state: Option<State>,
}

impl App {
    pub fn new() -> Self {
        Self { state: None }
    }
}

impl ApplicationHandler<State> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes();

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        self.state = Some(pollster::block_on(State::new(window)).unwrap());
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: State) {
        self.state = Some(event);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let state = match &mut self.state {
            Some(state) => state,
            _ => return,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => state.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                state.update();
                match state.render() {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("{e}");
                        event_loop.exit();
                    }
                }
            }
            WindowEvent::MouseInput {
                state: mouse_state,
                button,
                ..
            } => {
                state.handle_mouse_input(button, mouse_state.is_pressed());
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => state.handle_key(event_loop, code, key_state.is_pressed()),
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        let state = match &mut self.state {
            Some(state) => state,
            _ => return,
        };

        match event {
            DeviceEvent::MouseMotion { delta } => {
                state.handle_mouse_move(delta);
            }
            _ => {}
        }
    }
}

pub fn run() -> anyhow::Result<()> {
    let event_loop: EventLoop<State> = EventLoop::with_user_event().build()?;
    let mut app = App::new();
    event_loop.run_app(&mut app)?;

    Ok(())
}
