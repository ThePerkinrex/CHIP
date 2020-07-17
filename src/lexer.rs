use pest::{
	iterators::Pairs,
};

use std::collections::HashMap;

use crate::{
	ast::{StatementKind, AST},
	Rule,
};

#[derive(Debug)]
pub struct Program {
	pub files: HashMap<String, Chip>,
}

impl Program {
	pub fn new() -> Self {
		let mut hm = HashMap::new();
		hm.insert(
			"STD.AND".into(),
			Chip {
				ast: vec![
					AST::IN("in0".into()),
					AST::IN("in1".into()),
					AST::OUT("out".into()),
					AST::CUSTOM("AND".into())
					],
				ins: vec!["in0".into(), "in1".into()],
				outs: vec!["out".into()],
				name: "STD.AND".into()
			},
		);
		Self { files: hm }
	}

	pub fn get_chip(&self, name: &String) -> &Chip {
		if let Some(v) = self.files.get(name) {
			v
		} else {
			unreachable!("Name {} should've been loaded by now", name)
		}
	}

	pub fn resolve(&mut self, name: &String) {
		if !self.files.contains_key(name) {
			todo!("resolve import and insert into files") // TODO resolve import and insert into files
		}
	}
}

#[derive(Debug, Clone)]
pub struct Chip {
	pub ast: Vec<AST>,
	pub ins: Vec<String>,
	pub outs: Vec<String>,
	pub name: String
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InOut {
	IN,
	OUT,
}

impl Chip {
	pub fn new(name: String) -> Self {
		Self {
			ast: Vec::new(),
			ins: Vec::new(),
			outs: Vec::new(),
			name
		}
	}

	pub fn lex(&mut self, p: Pairs<Rule>, program: &mut Program) {
		let mut var: HashMap<String, StatementKind> = HashMap::new();
		let mut rail: HashMap<String, Vec<(InOut, String)>> = HashMap::new();
		let mut chip_defs:  HashMap<String, String> = HashMap::new();
		let mut uses: HashMap<String, String> = HashMap::new();
		for pair in p {
			let span = pair.as_span();
			match pair.as_rule() {
				Rule::IN => {
					let name = pair.into_inner().next().unwrap().as_str().to_string();
					if !var.contains_key(&name) {
						self.ast.push(AST::IN(name.clone()));
						self.ins.push(name.clone());
						var.insert(name, StatementKind::IN);
					} else {
						panic!("Name {} is already used", name);
					}
				}
				Rule::OUT => {
					let name = pair.into_inner().next().unwrap().as_str().to_string();
					if !var.contains_key(&name) {
						self.ast.push(AST::OUT(name.clone()));
						self.outs.push(name.clone());
						var.insert(name, StatementKind::OUT);
					} else {
						panic!("Name {} is already used", name);
					}
				}
				Rule::RAIL => {
					let name = pair.into_inner().next().unwrap().as_str().to_string();
					if !var.contains_key(&name) {
						self.ast.push(AST::RAIL(name.clone()));
						var.insert(name.clone(), StatementKind::RAIL);
						rail.insert(name, Vec::new());
					} else {
						panic!("Name {} is already used", name);
					}
				}
				Rule::CONNECT => {
					let mut inner = pair.into_inner();
					let name1 = inner
						.next()
						.unwrap()
						.as_str()
						.to_string()
						.split(".")
						.map(|x| x.to_string())
						.collect::<Vec<_>>();
					let name2 = inner
						.next()
						.unwrap()
						.as_str()
						.to_string()
						.split(".")
						.map(|x| x.to_string())
						.collect::<Vec<_>>();
					// TODO SPLIT DEFINED NAMES INTO VECS
					if var.contains_key(&name1[0]) {
						if var.contains_key(&name2[0]) {
							let kind1 = var.get(&name1[0]).unwrap();
							let kind2 = var.get(&name2[0]).unwrap();
							if kind1 == &StatementKind::RAIL {
								if kind2 == &StatementKind::RAIL {
									panic!(
										"Don't connect 2 RAILs {:?}",
										span.start_pos().line_col()
									)
								} else {
									if kind2 == &StatementKind::CHIP {
										let chip = program.get_chip(chip_defs.get(&name2[0]).unwrap());
										if chip.ins.contains(&name2[1]) {
											let r =
												rail.get_mut(&name1[0]).expect("RAIL not found");
											r.push((InOut::IN, name2.join(".")));
										// IN
										} else if chip.outs.contains(&name2[1]) {
											let r =
												rail.get_mut(&name1[0]).expect("RAIL not found");
											r.push((InOut::OUT, name2.join(".")));
										// OUT
										} else {
											panic!("Name {} is not defined", name2.join("."))
										}
									} else if kind2 == &StatementKind::IN {
										let r = rail.get_mut(&name1[0]).expect("RAIL not found");
										r.push((InOut::IN, name2.join(".")));
									} else if kind2 == &StatementKind::OUT {
										let r = rail.get_mut(&name1[0]).expect("RAIL not found");
										r.push((InOut::OUT, name2.join(".")));
									} else {
										unreachable!()
									}
								}
							} else if kind2 == &StatementKind::RAIL {
								if kind1 == &StatementKind::CHIP {
									let chip = program.get_chip(chip_defs.get(&name1[0]).unwrap());
									if chip.ins.contains(&name1[1]) {
										let r = rail.get_mut(&name2[0]).expect("RAIL not found");
										r.push((InOut::IN, name1.join(".")));
									// IN
									} else if chip.outs.contains(&name1[1]) {
										let r = rail.get_mut(&name2[0]).expect("RAIL not found");
										r.push((InOut::OUT, name1.join(".")));
									// OUT
									} else {
										panic!("Name {} is not defined", name1.join("."))
									}
								} else if kind1 == &StatementKind::IN {
									let r = rail.get_mut(&name2[0]).expect("RAIL not found");
									r.push((InOut::IN, name1.join(".")));
								} else if kind1 == &StatementKind::OUT {
									let r = rail.get_mut(&name2[0]).expect("RAIL not found");
									r.push((InOut::OUT, name1.join(".")));
								} else {
									unreachable!()
								}
							} else if kind1 == &StatementKind::CHIP {
								let chip1 = program.get_chip(chip_defs.get(&name1[0]).unwrap());
								let io = if chip1.ins.contains(&name1[1]) {
									InOut::IN
								} else if chip1.outs.contains(&name1[1]) {
									InOut::OUT
								} else {
									panic!("Name {} not found", name1.join("."))
								};

								if kind2 == &StatementKind::CHIP {
									let chip2 = program.get_chip(chip_defs.get(&name2[0]).unwrap());
									if chip2.ins.contains(&name2[1]) {
										//let r = rail.get_mut(&name1[0]).expect("RAIL not found");
										//r.push((InOut::IN, name2.join(".")));
										if io == InOut::IN {
											panic!(
												"Can't connect {} and {}, both are INPUT",
												name1.join("."),
												name2.join(".")
											)
										}
									// IN
									} else if chip2.outs.contains(&name2[1]) {
										if io == InOut::OUT {
											panic!(
												"Can't connect {} and {}, both are OUTPUT",
												name1.join("."),
												name2.join(".")
											)
										}
									// OUT
									} else {
										panic!("Name {} is not defined", name2.join("."))
									}
								} else if kind2 == &StatementKind::IN {
									if io == InOut::OUT {
										panic!(
											"Can't connect {} and {}, both are OUTPUT",
											name1.join("."),
											name2.join(".")
										)
									}
								} else if kind2 == &StatementKind::OUT {
									if io == InOut::IN {
										panic!(
											"Can't connect {} and {}, both are INPUT",
											name1.join("."),
											name2.join(".")
										)
									}
								}
							} else if kind2 == &StatementKind::CHIP {
								let chip1 = program.get_chip(chip_defs.get(&name1[0]).unwrap());
								let io = if chip1.ins.contains(&name1[1]) {
									InOut::IN
								} else if chip1.outs.contains(&name1[1]) {
									InOut::OUT
								} else {
									panic!("Name {} not found", name1.join("."))
								};

								if kind1 == &StatementKind::IN {
									if io == InOut::OUT {
										panic!(
											"Can't connect {} and {}, both are OUTPUT",
											name1.join("."),
											name2.join(".")
										)
									}
								} else if kind1 == &StatementKind::OUT {
									if io == InOut::IN {
										panic!(
											"Can't connect {} and {}, both are INPUT",
											name1.join("."),
											name2.join(".")
										)
									}
								}
							} else if kind1 == &StatementKind::IN {
								if kind2 == &StatementKind::IN {
									panic!(
										"Can't connect {} and {}, both are OUTPUT",
										name1.join("."),
										name2.join(".")
									)
								}
							} else if kind1 == &StatementKind::OUT {
								if kind2 == &StatementKind::OUT {
									panic!(
										"Can't connect {} and {}, both are INPUT",
										name1.join("."),
										name2.join(".")
									)
								}
							}
						} else {
							panic!("Name {} is not defined", name2[0]);
						}
					} else {
						panic!("Name {} is not defined", name1[0]);
					}
					self.ast.push(AST::CONNECT(name1.join("."), name2.join(".")))
				}
				Rule::CHIP_DEF => {
					let mut inner = pair.into_inner();
					let chip_use_name = inner.next().unwrap().as_str().to_string();
					let define_name = inner.next().unwrap().as_str().to_string();
					if var.contains_key(&chip_use_name) {
						if !var.contains_key(&define_name) {
							self.ast
								.push(AST::CHIP(chip_use_name.clone(), define_name.clone()));
							var.insert(define_name.clone(), StatementKind::CHIP);
							chip_defs.insert(define_name, uses.get(&chip_use_name).unwrap().clone());
						} else {
							panic!("Name {} is already used", define_name);
						}
					} else {
						panic!("Name {} is not defined", chip_use_name);
					}
				}
				Rule::USE => {
					let mut inner = pair.into_inner();
					let chip_path = inner.next().unwrap().as_str().to_string();
					let alias = inner.next().unwrap().as_str().to_string();
					if !var.contains_key(&alias) {
						program.resolve(&chip_path);
						self.ast.push(AST::USE(chip_path.clone(), alias.clone()));
						var.insert(alias.clone(), StatementKind::USE);
						uses.insert(alias, chip_path);
					} else {
						panic!("Name {} is already used", alias);
					}
				}
				Rule::EOI => {}
				x => unreachable!("Rule shouldn't be here {:?}", x),
			}
		}
	}
}
