
use proc_macro2::{Span, TokenStream};
use syn::{Ident, Expr, parse_str};
//use syn::parse::{Parse, ParseStream};
use quote::quote;

use ellocopo2::TypeTag;

use crate::parser::REGISTER_PATH_DELIMETR;

use crate::parser::{
    Register, 
    Section,
    DslTree,
    MetaDesc,
};

pub struct MsgStream {
    def: TokenStream,
    vars: TokenStream,
    lf: bool,
}

impl Default for MsgStream {
    fn default() -> Self {
        Self {
            def: TokenStream::new(),
            vars: TokenStream::new(),
            lf: false,
        }
    }
}

// cargo test && rustfmt --emit stdout codegen.rs && rm codegen.rs

/// Entry point for codegen
pub fn gen(mut dsl: DslTree) -> String {
    dsl = preproc(dsl);

    //let msg_enum = gen_msg_enum(list.clone());
    //let msg_infra = gen_req2msg(list.clone());
    //let _typecheck = gen_typecheck(list.clone());
    //let _path2num = gen_path2num(list.clone());
    
    let MsgStream{def, ..} = msg_enum_def::gen(&dsl);
    //let cb = gen_fstcb(&dsl);
    
    let dispatch = dispatch::gen(&dsl);

    quote!(
        pub mod msg {
            #def
        }
        pub use msg::*;

    //    #cb
        #dispatch

    ).to_string()
}

/// Preprocessing step for dsl tree
/// Right now only CamelCase names
fn preproc(mut dsl: DslTree) -> DslTree {
    dsl.visit(&mut |elem| {
        let name = match elem {
            DslTree::SectionV(section) => {
                &mut section.name
            }
            DslTree::RegisterV(register) => {
                &mut register.name
            }
        };
        *name = camel_case_names(name);
    });

    dsl
}

const ENUM_WRITE_POSTFIX: &'static str = "_W";
const ENUM_READ_POSTFIX:  &'static str = "_R";

/// Generation of enum definitions
mod msg_enum_def {
    use super::*;

    pub fn gen(dsl: &DslTree) -> MsgStream {
        visit_tree(dsl)
    }

    fn visit_tree(dsl: &DslTree) -> MsgStream {

        match dsl {
            DslTree::SectionV(section) => {
                if !section.meta.fast {
                    visit_section(section)
                } else { Default::default() }
            }
            DslTree::RegisterV(register) => {
                if !register.meta.fast {
                    visit_register(register)
                } else { Default::default() }
            }
        }
    }

    fn visit_section(sec: &Section) -> MsgStream {

        let Section{name, children, ..} = sec;
        let mut definitions = TokenStream::new();
        let mut variants = TokenStream::new();

        let name_ident = Ident::new(&name, Span::call_site());
        let variant_ident = Ident::new(&(name.clone()), Span::call_site());
        let mut g_lf = false;

        for c in children {
            let MsgStream{def, vars, lf} = visit_tree(c);
            definitions.extend(def);
            variants.extend(vars);
            g_lf |= lf;
        }

        let lf_token = if g_lf { quote!(<'a>) } else { quote!() };

        let out_def = quote!(

            #definitions

            #[allow(non_camel_case_types)]
            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            pub enum #name_ident#lf_token {
                #variants
            }
        );

        let out_var = quote!( 
            #variant_ident(#name_ident#lf_token),
        );

        MsgStream {
            def: out_def, vars: out_var,
            lf: g_lf,
        }
    }

    fn visit_register(reg: &Register) -> MsgStream {
        
        let Register{name, ty, meta: MetaDesc{w, r, ..}, ..} = reg;
        let mut stream = TokenStream::new();

        let lf = if (TypeTag::STR == *ty || TypeTag::BYTES == *ty) && *w {
            true
        } else  {
            false
        };
        
        if *w {
            let name_s = &(name.clone() + ENUM_WRITE_POSTFIX);
            let name_ident = Ident::new(&name_s, Span::call_site());
            let ty_ts = convert_ty(*ty);
            stream.extend(quote!( #name_ident(#ty_ts), ));
        }

        if *r {

            let name_s = &(name.clone() + ENUM_READ_POSTFIX);
            let name_ident = Ident::new(&name_s, Span::call_site());
            stream.extend(quote!( #name_ident, ));
        }
        
        MsgStream {
            def: TokenStream::new(),
            vars: stream,
            lf,
        }
    }

}

fn gen_fstcb(dsl: &DslTree) -> TokenStream {
    let mut fn_names = Vec::new();
    let mut pn_msgs = Vec::new();

    dsl.visit_regs(&mut |reg| {
        if reg.meta.fast {
            if reg.meta.w { unimplemented!() }
            let fn_name_s = "cb_".to_string() + &reg.path.join("_") + "_r";
            pn_msgs.push(fn_name_s.clone());
            let fn_name = Ident::new(&fn_name_s, Span::call_site());
            fn_names.push(fn_name);
        }
    });
    
    quote!(
        #(
            #[no_mangle]
            #[linkage = "weak"]
            pub fn #fn_names() -> Value<'static> {
                unimplemented!(#pn_msgs)
            }
        )*
    )
}


#[derive(Debug, Clone)]
pub struct EnumCtor {
    pre: Vec<String>,
    val: String,
    post: Vec<String>,
}

impl EnumCtor {
    pub fn new() -> Self {
        Self {
            pre: vec![],
            val: String::new(),
            post: vec![],
        }
    }
}

impl Into<String> for EnumCtor {
    fn into(self) -> String {
        self.pre.concat() + &self.val + &self.post.concat()
    }
}

mod dispatch {
    use super::*;

    pub fn gen(dsl: &DslTree) -> TokenStream {
        let mut right_arms = Vec::new();
        let ctor_pre: EnumCtor = EnumCtor::new();
        
        dsl.visit_accum(&mut |dsl, mut ctor_pre: EnumCtor| {
            match dsl {
                DslTree::SectionV(section) => {
                    // First lvl skip
                    if ctor_pre.pre.is_empty() {
                        ctor_pre.pre.push(section.name.clone());
                    } else {
                        ctor_pre.pre.push("::".to_string() + &section.name.clone());
                        ctor_pre.pre.push("(".to_string());
                        ctor_pre.pre.push(section.name.clone());

                        ctor_pre.post.push(")".to_string());
                    }
                }
                DslTree::RegisterV(register) => {
                    let (w_act, r_act, rw) = match register.meta {
                        MetaDesc{w: true, r: true, fast: false} => {
                            ctor_pre.val = "::".to_string() + &register.name + ENUM_WRITE_POSTFIX + "(map_ty_error!(v))";
                            let w_act: String = ctor_pre.clone().into();
                            ctor_pre.val = "::".to_string() + &register.name + ENUM_READ_POSTFIX;
                            let r_act: String = ctor_pre.clone().into();
                            (w_act + ",", r_act, "RW")
                        }
                        MetaDesc{w: true, r: false, fast: false} => {
                            ctor_pre.val = "::".to_string() + &register.name + ENUM_WRITE_POSTFIX + "(map_ty_error!(v))";
                            let w_act: String = ctor_pre.clone().into();
                            (w_act, "".to_string(), "WO")
                        }
                        MetaDesc{w: false, r: true, fast: false} => {
                            ctor_pre.val = "::".to_string() + &register.name + ENUM_READ_POSTFIX;
                            let r_act: String = ctor_pre.clone().into();
                            ("".to_string(), r_act, "RO")
                        }
                        MetaDesc{w: true, r: true, fast: true} => {
                            let w_act = "cb_".to_string() + &register.path.join("_") + "_w((map_ty_error!(v)))";
                            let r_act = "cb_".to_string() + &register.path.join("_") + "_r()";
                            (w_act + ",", r_act, "RW")
                        }
                        MetaDesc{w: true, r: false, fast: true} => {
                            let w_act = "cb_".to_string() + &register.path.join("_") + "_w((map_ty_error!(v)))";
                            (w_act, "".to_string(), "WO")
                        }
                        MetaDesc{w: false, r: true, fast: true} => {
                            let r_act = "cb_".to_string() + &register.path.join("_") + "_r()";
                            ("".to_string(), r_act, "RO")
                        }
                        _ => panic!("Imposible reg meta combintation: {:?}", &register.meta),
                    };

                    let fast = if register.meta.fast { "FAST," } else { "" };

                    let right_arm = format!("impl_arm!({} {}, code, sys_lvl, {}, {} {})", fast, rw, "PrivLvl::NORMAL_LVL", w_act, r_act);
                    right_arms.push(parse_str::<Expr>(&right_arm).unwrap());
                }
            }
            
            ctor_pre
        }, ctor_pre);

        let mut left_arms = Vec::new();
        dsl.visit_regs(&mut |reg| {
            left_arms.push("/".to_string() + &reg.path.join("/"));
        });

        //for (l,r) in left_arms.iter().zip(right_arms.iter()) {
        //    println!("\"{}\" => {}", l, r);
        //}

        quote!(
            pub fn req2msg<'a>(code: RequestCode, path: &str, v: Value<'a>, sys_lvl: PrivLvl) -> DispatchResult<'a> {
                // this is sad
                if code == RequestCode::READ {
                    let _: () = map_ty_error!(v);
                }

                match path {
                    #(
                        #left_arms => { #right_arms }
                    )*
                    _ => {
                        DispatchResult::Err(AnswerCode::ERR_PATH)
                    }
                }
            }
        )
    }
}


fn camel_case_names(name: &str) -> String {
    let empty = String::with_capacity(name.len());
    name.split(|c| c == REGISTER_PATH_DELIMETR.chars().next().unwrap() || c == '_')
        .fold(empty, |acc, s| {
            let mut c = s.chars();
            let s = match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            };
            acc + &s
        })
}

//fn gen_dispatch(list: Vec<RegisterDesc>) -> TokenStream {
//
//    let path: Vec<LitStr> = list.iter()
//                      .map(|r| r.path.clone())
//                      .map(|p| LitStr::new(&p, Span::call_site()))
//                      .collect();
//
//    let typetag: Vec<TokenStream> = list.iter()
//                      .map(|r| r.ty)
//                      .map(|ty| convert_typetag(ty))
//                      .collect();
//
//    let rd_variants: Vec<Ident> = list.iter()
//                       .map(|r| r.path.clone())
//                       .map(|r| butify_path(&r))
//                       .map(|p| Ident::new(&(String::from("Rd_") + &p), Span::call_site()))
//                       .collect();
//    
//    let wr_variants: Vec<Ident> = list.iter()
//                       .map(|r| r.path.clone())
//                       .map(|r| butify_path(&r))
//                       .map(|p| Ident::new(&(String::from("Wr_") + &p), Span::call_site()))
//                       .collect();
//    
//    let ty: Vec<TokenStream> = list.iter()
//                      .map(|reg| convert_ty(reg.ty))
//                      .collect();
//
//    quote!(
//
//        #[allow(non_camel_case_types)]
//        #[derive(Debug, Clone, Copy)]
//        pub enum MsgReq<'a> {
//            #(
//                #rd_variants,
//                #wr_variants(#ty),
//            )*
//        }
//
//        pub fn req2msg<'a>(code: RequestCode, path: &str, value: Value<'a>) -> Result<MsgReq<'a>, ()> {
//            match (code, path, value) {
//                #(
//                    (RequestCode::READ, #path, _) => Ok(MsgReq::#rd_variants),
//                )*
//                #(
//                    (RequestCode::WRITE, #path, Value::#typetag(v)) => Ok(MsgReq::#wr_variants(v)),
//                )*
//                _ => Err(())
//            }
//        }
//    )
//}
//
//
//fn gen_path2num(list: Vec<RegisterDesc>) -> TokenStream {
//    let path: Vec<LitStr> = list.iter()
//                      .map(|r| r.path.clone())
//                      .map(|p| LitStr::new(&p, Span::call_site()))
//                      .collect();
//
//    let num: Vec<LitInt> = (0 .. list.len())
//                      .map(|n| n.to_string())
//                      .map(|n| LitInt::new(&n, Span::call_site()))
//                      .collect();
//    
//    quote!(
//        pub fn path2num(path: &str) -> Option<u32> {
//            match path {
//                #(
//                    #path => Some(#num),
//                )*
//                _ => None,
//            }
//        }
//        pub fn num2path(num: u32) -> Option<&'static str> {
//            match path {
//                #(
//                    #num => #path,
//                )*
//                _ => None,
//            }
//        }
//    )
//}
//
//fn gen_typecheck(list: Vec<RegisterDesc>) -> TokenStream {
//    
//    let path: Vec<LitStr> = list.iter()
//                      .map(|r| r.path.clone())
//                      .map(|p| LitStr::new(&p, Span::call_site()))
//                      .collect();
//    
//    let typetag: Vec<TokenStream> = list.iter()
//                      .map(|r| r.ty)
//                      .map(|ty| convert_typetag(ty))
//                      .collect();
//
//    quote!(
//        pub fn path2ty(path: &str) -> Option<TypeTag> {
//            use TypeTag::*;
//            match path {
//                #(
//                    #path => Some(#typetag),
//                )*
//                _ => None,
//            }
//        }
//    )
//}
//
//fn gen_msg_enum(list: Vec<RegisterDesc>) -> TokenStream {
//
//    let rd_variants: Vec<Ident> = list.iter()
//                       .map(|r| r.path.clone() )
//                       .map(|r| r.replace(REGISTER_PATH_DELIMETR, "_"))
//                       .map(|p| Ident::new(&(String::from("rd_") + &p), Span::call_site()))
//                       .collect();
//    
//    let wr_variants: Vec<Ident> = list.iter()
//                       .map(|r| r.path.clone() )
//                       .map(|r| r.replace(REGISTER_PATH_DELIMETR, "_"))
//                       .map(|p| Ident::new(&(String::from("wr_") + &p), Span::call_site()))
//                       .collect();
//    
//    let ty: Vec<TokenStream> = list.iter()
//                      .map(|reg| convert_ty(reg.ty))
//                      .collect();
//
//    quote!(
//        #[derive(Debug, Clone, Copy)]
//        pub enum MsgReq<'a> {
//            #(
//                #rd_variants,
//                #wr_variants(#ty),
//            )*
//        }
//    )
//}

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
