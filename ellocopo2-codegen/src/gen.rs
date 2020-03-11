
use proc_macro2::{Span, TokenStream};
use syn::{parse_macro_input,  Fields, ItemStruct, Ident, FieldsNamed, ExprLit, Lit, LitInt};
//use syn::parse::{Parse, ParseStream};
use quote::quote;

use crate::parser::RegisterDesc;

pub fn gen(list: Vec<RegisterDesc>) -> String {
    String::new()
}
