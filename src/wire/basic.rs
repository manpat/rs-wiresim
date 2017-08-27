use wire::*;

pub struct ConstantNode { pub value: i32 }

impl WireNode for ConstantNode {
	fn get_num_outputs(&self) -> u32 { 1 }

	fn get_output(&self, port: u32) -> WireValue {
		if port == 0 {
			WireValue::Int(self.value)
		} else {
			WireValue::Null
		}
	}

	fn get_label(&self) -> String {
		format!("Constant({})", self.value)
	}
}


pub struct OutputNode { pub name: String, value: WireValue }

impl OutputNode {
	pub fn new(name: &str) -> Self {
		OutputNode {
			name: name.to_string(),
			value: WireValue::Null
		}
	}
}

impl WireNode for OutputNode {	
	fn get_num_inputs(&self) -> u32 { 1 }

	fn on_input_changed(&mut self, port: u32, value: WireValue) {
		if port != 0 { return }

		self.value = value;
		println!("'{}': {:?}", self.name, value);
	}

	fn get_label(&self) -> String {
		format!("{}:{:?}", self.name, self.value)
	}
}


pub struct CounterNode { count: i32 }

impl CounterNode {
	pub fn new() -> Self {
		CounterNode { count: 0 }
	}
}

impl WireNode for CounterNode {
	fn get_num_outputs(&self) -> u32 { 1 }

	fn update(&mut self) {
		self.count += 1;
	}

	fn get_output(&self, port: u32) -> WireValue {
		if port != 0 { return WireValue::Null }
		WireValue::Int(self.count)
	}

	fn get_label(&self) -> String {
		format!("{}", self.count)
	}
}


pub struct AddNode { pub inputs: [i32; 2], value: i32 }

impl AddNode {
	pub fn new() -> Self {
		AddNode {
			inputs: [0; 2],
			value: 0,
		}
	}
}

impl WireNode for AddNode {
	fn get_num_inputs(&self) -> u32 { self.inputs.len() as u32 }
	fn get_num_outputs(&self) -> u32 { 1 }

	fn on_input_changed(&mut self, port: u32, value: WireValue) {
		if port >= self.get_num_inputs() { return }
		if let WireValue::Int(val) = value {
			self.inputs[port as usize] = val;
		}
	}

	fn update(&mut self) {
		self.value = self.inputs.iter().sum();
	}

	fn get_output(&self, port: u32) -> WireValue {
		if port == 0 {
			WireValue::Int(self.value)
		} else {
			WireValue::Null
		}
	}

	fn get_label(&self) -> String {
		format!("{} + {}", self.inputs[0], self.inputs[1])
	}
}


pub struct AndNode { inputs: [bool; 2] }

impl AndNode {
	pub fn new() -> Self {
		AndNode { inputs: [false; 2] }
	}
}

impl WireNode for AndNode {
	fn get_num_inputs(&self) -> u32 { self.inputs.len() as u32 }
	fn get_num_outputs(&self) -> u32 { 1 }

	fn on_input_changed(&mut self, port: u32, value: WireValue) {
		if port >= self.get_num_inputs() { return }
		if let WireValue::Bool(val) = value {
			self.inputs[port as usize] = val;
		}
	}

	fn get_output(&self, port: u32) -> WireValue {
		if port == 0 {
			WireValue::Bool(self.inputs[0] & self.inputs[1])
		} else {
			WireValue::Null
		}
	}

	fn get_label(&self) -> String {
		format!("{} and {}", self.inputs[0], self.inputs[1])
	}
}