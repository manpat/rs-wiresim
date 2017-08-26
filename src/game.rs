use math::*;
use wire::*;
use gl;

use text::TextRenderer;

use std::ops::Fn;

const WIRE_TICK_DURATION: f32 = 1.0/10.0;

const PLAYER_HEAD_HEIGHT: f32 = 2.0;
const PLAYER_YAW_RATE: f32 = PI * 4.0;
const PLAYER_PITCH_RATE: f32 = PLAYER_YAW_RATE;
const PLAYER_WALK_SPEED: f32 = 5.0;

#[derive(Copy, Clone)]
pub enum Key {
	Forward, Left, Right, Back
}

pub struct Item {
	spawn: Box<Fn(&mut WireContext) -> u32>,
	color: Vec3,
	name: &'static str,
}

pub struct GameContext {
	wire_context: WireContext,
	wire_update_timer: f32,

	text_renderer: TextRenderer,

	player_position: Vec2,
	player_yaw: f32,
	player_pitch: f32,

	hovered_node: Option<u32>,
	hovered_port: u32,
	connecting_node: Option<(u32, u32)>,

	key_states: [bool; 4],

	node_views: Vec<NodeView>,

	items: Vec<Item>,
	current_item: i32,
}

struct NodeView {
	node_id: u32,
	position: Vec3,
	color: Vec3,
}

impl GameContext {
	pub fn new() -> Self {
		let items = vec![
			Item{
				spawn: box |ctx: &mut WireContext| ctx.add_node( ConstantNode{value: 5} ),
				color: Vec3::new(0.2, 0.2, 0.2),
				name: "Constant",
			},

			Item{
				spawn: box |ctx: &mut WireContext| ctx.add_node( AddNode::new() ),
				color: Vec3::new(0.2, 0.6, 0.2),
				name: "Adder",
			},

			Item{
				spawn: box |ctx: &mut WireContext| ctx.add_node( OutputNode::new("output") ),
				color: Vec3::new(0.6, 0.2, 0.2),
				name: "Output",
			},

			Item{
				spawn: box |ctx: &mut WireContext| ctx.add_node( CounterNode::new() ),
				color: Vec3::new(0.6, 0.5, 0.2),
				name: "Counter",
			},
		];

		GameContext {
			wire_context: WireContext::new(),
			wire_update_timer: 0.0,

			text_renderer: TextRenderer::new(),

			player_position: Vec2::splat(0.0),
			player_yaw: 0.0,
			player_pitch: 0.0,

			hovered_node: None,
			hovered_port: 0,
			connecting_node: None,

			key_states: [false; 4],

			node_views: Vec::new(),
			items,
			current_item: 0,
		}
	}

	pub fn get_key_state(&mut self, key: Key) -> &mut bool {
		use self::Key::*;

		match key {
			Forward => &mut self.key_states[0],
			Left => &mut self.key_states[1],
			Right => &mut self.key_states[2],
			Back => &mut self.key_states[3],
		}
	}

	pub fn set_key_state(&mut self, key: Key, down: bool) {
		*self.get_key_state(key) = down;
	}

	pub fn prev_item(&mut self) {
		self.current_item -= 1;
		if self.current_item < 0 {
			self.current_item += self.items.len() as i32;
		}
	}

	pub fn next_item(&mut self) {
		self.current_item += 1;
		if self.current_item >= self.items.len() as i32 {
			self.current_item = 0;
		}
	}

	fn get_hovered_node(&self) -> Option<&WireNode> {
		if self.hovered_node.is_none() { return None }
		let hovered_node_id = self.hovered_node.unwrap();

		self.wire_context.get_node(hovered_node_id)
	}

	pub fn prev_port(&mut self) {
		let connecting = self.connecting_node.is_some();

		if let Some(node) = self.get_hovered_node() {
			let port_count = if connecting {
				node.get_num_inputs()
			} else {
				node.get_num_outputs()
			};

			if port_count == 0 { return }
		}

		if self.hovered_port > 0 {
			self.hovered_port -= 1;
		}
	}

	pub fn next_port(&mut self) {
		let mut port_count = 0;
		let connecting = self.connecting_node.is_some();

		if let Some(node) = self.get_hovered_node() {
			port_count = if connecting {
				node.get_num_inputs()
			} else {
				node.get_num_outputs()
			};
		}

		if port_count == 0 { return }

		self.hovered_port += 1;
		if self.hovered_port >= port_count {
			self.hovered_port = port_count - 1;
		}
	}

	pub fn on_click(&mut self) {
		let item = &self.items[self.current_item as usize];
		let node_id = (item.spawn)(&mut self.wire_context);

		let eye_fwd = self.get_eye_fwd();
		let position = self.get_head_pos() + eye_fwd * 1.5;

		self.node_views.push(NodeView {
			node_id, position,
			color: item.color
		});
	}

	pub fn on_rclick(&mut self) {
		if let Some(src) = self.connecting_node {
			if self.hovered_node.is_none() {
				self.connecting_node = None;
				return;
			}

			self.connecting_node = None;

			let dst = self.hovered_node.unwrap();
			if let Some(node) = self.wire_context.get_node(dst) {
				if node.get_num_inputs() <= self.hovered_port { return }
			}

			self.wire_context.add_connection(src, (dst, self.hovered_port));

		} else if let Some(node_id) = self.hovered_node {
			let node = self.wire_context.get_node(node_id);
			if node.is_none() { return }

			let node = node.unwrap();

			if node.get_num_outputs() > self.hovered_port {
				self.connecting_node = Some((node_id, self.hovered_port));
			}
		}
	}

	pub fn get_eye_fwd(&self) -> Vec3 {
		let Vec2{x, y: z} = Vec2::from_angle(-self.player_yaw - PI/2.0);
		let y = self.player_pitch.sin();

		Vec3::new(x, y, z).normalize()
	}

	pub fn get_head_pos(&self) -> Vec3 {
		let Vec2{x, y: z} = self.player_position;
		Vec3::new(x, PLAYER_HEAD_HEIGHT, z)
	}

	pub fn process_mouse_delta(&mut self, mouse_delta: Vec2) {
		let Vec2{x: yaw_delta, y: pitch_delta} = mouse_delta;
		self.player_yaw -= yaw_delta * PLAYER_YAW_RATE;
		self.player_pitch = (self.player_pitch - pitch_delta * PLAYER_PITCH_RATE)
			.max(-PI/3.0)
			.min(PI/3.0);
	}

	pub fn update(&mut self, dt: f32) {
		self.wire_update_timer -= dt;
		if self.wire_update_timer < 0.0 {
			self.wire_context.step();
			self.wire_update_timer = WIRE_TICK_DURATION;
		}

		let mut vel = Vec2::zero();
		let right = Vec2::from_angle(-self.player_yaw);
		let fwd = Vec2::new(right.y, -right.x);

		if *self.get_key_state(Key::Forward) { vel = vel + fwd; }
		if *self.get_key_state(Key::Back) { vel = vel - fwd; }
		if *self.get_key_state(Key::Right) { vel = vel + right; }
		if *self.get_key_state(Key::Left) { vel = vel - right; }

		let vel_len = vel.length();
		if vel_len > 0.1 {
			self.player_position = self.player_position + vel / vel_len * dt * PLAYER_WALK_SPEED;
		}

		let prev_hovered = self.hovered_node.is_some();
		self.hovered_node = None;
		let dir = self.get_eye_fwd();
		let mut pos = self.get_head_pos();

		let incr = 0.4;
		let radius = 0.3;

		'cast: for _ in 0..10 {
			for v in self.node_views.iter() {
				if (v.position-pos).length() < radius {
					self.hovered_node = Some(v.node_id);
					break 'cast;
				}
			}

			pos = pos + dir * incr;
		}

		if !prev_hovered && self.hovered_node.is_some() {
			self.hovered_port = 0;
		}
	}

	pub unsafe fn draw(&mut self) {
		gl::ClearColor(0.1, 0.1, 0.1, 1.0);
		gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

		gl::LineWidth(3.0);
		gl::PointSize(3.0);

		self.setup_camera();
		GameContext::draw_floor(5.0);

		let hovered_node_id = self.hovered_node.unwrap_or(!0);

		for v in self.node_views.iter() {
			let hovered = v.node_id == hovered_node_id;

			let boost = if hovered { Vec3::splat(0.1) } else { Vec3::zero() };
			let col = v.color + boost;

			gl::Color3f(col.x, col.y, col.z);

			gl::PushMatrix();
			let Vec3{x,y,z} = v.position;
			gl::Translatef(x, y, z);
			GameContext::draw_cube(0.3);
			gl::PopMatrix();
			
			if let Some(node) = self.wire_context.get_node(v.node_id) {
				gl::Color3f(0.8, 0.8, 0.8);
				self.text_renderer.draw(&node.get_label(),
					v.position + Vec3::new(-0.14, 0.0, 0.16));
			}
		}

		gl::Disable(gl::DEPTH_TEST);

		if let Some((source_id, _port)) = self.connecting_node {
			if let Some(node) = self.node_views.iter().find(|&n| n.node_id == source_id) {
				let end_pos = self.get_head_pos() + self.get_eye_fwd() * 1.5;
				let pos = node.position;

				gl::Begin(gl::LINES);
				gl::Color3f(1.0, 0.0, 1.0);
				gl::Vertex3fv(&pos.x);
				gl::Vertex3fv(&end_pos.x);
				gl::End();
			} else {
				self.connecting_node = None;
			}
		}

		for c in self.wire_context.connections.iter() {
			let src = self.node_views.iter().find(|&n| n.node_id == c.input_node);
			let dst = self.node_views.iter().find(|&n| n.node_id == c.output_node);

			if src.is_none() || dst.is_none() { continue }

			let (src, dst) = (src.unwrap(), dst.unwrap());

			gl::Begin(gl::LINES);
			gl::Color3f(0.5, 0.8, 0.8);
			gl::Vertex3fv(&src.position.x);
			gl::Vertex3fv(&dst.position.x);
			gl::End();
		}

		gl::Enable(gl::DEPTH_TEST);

		gl::MatrixMode(gl::PROJECTION);
		gl::LoadIdentity();
		gl::Ortho(
			0.0, 12.0,
			0.0, 12.0,
			-1.0, 1.0);

		gl::MatrixMode(gl::MODELVIEW);
		gl::LoadIdentity();

		gl::Color3f(1.0, 1.0, 1.0);

		{
			let item = &self.items[self.current_item as usize];
			self.text_renderer.draw_scale(item.name, Vec3::new(0.1, 0.1, 0.0), 6.0);
		}

		let connecting = self.connecting_node.is_some();

		if let Some(node) = self.get_hovered_node() {
			let port_count = if connecting {
				node.get_num_inputs()
			} else {
				node.get_num_outputs()
			};

			let center = 6.0;
			let size = 0.4;
			let start = center - port_count as f32 / 2.0 * size + 0.5;

			gl::PointSize(10.0);
			gl::Begin(gl::POINTS);

			for i in 0..port_count {
				if i == self.hovered_port {
					gl::Color3f(1.0, 0.5, 0.5);
				} else {
					gl::Color3f(1.0, 1.0, 1.0);
				}

				gl::Vertex3f(7.0, start + size * i as f32, 0.0);
			}

			gl::End();
		}
	}

	unsafe fn setup_camera(&self) {
		let aspect = 4.0/3.0;
		let fovv = PI as f64/3.0;

		let near = 0.1;
		let far = 50.0;

		let tangent = (fovv/2.0).tan();
		let height = near * tangent;
		let width = height * aspect;

		gl::MatrixMode(gl::PROJECTION);
		gl::LoadIdentity();
		gl::Frustum(
			-width, width,
			-height, height,
			near, far);

		gl::MatrixMode(gl::MODELVIEW);
		gl::LoadIdentity();

		let camera_pos = self.get_head_pos();

		gl::Rotatef(-self.player_pitch / PI * 180.0, 1.0, 0.0, 0.0);
		gl::Rotatef(-self.player_yaw / PI * 180.0, 0.0, 1.0, 0.0);
		gl::Translatef(-camera_pos.x, -camera_pos.y, -camera_pos.z);
	}

	unsafe fn draw_floor(size: f32) {
		gl::Begin(gl::TRIANGLE_FAN);
		gl::Color3f(0.5, 0.5, 0.5);
		gl::Vertex3f(-size, 0.0,-size);
		gl::Vertex3f(-size, 0.0, size);
		gl::Vertex3f( size, 0.0, size);
		gl::Vertex3f( size, 0.0,-size);
		gl::End();
	}

	unsafe fn draw_cube(size: f32) {
		let sz = size/2.0;

		let verts = [
			Vec3::new(-sz,-sz,-sz), // 0
			Vec3::new(-sz,-sz, sz), // 1
			Vec3::new( sz,-sz, sz), // 2
			Vec3::new( sz,-sz,-sz), // 3
			Vec3::new(-sz, sz,-sz), // 4
			Vec3::new(-sz, sz, sz), // 5
			Vec3::new( sz, sz, sz), // 6
			Vec3::new( sz, sz,-sz), // 7
		];

		let idxs = [
			0, 2, 1,  0, 3, 2, // bottom
			4, 5, 6,  4, 6, 7, // top

			0, 5, 4,  0, 1, 5, // left
			2, 7, 6,  2, 3, 7, // right

			1, 6, 5,  1, 2, 6, // front
			0, 4, 7,  0, 7, 3, // back
		];

		gl::Begin(gl::TRIANGLES);
		for &i in idxs.iter() {
			gl::Vertex3fv(&verts[i].x);
		}
		gl::End();
	}
}