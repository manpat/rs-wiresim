use wire::*;

pub struct ButtonNode {
	pressed_ticks: u32
}

impl ButtonNode {
	pub fn new() -> Self {
		ButtonNode {
			pressed_ticks: 0,
		}
	}
}

impl WireNode for ButtonNode {
	fn get_num_outputs(&self) -> u32 { 1 }

	fn get_output(&self, port: u32) -> WireValue {
		if self.pressed_ticks > 0 { WireValue::Bool(true) } else { WireValue::Bool(false) }
	}

	fn on_frob(&mut self) {
		self.pressed_ticks = 2;
	}

	fn update(&mut self) {
		if self.pressed_ticks > 0 {
			self.pressed_ticks -= 1;
		}
	}

	fn get_label(&self) -> String {
		if self.pressed_ticks > 0 {
			"click".to_string()
		} else {
			String::new()
		}
	}
}

pub struct ToggleNode {
	state: bool
}

impl ToggleNode {
	pub fn new() -> Self {
		ToggleNode {
			state: false,
		}
	}
}

impl WireNode for ToggleNode {
	fn get_num_outputs(&self) -> u32 { 1 }

	fn get_output(&self, port: u32) -> WireValue {
		if self.state { WireValue::Bool(true) } else { WireValue::Bool(false) }
	}

	fn on_frob(&mut self) {
		self.state = !self.state;
	}

	fn get_label(&self) -> String {
		if self.state {
			"on".to_string()
		} else {
			"off".to_string()
		}
	}
}