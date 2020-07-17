use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

use recolored::*;

mod ast;
mod backend;
mod lexer;
use backend::{Backend, JsBackend};

fn main() {
    let src = include_str!("test.chip");
    let mut program = lexer::Program::new();
    let chip = lexer::Chip::parse("main".into(), src, &mut program);
    JsBackend::compile(chip, program);
}

#[allow(dead_code)]
fn print(p: Pair<lexer::Rule>, s: String, e: String) {
    print!(
        "{}({})",
        format!("{:?}", p.as_rule()).blue(),
        p.as_str().green()
    );
    let inner = p.into_inner().collect::<Vec<_>>();
    if inner.len() > 1 {
        println!("");
        let mut i = 0;
        for pair in inner.clone() {
            println!("{} ┃ ", s[..s.len() - " ┣ ".len()].to_string());
            if i < inner.len() - 1 {
                print!("{}", s);
            } else {
                print!("{}", e);
                //line = " ";
            }
            print(pair, format!(" ┃ {}", s), format!(" ┃ {}", e));
            i += 1;
            if i < inner.len() {
                println!("");
            }
        }
    } else if inner.len() > 0 {
        print!("{}", " > ".red());
        for pair in inner {
            print(pair, s.clone(), e);
            break;
        }
    }
}
