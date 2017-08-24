#![feature(box_syntax)]

mod wire;
use wire::*;

fn main() {
	let mut context = WireContext::new();

	let const_1		= context.add_node( ConstantNode{value: 1} );
	let buffer		= context.add_node( BufferNode::new() );
	let self_adder	= context.add_node( AddNode::new() );
	let output		= context.add_node( OutputNode{name: "Fib".to_string()} );

	context.add_connection((const_1, 0), (self_adder, 0));
	context.add_connection((self_adder, 0), (output, 0));
	context.add_connection((self_adder, 0), (buffer, 0));

	for _ in 0..3 {
		context.step();

		println!("--------------");
	}

	context.add_connection((buffer, 0), (self_adder, 1));
	context.add_connection((self_adder, 0), (self_adder, 0));

	println!("--- rewire ---");

	for _ in 0..9 {
		context.step();

		println!("--------------");
	}
}


