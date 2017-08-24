
pub type WireValue = Option<i32>;

pub trait WireNode {
	fn get_num_inputs(&self) -> u32 { 0 }
	fn get_num_outputs(&self) -> u32 { 0 }

	fn on_input_changed(&mut self, port: u32, value: WireValue) {}
	fn get_output(&self, port: u32) -> WireValue { None }

	fn update(&mut self) {}
}

#[derive(Copy, Clone, Debug)]
pub struct WireConnection {
	input_node: u32,
	output_node: u32,

	input_port: u32,
	output_port: u32,

	value: WireValue,
	changed: bool,
	invalid: bool,
}

pub struct WireContext {
	pub nodes: Vec<(u32, Box<WireNode>)>,
	pub connections: Vec<WireConnection>,

	pub next_id: u32,
}

impl WireContext {
	pub fn new() -> Self {
		WireContext {
			nodes: Vec::new(),
			connections: Vec::new(),

			next_id: 0,
		}
	}

	pub fn add_node<T: WireNode + 'static>(&mut self, node: T) -> u32 {
		let id = self.next_id;
		self.next_id += 1;
		self.nodes.push((id, box node));
		id
	}

	pub fn remove_node(&mut self, node_id: u32) {
		self.nodes.retain(|n| n.0 != node_id);
	}

	pub fn add_connection(&mut self, from_node: (u32, u32), to_node: (u32, u32)) {
		let input = self.nodes.binary_search_by_key(&from_node.0, |a| a.0);
		let output = self.nodes.binary_search_by_key(&to_node.0, |a| a.0);

		if input.is_err() || output.is_err() {
			panic!("Faulty connection between {} and {}", from_node.0, to_node.0);
		}

		let output = &self.nodes[output.unwrap()].1;
		let input = &self.nodes[input.unwrap()].1;

		if from_node.1 >= input.get_num_outputs() {
			panic!("Can't connect to source node on port #{}", to_node.1);
		}

		if to_node.1 >= output.get_num_inputs() {
			panic!("Can't connect to target node on port #{}", to_node.1);
		}

		// Only one input per port
		self.connections.retain(|c| c.output_node != to_node.0 || c.output_port != to_node.1);

		self.connections.push(WireConnection {
			input_node: from_node.0,
			output_node: to_node.0,

			input_port: from_node.1,
			output_port: to_node.1,

			value: None,
			changed: false,
			invalid: false,
		});
	}

	pub fn step(&mut self) {
		for connection in self.connections.iter_mut() {
			let output = self.nodes.binary_search_by_key(&connection.output_node, |a| a.0);
			if output.is_err() {
				connection.invalid = true;
				continue
			}

			if connection.changed {
				let output = &mut self.nodes[output.unwrap()].1;

				output.on_input_changed(connection.output_port, connection.value);
				connection.changed = false;
			}
		}

		self.connections.retain(|c| !c.invalid);

		for node in self.nodes.iter_mut() {
			node.1.update();
		}

		for connection in self.connections.iter_mut() {
			let input = self.nodes.binary_search_by_key(&connection.input_node, |a| a.0);
			if input.is_err() {
				if connection.value.is_some() {
					connection.value = None;
					connection.changed = true;
					connection.invalid = true;
				}

				continue
			}

			let input = &self.nodes[input.unwrap()].1;
			let new_value = input.get_output(connection.input_port);

			if new_value != connection.value {
				connection.value = new_value;
				connection.changed = true;
			}
		}
	}
}

pub struct ConstantNode { pub value: i32 }
pub struct OutputNode { pub name: String }
pub struct CounterNode { count: i32 }
pub struct BufferNode { value: WireValue }

pub struct AddNode { pub inputs: [i32; 2], value: i32 }

impl WireNode for ConstantNode {
	fn get_num_outputs(&self) -> u32 { 1 }

	fn get_output(&self, port: u32) -> WireValue {
		if port == 0 {
			Some(self.value)
		} else {
			None
		}
	}
}

impl WireNode for OutputNode {
	fn get_num_inputs(&self) -> u32 { 1 }

	fn on_input_changed(&mut self, port: u32, value: WireValue) {
		if port != 0 { return }

		println!("'{}': {:?}", self.name, value);
	}
}

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
		if port != 0 { return None }
		Some(self.count)
	}
}

impl BufferNode {
	pub fn new() -> Self { BufferNode{ value: None } }
}

impl WireNode for BufferNode {
	fn get_num_inputs(&self) -> u32 { 1 }
	fn get_num_outputs(&self) -> u32 { 1 }

	fn on_input_changed(&mut self, port: u32, value: WireValue) {
		self.value = value;
	}

	fn get_output(&self, port: u32) -> WireValue {
		self.value
	}
}

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

		self.inputs[port as usize] = value.unwrap_or(0);
	}

	fn update(&mut self) {
		self.value = self.inputs.iter().sum();
	}

	fn get_output(&self, port: u32) -> WireValue {
		if port == 0 {
			Some(self.value)
		} else {
			None
		}
	}
}