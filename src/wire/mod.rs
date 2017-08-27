mod basic; 
mod io; 

pub use self::basic::*;
pub use self::io::*;

use std::borrow::BorrowMut;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum WireValue {
	Null,
	Int(i32),
	Bool(bool),
}

impl WireValue {
	pub fn is_null(&self) -> bool {
		match_enum!(*self, WireValue::Null)
	}
}

pub trait WireNode {
	fn get_num_inputs(&self) -> u32 { 0 }
	fn get_num_outputs(&self) -> u32 { 0 }

	fn on_input_changed(&mut self, port: u32, value: WireValue) {}
	fn on_frob(&mut self) {}

	fn get_output(&self, port: u32) -> WireValue { WireValue::Null }

	fn get_label(&self) -> String { String::new() }

	fn update(&mut self) {}
}

#[derive(Copy, Clone, Debug)]
pub struct WireConnection {
	pub input_node: u32,
	pub output_node: u32,

	pub input_port: u32,
	pub output_port: u32,

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

	pub fn get_node(&self, node_id: u32) -> Option<&WireNode> {
		self.nodes.binary_search_by_key(&node_id, |a| a.0).ok()
			.map(|n| &*self.nodes[n].1)
	}

	pub fn get_node_mut(&mut self, node_id: u32) -> Option<&mut (WireNode + 'static)> {
		self.nodes.binary_search_by_key(&node_id, |a| a.0).ok()
			.map(move |n| self.nodes[n].1.borrow_mut())
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

			value: WireValue::Null,
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
				if !connection.value.is_null() {
					connection.value = WireValue::Null;
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
