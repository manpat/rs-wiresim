#![feature(box_syntax)]
#![feature(slice_patterns)]

extern crate sdl2;
extern crate lodepng;

mod gl {
	include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

mod game;
mod wire;
mod text;
mod math;
mod easing;

use math::*;

fn main() {
	let sdl_ctx = sdl2::init().unwrap();
	let video = sdl_ctx.video().unwrap();

	let gl_attr = video.gl_attr();
	gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
	gl_attr.set_context_flags().debug().set();
	gl_attr.set_context_version(2, 1);

	let window = video.window("Window", 800, 600)
		.opengl()
		.build()
		.unwrap();

	let gl_ctx = window.gl_create_context().unwrap();
	window.gl_make_current(&gl_ctx).unwrap();

	sdl_ctx.mouse().show_cursor(false);
	sdl_ctx.mouse().warp_mouse_in_window(&window, 400, 300);

	gl::load_with(|name| video.gl_get_proc_address(name) as *const _);

	let mut game_ctx = game::GameContext::new();

	unsafe {
		gl::Enable(gl::CULL_FACE);
		gl::Enable(gl::DEPTH_TEST);
		gl::Enable(gl::BLEND);

		gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
	}

	let mut events = sdl_ctx.event_pump().unwrap();
	let mut capture = true;

	'main: loop {
		for event in events.poll_iter() {
			use sdl2::event::Event;
			use sdl2::keyboard::Keycode;
			use sdl2::mouse::MouseButton;
			use game::Key;

			match event {
				Event::KeyDown { keycode: Some(Keycode::Escape), .. } |
				Event::Quit { .. } => break 'main,

				Event::KeyDown { keycode: Some(Keycode::C), .. } => {
					capture = !capture;
					sdl_ctx.mouse().warp_mouse_in_window(&window, 400, 300);
				}

				Event::KeyDown { keycode: Some(key), .. } => {
					let key = match key {
						Keycode::W => Key::Forward,
						Keycode::A => Key::Left,
						Keycode::S => Key::Back,
						Keycode::D => Key::Right,

						Keycode::Q => {
							game_ctx.prev_item();
							break;
						},

						Keycode::E => {
							game_ctx.next_item();
							break;
						},

						_ => break
					};

					game_ctx.set_key_state(key, true);
				}

				Event::KeyUp { keycode: Some(key), .. } => {
					let key = match key {
						Keycode::W => Key::Forward,
						Keycode::A => Key::Left,
						Keycode::S => Key::Back,
						Keycode::D => Key::Right,
						_ => break
					};

					game_ctx.set_key_state(key, false);
				}

				Event::MouseButtonDown{ mouse_btn, .. } => {
					match mouse_btn {
						MouseButton::Left => game_ctx.on_click(),
						MouseButton::Right => game_ctx.on_rclick(),
						_ => {}
					}
				}

				_ => {}
			}
		}

		let mouse = {
			let ms = events.mouse_state();
			Vec2i::new(ms.x(), ms.y())
		};

		if capture {
			let mdiff = (mouse - Vec2i::new(400, 300)).to_vec2() / Vec2::new(400.0, 300.0);
			sdl_ctx.mouse().warp_mouse_in_window(&window, 400, 300);
			game_ctx.process_mouse_delta(mdiff * 1.0/16.0);
		}

		game_ctx.update(1.0/60.0);

		unsafe {
			game_ctx.draw();
		}

		window.gl_swap_window();
	}
}
