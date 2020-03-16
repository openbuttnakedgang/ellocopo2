use core::fmt::Display;

use serde_json::{Value as JsonValue, map::Map};
use ellocopo2::TypeTag;

const ANNOTATION_ACCESS_STR : &'static str = "@access";
const ANNOTATION_TYPE_STR   : &'static str = "@type";
pub const REGISTER_PATH_DELIMETR: &'static str = "/";

#[derive(Clone)]
pub struct RegisterDesc {
    pub path: String,
    pub ty: TypeTag,
    pub meta: MetaDesc,
}

impl core::fmt::Debug for RegisterDesc {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "\nReg: {:40} : {:5} : {:2}", self.path, &format!("{:?}", self.ty), self.meta)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MetaDesc {
    pub w: bool, // Write rights
    pub r: bool, // Read rights
    pub fast: bool, // Fast impl
}

impl Display for MetaDesc {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            MetaDesc{w: true, r: true, ..} => write!(f, "RW"),
            MetaDesc{w: true, r: false, ..} => write!(f, "WO"),
            MetaDesc{w: false, r: true, ..} => write!(f, "RO"),
            _ => write!(f, "!!"),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Access {
    pub w: bool,
    pub r: bool,
}

impl Default for MetaDesc {
    fn default() -> Self {
        Self {
            w: false,
            r: true,
            fast: false,
        }
    }
}

pub fn parser(dsl: &str) -> Vec<RegisterDesc> {
    let v: JsonValue = serde_json::from_str(dsl).unwrap();
    //println!("{:#?}", v);
    //sections(v);
    let l = visit_regs(v);
    println!("{:?}", &l);
        
    l
}

pub fn visit_regs(root: JsonValue) -> Vec<RegisterDesc> {

    fn extract_meta(fields: Map<String, JsonValue>, inhereted_meta: MetaDesc) -> MetaDesc {
        let mut meta = inhereted_meta;
        for (k,v) in &fields {
            if k.starts_with(ANNOTATION_ACCESS_STR) {
                if let JsonValue::String(rights) = v {
                    let Access{ w, r} = access_convert(rights.clone())
                        .expect("Malformed access rights format");
                    meta.w = w;
                    meta.r = r;
                } else  {
                    panic!("Malformed access rights inner type")
                }
            }
        }
        meta
    }

    fn extract_ty(fields: Map<String, JsonValue>) -> Option<TypeTag> {
        let mut ty = None;
        for (k,v) in &fields {
            if k.starts_with(ANNOTATION_TYPE_STR) {
                if let JsonValue::String(tyy) = v {
                    ty = Some(ty_convert(tyy.clone()).unwrap());
                } else  {
                    panic!("Wrong type in @type")
                }
            }
        }
        ty
    }
    
    fn inner_visit(path: String, fields: Map<String, JsonValue>, meta: MetaDesc) -> Vec<RegisterDesc> {
        let mut list = Vec::new();
        let mut meta = extract_meta(fields.clone(), meta);
            
        for (k,v) in &fields {
            // skip @annotations
            if !k.starts_with("@") {
                let updated_path = path.clone() + REGISTER_PATH_DELIMETR + &k;
                match &v {
                    JsonValue::String(field) => {
                        // should be type
                        let ty = ty_convert(field.clone())
                            .expect("Wrong register type");
                        if let TypeTag::UNIT = ty {
                            meta = MetaDesc{w: true, r: false, .. meta};
                        }
                        //println!("Reg: {:40} : {:5} : {:2}", updated_path, &format!("{:?}",ty), meta);
                        list.push(RegisterDesc{ path: updated_path, ty, meta });
                    }
                    JsonValue::Object(fields) => {
                        meta = extract_meta(fields.clone(), meta);
                        match extract_ty(fields.clone()) {
                            // test if nested register description
                            Some(ty) => {
                                if let TypeTag::UNIT = ty {
                                    meta = MetaDesc{w: true, r: false, .. meta};
                                }
                                //println!("Reg: {:40} : {:5} : {:2} : ext", updated_path, &format!("{:?}",ty), meta);
                                list.push(RegisterDesc{ path: updated_path, ty, meta });
                            }
                            // if None it's nested section, so continue
                            None => {
                                let l = inner_visit(path.clone()+ REGISTER_PATH_DELIMETR + &k, fields.clone(), meta);
                                list.extend(l);
                            }
                        }
                    }
                    _ => (),
                }
            }
        }

        list
    }

    let mut list = Vec::new();
    let meta = MetaDesc::default();
    // root object
    if let JsonValue::Object(root) = root {
        for (k,v) in &root {
            // skip all root @annotations for now
            if !k.starts_with("@") {
                match &v {
                    JsonValue::Object(fields) => {
                        let l = inner_visit(String::from("/") + &k, fields.clone(), meta);
                        list.extend(l);
                    }
                    _ => ()
                }
            }
        }
    } else {
        panic!("None root object!")
    }

    list
}

fn ty_convert(tytag: String) -> Result<TypeTag, String> {
    let ty = match tytag.as_str() {
        "()"   => TypeTag::UNIT,
        "bool" => TypeTag::BOOL,
        "u8"   => TypeTag::U8,
        "i32"  => TypeTag::I32,
        "u32"  => TypeTag::U32,
        "str"  => TypeTag::STR,
        "[u8]" => TypeTag::BYTES,
        _      => return Err(format!("Unsupproted type: {}", &tytag)),
    };
    Ok(ty)
}

fn access_convert(access: String) -> Result<Access, String> {
    let access = match access.as_str() {
        "WO" => Access{ w: true, r: false },
        "RO" => Access{ w: false, r: true },
        "RW" => Access{ w: true, r: true },
        _    => return Err(format!("Unsupproted access meta: {}", &access)),
    };
    Ok(access)
}

