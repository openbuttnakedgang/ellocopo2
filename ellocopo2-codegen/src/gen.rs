
use proc_macro2::{Span, TokenStream};
use syn::{Ident, LitInt, LitStr};
//use syn::parse::{Parse, ParseStream};
use quote::quote;

use ellocopo2::TypeTag;

use crate::parser::{RegisterDesc, REGISTER_PATH_DELIMETR};

// cargo test && rustfmt --emit stdout codegen.rs
pub fn gen(list: Vec<RegisterDesc>) -> String {
    //let msg_enum = gen_msg_enum(list.clone());
    let msg_infra = gen_req2msg(list.clone());
    let _typecheck = gen_typecheck(list.clone());
    let _path2num = gen_path2num(list.clone());

    quote!(
        #msg_infra
        //#typecheck
        //#path2num
    ).to_string()
}

fn gen_req2msg(list: Vec<RegisterDesc>) -> TokenStream {

    let path: Vec<LitStr> = list.iter()
                      .map(|r| r.path.clone())
                      .map(|p| LitStr::new(&p, Span::call_site()))
                      .collect();

    let typetag: Vec<TokenStream> = list.iter()
                      .map(|r| r.ty)
                      .map(|ty| convert_typetag(ty))
                      .collect();


    let rd_variants: Vec<Ident> = list.iter()
                       .map(|r| r.path.clone() )
                       .map(|r| r.replace(REGISTER_PATH_DELIMETR, "_"))
                       .map(|p| Ident::new(&(String::from("rd") + &p), Span::call_site()))
                       .collect();
    
    let wr_variants: Vec<Ident> = list.iter()
                       .map(|r| r.path.clone() )
                       .map(|r| r.replace(REGISTER_PATH_DELIMETR, "_"))
                       .map(|p| Ident::new(&(String::from("wr") + &p), Span::call_site()))
                       .collect();
    
    let ty: Vec<TokenStream> = list.iter()
                      .map(|reg| convert_ty(reg.ty))
                      .collect();

    quote!(

        #[allow(non_camel_case_types)]
        #[derive(Debug, Clone, Copy)]
        pub enum MsgReq<'a> {
            #(
                #rd_variants,
                #wr_variants(#ty),
            )*
        }

        pub fn req2msg<'a>(code: RequestCode, path: &str, value: Value<'a>) -> Result<MsgReq<'a>, ()> {
            match (code, path, value) {
                #(
                    (RequestCode::READ, #path, _) => Ok(MsgReq::#rd_variants),
                )*
                #(
                    (RequestCode::WRITE, #path, Value::#typetag(v)) => Ok(MsgReq::#wr_variants(v)),
                )*
                _ => Err(())
            }
        }
    )
}


fn gen_path2num(list: Vec<RegisterDesc>) -> TokenStream {
    let path: Vec<LitStr> = list.iter()
                      .map(|r| r.path.clone())
                      .map(|p| LitStr::new(&p, Span::call_site()))
                      .collect();

    let num: Vec<LitInt> = (0 .. list.len())
                      .map(|n| n.to_string())
                      .map(|n| LitInt::new(&n, Span::call_site()))
                      .collect();
    
    quote!(
        pub fn path2num(path: &str) -> Option<u32> {
            match path {
                #(
                    #path => Some(#num),
                )*
                _ => None,
            }
        }
        pub fn num2path(num: u32) -> Option<&'static str> {
            match path {
                #(
                    #num => #path,
                )*
                _ => None,
            }
        }
    )
}

fn gen_typecheck(list: Vec<RegisterDesc>) -> TokenStream {
    
    let path: Vec<LitStr> = list.iter()
                      .map(|r| r.path.clone())
                      .map(|p| LitStr::new(&p, Span::call_site()))
                      .collect();
    
    let typetag: Vec<TokenStream> = list.iter()
                      .map(|r| r.ty)
                      .map(|ty| convert_typetag(ty))
                      .collect();

    quote!(
        pub fn path2ty(path: &str) -> Option<TypeTag> {
            use TypeTag::*;
            match path {
                #(
                    #path => Some(#typetag),
                )*
                _ => None,
            }
        }
    )
}

fn gen_msg_enum(list: Vec<RegisterDesc>) -> TokenStream {

    let rd_variants: Vec<Ident> = list.iter()
                       .map(|r| r.path.clone() )
                       .map(|r| r.replace(REGISTER_PATH_DELIMETR, "_"))
                       .map(|p| Ident::new(&(String::from("rd_") + &p), Span::call_site()))
                       .collect();
    
    let wr_variants: Vec<Ident> = list.iter()
                       .map(|r| r.path.clone() )
                       .map(|r| r.replace(REGISTER_PATH_DELIMETR, "_"))
                       .map(|p| Ident::new(&(String::from("wr_") + &p), Span::call_site()))
                       .collect();
    
    let ty: Vec<TokenStream> = list.iter()
                      .map(|reg| convert_ty(reg.ty))
                      .collect();

    quote!(
        #[derive(Debug, Clone, Copy)]
        pub enum MsgReq<'a> {
            #(
                #rd_variants,
                #wr_variants(#ty),
            )*
        }
    )
}

fn convert_ty(ty: TypeTag) -> TokenStream {
    use TypeTag::*;
    match ty {
         UNIT  => quote!(()),
         BOOL  => quote!(bool),
         U8    => quote!(u8),
         I32   => quote!(i32),
         U32   => quote!(u32),
         STR   => quote!(&'a str),
         BYTES => quote!(&'a [u8]),
        _      => panic!("Gen: unsupproted type: {:?}", ty),
    }
}

fn convert_typetag(ty: TypeTag) -> TokenStream {
    use TypeTag::*;
    match ty {
         UNIT  => quote!(UNIT),
         BOOL  => quote!(BOOL),
         U8    => quote!(U8),
         I32   => quote!(I32),
         U32   => quote!(U32),
         STR   => quote!(STR),
         BYTES => quote!(BYTES),
        _      => panic!("Gen: unsupproted type: {:?}", ty),
    }
}







