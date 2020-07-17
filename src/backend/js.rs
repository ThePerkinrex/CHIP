pub struct JsBackend;
use super::Backend;
use crate::ast::AST;
use crate::lexer::{Chip, Program};

use std::collections::HashMap;
use std::fmt::Write;

// const STD_AND: *const str = "
// class STD_AND {
// 	run(in0, in1) {
// 		return {
// 			out: in0 && in1
// 		}
// 	}
// }
// ";

#[derive(Debug, Clone)]
enum ConnectionTree {
	Regular(String, Vec<ConnectionTree>),
	Chip(String, HashMap<String, ConnectionTree>),
}

impl Backend for JsBackend {
	fn compile(chip: Chip, program: Program) -> String {
		let mut file = String::new();
		writeln!(file, "{}", gen_class(chip.clone().name, chip, &program));
		for (k, v) in program.files.iter() {
			writeln!(file, "{}", gen_class(k.clone().replace(".", "_"), v.clone(), &program));
		}
		//println!("------------------- JS -------------------");
		println!("{}", file);
		file
	}
}

fn gen_class(name: String, chip: Chip, program: &Program) -> String {
	let mut file = String::new();
	writeln!(file, "class {} {{", name);
	writeln!(file, "{}", gen_run_code(chip, &program));
	write!(file, "}}");
	file
}

fn gen_run_code(chip: Chip, program: &Program) -> String {
	let mut func = String::new();
	writeln!(func, "run({}){{", chip.ins.join(","));
	let mut inputs = Vec::new();
	let mut outputs = Vec::new();
	let mut chip_aliases = HashMap::new();
	let mut chip_aliases_v: HashMap<String, &Chip> = HashMap::new();
	let mut chip_defines: HashMap<String, &Chip> = HashMap::new();
	let mut connections = Vec::new();
	let mut rails = Vec::new();
	let mut types = HashMap::new();
	let mut is_custom = false;
	for statement in chip.ast {
		match statement.clone() {
			AST::USE(p, n) => {
				chip_aliases.insert(n.clone(), p.replace(".", "_"));
				chip_aliases_v.insert(n, program.get_chip(&p));
			}
			AST::IN(n) => {
				outputs.push(n.clone());
				types.insert(n, statement.as_kind());
			}
			AST::OUT(n) => {
				writeln!(func, "let {} = false;", n);
				inputs.push(n.clone());
				types.insert(n, statement.as_kind());
			}
			AST::RAIL(n) => {
				writeln!(func, "let {} = false;", n);
				rails.push(n.clone());
				types.insert(n, statement.as_kind());
			}
			AST::CUSTOM(n) => {
				write!(func, "{}", get_custom_code(n));
				is_custom = true;
			}
			AST::CHIP(a, n) => {
				writeln!(func, "let {} = new {}();", n, chip_aliases.get(&a).unwrap());
				for i in &chip_aliases_v.get(&a).unwrap().ins {
					inputs.push(format!("{}.{}", n, i));
				}

				for o in &chip_aliases_v.get(&a).unwrap().outs {
					outputs.push(format!("{}.{}", n, o));
				}
				types.insert(n.clone(), statement.as_kind());
				chip_defines.insert(n, chip_aliases_v.get(&a).unwrap());
			}
			AST::CONNECT(a, b) => {
				let a_state = if inputs.contains(&a) {
					0
				} else if outputs.contains(&a) {
					1
				} else {
					2
				};

				let b_state = if inputs.contains(&b) {
					0
				} else if outputs.contains(&b) {
					1
				} else {
					2
				};
				if a_state == 0 || b_state == 1 {
					connections.push((a, b));
				} else if b_state == 0 || a_state == 1 {
					connections.push((b, a));
				} else {
					unreachable!("LEXER FAILED: (a={}; b={})", a_state, b_state);
				}
			}
		}
	}
	//println!("{:?}", connections);
	if !is_custom {
		// ! BUILD connection tree
		let mut trees = Vec::new();
		for out in &chip.outs {
			trees.push(build_tree(out.clone(), &connections));
		}
		//println!("TREES: {:?}", trees);
		for tree in trees {
			if let ConnectionTree::Regular(name, connected) = tree {
				writeln!(func, "{} = {};", name, val(connected, &chip_defines));
			} else {
				unreachable!("Code shouldn't be here, found a chip output")
			}
		}
	}
	// We should go from the output all the way to the to to see the order of calculation
	// for rail in rails {
	// 	let mut ins = Vec::new();
	// 	for (o,i) in &connections {
	// 		if o == &rail {
	// 			// TODO check for chip outputs
	// 			ins.push(i.clone());
	// 		}
	// 	}
	// 	write!(func, "{} = {};", rail, ins.join("||"));
	// }

	writeln!(func, "return [{}];", chip.outs.join(","));
	func += "}";
	// println!("{}", func);
	func
}

fn build_tree(start: String, connections: &Vec<(String, String)>) -> ConnectionTree {
	if start.contains(".") {
		let mut top: HashMap<String, ConnectionTree> = HashMap::new();
		for (o, i) in connections {
			//println!("{} {}", o, start);
			if o.split(".").next().unwrap() == start.split(".").next().unwrap() && o != &start {
				//FIXME add vec instead of a single value
				top.insert(o.clone(), build_tree(i.clone(), &connections));
			}
		}
		return ConnectionTree::Chip(start, top);
	} else {
		let mut top = Vec::new();
		for (o, i) in connections {
			//println!("{} {}", o, start);
			if o == &start {
				top.push(build_tree(i.clone(), &connections));
			}
		}
		return ConnectionTree::Regular(start, top);
	}
}

fn val(trees: Vec<ConnectionTree>, chip_aliases_v: &HashMap<String, &Chip>) -> String {
	let mut vals: Vec<String> = Vec::new();
	for tree in trees {
		match tree {
			ConnectionTree::Regular(name, children) => {
				if children.len() > 0 {
					vals.push(format!("({})", val(children, chip_aliases_v)));
				} else {
					vals.push(name);
				}
			}
			ConnectionTree::Chip(name, children) => {
				let actual_name: String = name.split(".").next().unwrap().into();
				//dbg!(&actual_name, chip_aliases_v);
				let chip = chip_aliases_v.get(&actual_name).unwrap();
				let mut args = Vec::new();
				for i in &chip.ins {
					let n = format!("{}.{}", actual_name, i);
					args.push(val(vec![children.get(&n).unwrap().clone()], chip_aliases_v));
				}
				//let out_names: Vec<String> = chip.outs.iter().map(|x| format!("{}.{}", actual_name, x)).collect();
				vals.push(format!(
					"{}.run({})[{}]",
					actual_name,
					args.join(","),
					chip.outs
						.iter()
						.position(|x| format!("{}.{}", actual_name, x) == name)
						.expect("Error finding index")
				));
			}
		};
	}
	vals.join("||")
}

fn get_custom_code<'a>(n: String) -> &'a str {
	match n.as_str() {
		"AND" => "out = in0 && in1;\n",
		"NOT" => "o = !i;\n",
		c => panic!("{} is not a valid custom code in JS", c),
	}
}
