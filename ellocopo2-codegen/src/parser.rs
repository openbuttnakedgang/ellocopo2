
use quote::quote;
use serde_json::{Result, Value};

//#[derive(Serialize, Deserialize)]
//#[derive(Debug, Clone, PartialEq)]
//pub enum Node {
//    S { n: String, c: Vec<Node>, d : Option<String> },
//    R { n: String, t: String, d : Option<String> },
//}

pub fn parser(dsl: &str) -> String {
    let v: Value = serde_json::from_str(dsl).unwrap();
    //println!("{:#?}", v);
    sections(v);
        
    //let tokens = quote! {
    //    struct MyTest {
    //        a: u32,
    //        b: u8,
    //    }
    //};
    //println!("{:#}", &tokens);

    String::new()
}


fn sections(dsl: Value) {
    if let Value::Object(dsl) = dsl {
        for (k,v) in &dsl {
            if !k.starts_with("@") {
                println!("Section name: {}", k);
            }
        }
    }
}

fn paths(dsl: Value) {

}

enum Request {

}

enum Answer {

}
