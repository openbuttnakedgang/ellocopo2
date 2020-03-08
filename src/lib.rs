#![no_std]
#![allow(dead_code)]


mod protocol;
mod ty;
mod parser;

pub use protocol::*;
pub use ty::*;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
