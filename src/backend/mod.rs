use crate::lexer::{Chip, Program};
mod js;
pub use js::JsBackend;

pub trait Backend {
	fn compile(chip: Chip, program: Program) -> String;
}