use math::*;
use wire::*;
use gl;

const WIRE_TICK_DURATION: f32 = 1.0/10.0;

const PLAYER_HEAD_HEIGHT: f32 = 2.0;
const PLAYER_YAW_RATE: f32 = PI * 5.0;
const PLAYER_PITCH_RATE: f32 = PLAYER_YAW_RATE;
const PLAYER_WALK_SPEED: f32 = 5.0;

#[derive(Copy, Clone)]
pub enum Key {
	Forward, Left, Right, Back,
}

pub struct GameContext {
	wire_context: WireContext,
	wire_update_timer: f32,

	player_position: Vec2,
	player_yaw: f32,
	player_pitch: f32,

	key_states: [bool; 4],
}

impl GameContext {
	pub fn new() -> Self {
		let mut wire_context = WireContext::new();

		let const_1		= wire_context.add_node( ConstantNode{value: 1} );
		let buffer		= wire_context.add_node( BufferNode::new() );
		let self_adder	= wire_context.add_node( AddNode::new() );
		let output		= wire_context.add_node( OutputNode{name: "Fib".to_string()} );

		wire_context.add_connection((const_1, 0), (self_adder, 0));
		wire_context.add_connection((self_adder, 0), (output, 0));
		wire_context.add_connection((self_adder, 0), (buffer, 0));
		wire_context.add_connection((buffer, 0), (self_adder, 1));

		GameContext {
			wire_context,
			wire_update_timer: 0.0,

			player_position: Vec2::splat(0.0),
			player_yaw: 0.0,
			player_pitch: 0.0,

			key_states: [false; 4],
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
	}

	pub unsafe fn draw(&mut self) {
		gl::ClearColor(0.1, 0.1, 0.1, 1.0);
		gl::Clear(gl::COLOR_BUFFER_BIT);

		self.setup_camera();
		GameContext::draw_floor(5.0);
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

		let camera_pos = {
			let Vec2{x, y: z} = self.player_position;
			Vec3::new(x, PLAYER_HEAD_HEIGHT, z)
		};

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
}