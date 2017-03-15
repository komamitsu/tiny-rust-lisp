extern crate tiny_rust_lisp;

use std::io::{self, Write};
use tiny_rust_lisp::{Lisp, LispError};

fn main() {
    let lisp = Lisp::new();
    
    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let result = lisp.eval_line(input.as_str());
        match result {
            Err(LispError::EOF) => break,
            _ => println!("{:?}", result)
        }
    }
}
