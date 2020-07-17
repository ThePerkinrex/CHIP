use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

use recolored::*;

mod lexer;
mod ast;
mod backend;
use backend::{JsBackend, Backend};

#[derive(Parser)]
#[grammar = "chip.pest"]
pub struct ChipParser;

fn main() {
    // println!("Hello, world!");
    let src = include_str!("test.chip");
    let file = match ChipParser::parse(Rule::CHIP, src) {
        Ok(mut p) => p.next().unwrap(),
        Err(e) => {
            //print_e(&e, src);
            panic!("Parser error:\n{}", e);
        }
    };
        // .expect("unsuccessful parse") // unwrap the parse result
        // .next()
        // .unwrap(); // get and unwrap the `file` rule; never fails
    // print(file.clone(), " ┣ ".into(), " ┗ ".into());
    // println!("");
    let mut program = lexer::Program::new();
    let mut chip = lexer::Chip::new("main".into());
    chip.lex(file.into_inner(), &mut program);
    // println!("{:?}", chip);
    JsBackend::compile(chip, program);
    //println!("\n{:?}", lexer::lexer(file.into_inner()));
}

#[allow(dead_code)]
fn print(p: Pair<Rule>, s: String, e: String) {
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
    //print!("\n{}", s[..s.len()-" ┣ ".len()].to_string());
    } else if inner.len() > 0 {
        print!("{}", " > ".red());
        for pair in inner {
            print(pair, s.clone(), e);
            break;
        }
    }
}
