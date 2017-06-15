use std::cmp::min;
use std::thread::{self, JoinHandle};
use std::sync::mpsc::{channel, Sender, Receiver};

use gl;
use gl::types::*;
use glutin::{Window, WindowBuilder, EventsLoop};
use glutin::Event::WindowEvent;
use glutin::WindowEvent::*;
use glutin::ElementState::Pressed;
use glutin::MouseButton::Left;

use render::*;
use game::SquareGame;

/// Converts a board into a texture
fn into_texture(game: SquareGame) -> Vec<u8> {
    let mut data = Vec::with_capacity(game.size * game.size * 4);
    for y in 0..game.size {
        for x in 0..game.size {
            let background = if x % 2 == 0 || y % 2 == 0 { 0x60 } else { 0xff };

            let cell = game.cell(x, y);
            let is_red = cell == 1.0;
            let is_blue = cell == 2.0;

            data.append(&mut vec![
                if is_red { 0xff } else if is_blue { 0x00 } else { background },
                if is_red || is_blue { 0x00 } else { background },
                if is_blue { 0xff } else if is_red { 0x00 } else { background },
                0xff,
            ]);
        }
    }

    data
}

fn make_uv(board_size: usize) -> Vec<u8> {
    let limit = 5 * board_size + 1;
    let line = 2.0 * board_size as f64;

    let mut data = Vec::with_capacity(limit * limit * 4);

    let mut x: f64 = 0.0;
    for _ in 0..limit {
        let mut y: f64 = 0.0;
        for _ in 0..limit {
            data.append(&mut vec![
                (x.floor() / line * 255.0).round() as u8,
                (y.floor() / line * 255.0).round() as u8,
                0x00,
                0x00,
            ]);

            y += if y % 2.0 < 1.0 {
                1.0
            } else {
                0.25
            };
        }

        x += if x % 2.0 < 1.0 {
            1.0
        } else {
            0.25
        };
    }

    data
}

#[derive(Debug)]
struct GameState {
    pub board_size: usize,
    pub edge_size: usize,

    pub program: Program,
    pub uv_tex: GLuint,
    pub board_tex: GLuint,

    pub running: bool,
    pub mouse_x: i32,
    pub mouse_y: i32,
}

impl GameState {
    pub fn new(board_size: usize) -> GameState {
        let edge_size = 2 * board_size + 1;

        init_geometry();

        let uv_tex = make_texture(edge_size);
        update_tex(uv_tex, 5 * board_size + 1, make_uv(board_size));

        GameState {
            board_size,
            edge_size,

            program: load_program("
                #version 420 compatibility

                in vec2 UV;
                uniform sampler2D uvSampler;
                uniform sampler2D boardSampler;

                void main() {
                    vec2 uv_tex = texture2D(uvSampler, UV).yx;
                    gl_FragColor = texture2D(boardSampler, uv_tex);
                }
            "),
            uv_tex,
            board_tex: make_texture(edge_size),

            running: true,
            mouse_x: 0,
            mouse_y: 0,
        }
    }
}

fn init_window() -> (Window, EventsLoop) {
    let events_loop = EventsLoop::new();
    let window = {
        WindowBuilder::new()
            .with_title("Square".to_string())
            .with_dimensions(512, 512)
            .with_vsync()
            .build(&events_loop)
            .unwrap()
    };

    unsafe {
        window.make_current().unwrap();
    }

    (window, events_loop)
}

pub fn display_thread(board_size: usize) -> (Sender<SquareGame>, Receiver<(f32, f32)>, JoinHandle<()>) {
    let (in_sender, in_receiver) = channel();
    let (out_sender, out_receiver) = channel();

    let handle = thread::spawn(move || {
        let (window, events_loop) = init_window();

        gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

        let mut state = GameState::new(board_size);

        while state.running {
            let (width, height) = window.get_inner_size().unwrap();
            let size = min(width, height);

            events_loop.poll_events(|event| {
                match event {
                    WindowEvent{ event: Closed, .. } => {
                        state.running = false;
                    },
                    WindowEvent{ event: MouseMoved(x, y), .. } => {
                        state.mouse_x = x;
                        state.mouse_y = y;
                    },
                    WindowEvent{ event: MouseInput(Pressed, Left), .. } => {
                        out_sender
                            .send((
                                state.mouse_x as f32 / size as f32,
                                state.mouse_y as f32 / size as f32,
                            ))
                            .expect("send");
                    },
                    _ => {},
                }
            });

            if let Some(game) = in_receiver.try_iter().last() {
                let game: SquareGame = game;
                window.set_title(&format!("{} / {}", game.score(0), game.score(1)));
                update_tex(state.board_tex, game.size, into_texture(game));
            }

            blit(&state.program, (state.uv_tex, state.board_tex), size);

            window.swap_buffers().unwrap();
        }
    });

    (in_sender, out_receiver, handle)
}
