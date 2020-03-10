mod parser;

pub fn generate(dsl: &str) -> String {
    parser::parser(dsl)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        use std::fs::read_to_string;
        use std::fs::write;
        use std::path::Path;

        let txt = read_to_string("../scheme.json").unwrap();
        let txt = generate(&txt);
    }

    #[test]
    fn it_works2() {
        let mut my_str = "foo";

        let s = match my_str {
            s @ "foo" => s,
            s @ "bar" => s,
            s @ "bazzz" => s,
            _ => "not any",
        };

        println!("s is : {}", s);
    }
}
