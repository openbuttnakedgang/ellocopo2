#![allow(dead_code)]

pub mod parser;
mod gen;

pub fn generate(dsl: &str) -> String {
    let l = parser::parser(dsl);
    gen::gen(l)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::prelude::*;
    use std::fs::read_to_string;
    use std::fs::File;

    #[test]
    fn it_works() {

        let txt = read_to_string("../scheme.json").unwrap();
        let txt = generate(&txt);
        
        let mut file = File::create("codegen.rs").unwrap();
        file.write_all(txt.as_bytes()).unwrap();
    }
}
