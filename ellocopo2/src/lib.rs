
#![cfg_attr(not(feature="std"), no_std)]
#![allow(dead_code)]

mod protocol;
mod ty;
mod parser;

pub mod priv_lvl;

pub use protocol::*;
pub use ty::*;
pub use parser::*;
#[cfg(feature = "std")]
pub use crate::parser::owned;



#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
