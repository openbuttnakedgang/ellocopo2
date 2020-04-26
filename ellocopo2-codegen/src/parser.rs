use std::fmt::Debug;

use serde_json::{Value as JsonValue, map::Map};
use ellocopo2::TypeTag;

const ANNOTATION_TOKEN:           &'static str = "@";
const ANNOTATION_ACCESS_STR:      &'static str = "@access";
const ANNOTATION_CONTROL_STR:     &'static str = "@control";
const ANNOTATION_TYPE_STR:        &'static str = "@type";
pub const REGISTER_PATH_DELIMETR: &'static str = "/";

#[derive(Clone, Debug)]
pub enum DslTree {
    SectionV(Section),
    RegisterV(Register),
}

#[derive(Clone, Debug)]
pub struct Section {
    pub path: Vec<String>,
    pub name: String,
    pub meta: MetaDesc,
    pub children: Vec<DslTree>,
}

#[derive(Clone, Debug)]
pub struct Register {
    pub path: Vec<String>,
    pub name: String,
    pub ty: TypeTag,
    pub meta: MetaDesc,
}

#[derive(Clone, Copy)]
pub struct MetaDesc {
    pub w: bool, // Write rights
    pub r: bool, // Read rights
    pub fast: bool, // Fast impl
    // TODO: Смотри заметку ниже
    //pub w_plvl: --; 
    //pub r_plvl: --;
}

impl DslTree {
    pub fn visit(&mut self, f: &mut impl FnMut(&mut DslTree)) {
        f(self);
        match self {
            DslTree::SectionV(section) => {
                for mut c in &mut section.children {
                    Self::visit(&mut c, f);
                }
            }
            _ => (),
        }
    }

    pub fn visit_accum<A: Clone>(&self, f: &mut impl FnMut(&DslTree, A) -> A, a: A) {
        let a = f(self, a);
        match self {
            DslTree::SectionV(section) => {
                for c in &section.children {
                    let a = a.clone();
                    Self::visit_accum(&c, f, a);
                }
            }
            _ => (),
        }
    }

    pub fn visit_regs(&self, f: &mut impl FnMut(&Register)) {
        match self {
            DslTree::SectionV(section) => {
                for c in &section.children {
                    Self::visit_regs(c, f);
                }
            }
            DslTree::RegisterV(register) => {
                f(register);
            }
        }
    }
}


impl Debug for MetaDesc {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            MetaDesc{w: true, r: true,  ..} => write!(f, "RW")?,
            MetaDesc{w: true, r: false, ..} => write!(f, "WO")?,
            MetaDesc{w: false, r: true, ..} => write!(f, "RO")?,
            _ => write!(f, "!!")?,
        };

        if self.fast {
            write!(f, " fast")?;
        }

        Ok(())
    }
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

pub fn parser(dsl: &str) -> Result<DslTree, String> {
    let v: JsonValue = serde_json::from_str(dsl).unwrap();
    //println!("{:#?}", v);
    //sections(v);
    let l = parse_dsl(v);
    println!("{:#?}", &l);
        
    l
}

fn parse_dsl(root: JsonValue) -> Result<DslTree, String>{
    // Default meta RO
    let meta = MetaDesc::default();
    // Prefix path with root elem /
    let mut children = Vec::new();
    let path = Vec::new();

    // root object
    if let JsonValue::Object(root) = root {
        for (name, fields) in root {
            if filter_nodes(&name) {
                let mut new_path = path.clone();
                new_path.push(name.clone());
                children.push(visit_tree(&new_path, &name, &fields, meta)?);
            }
        }
    } else {
        Err("Non root object".to_string())?
    };

    Ok(DslTree::SectionV(
        Section {
            path,
            name: "Msg".to_string(),
            meta,
            children,
        }
    ))
}

// Здесь читаем аннтоцаии рута, из них строим особоые права доступа для секции и регистров
// потом в корненвую структуру MetaDesc положим Rc<AccessPriv> у которой поля хэшмапы с именами
// секций/регистров и в каждом вызове extract_meta будем их правильно устнавливать если значения в
// хэше сошлись с именем(путем) фактической ноды
fn parse_root_meta(_root: JsonValue) -> MetaDesc {
    todo!()
}

fn filter_nodes(name: &String) -> bool {
    // filter @annotations
    !name.starts_with(ANNOTATION_TOKEN)
}

fn visit_tree(path: &Vec<String>, name: &String, value: &JsonValue, meta: MetaDesc) -> Result<DslTree, String> {
    Ok(match value {
        JsonValue::Object(fields) => visit_node(path, name, fields, meta)?,
        JsonValue::String(ty_s) => { 
            let ty = ty_convert(ty_s)?;
            visit_leaf(path, name, ty, meta)?
        }
        err_str @ _ => Err(&format!("Unexpected entity in parse tree: {:?}", err_str))?,
    })
}

fn visit_node(path: &Vec<String>, name: &String, fields: &Map<String, JsonValue>, meta: MetaDesc) -> Result<DslTree, String> {
    let meta = meta.extract_update(path, fields);
    
    // Test for nested register definition
    let res = match extract_ty(fields) {
        // It's nested register definition, proceed to creating a leaf
        Some(ty) => {
            visit_leaf(path, name, ty, meta)?
        }
        // None => then it's nested section, so continue recursively
        None => {
            let mut children = Vec::new();
            for (name, keys) in fields {
                if filter_nodes(&name) {
                    let mut new_path = path.clone();
                    new_path.push(name.clone());
                    children.push(
                        visit_tree(&new_path, name, keys, meta)?
                    );
                }
            }

            DslTree::SectionV(Section {
                name: name.clone(),
                path: path.clone(),
                meta,
                children,
            })
        }
    };

    Ok(res)
}

fn visit_leaf(path: &Vec<String>, name: &String, ty: TypeTag, meta: MetaDesc) -> Result<DslTree, String> {

    // WO behaviour for UNIT ty
    let meta = if let TypeTag::UNIT = ty {
        MetaDesc{w: true, r: false, .. meta}
    } else { meta };

    Ok(DslTree::RegisterV(Register {
        name: name.clone(),
        path: path.clone(),
        meta,
        ty,
    }))
}

fn extract_ty(fields: &Map<String, JsonValue>) -> Option<TypeTag> {
    let mut ty = None;
    for (k,v) in fields {
        if k.starts_with(ANNOTATION_TYPE_STR) {
            if let JsonValue::String(tyy) = v {
                ty = Some(ty_convert(tyy).unwrap());
            } else  {
                panic!("Wrong type in @type")
            }
        }
    }
    ty
}

impl MetaDesc {
    fn extract_update(self, _path: &Vec<String>, fields: &Map<String, JsonValue>) -> MetaDesc {
        let mut meta = self;
        for (k,v) in fields {
            if k == ANNOTATION_ACCESS_STR {
                if let JsonValue::String(rights) = v {
                    let Access{ w, r} = access_convert(rights)
                        .expect("Malformed access rights format");
                    meta.w = w;
                    meta.r = r;
                } else  {
                    panic!("Malformed access rights inner type")
                }
            }
            if k == ("@fast") {
                if let JsonValue::Bool(true) = v {
                    meta.fast = true;
                }
            }
        }
        meta
    }
}


fn ty_convert(tytag: &String) -> Result<TypeTag, String> {
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

struct Access {
    w: bool,
    r: bool,
}

fn access_convert(access: &String) -> Result<Access, String> {
    let access = match access.as_str() {
        "WO" => Access{ w: true, r: false },
        "RO" => Access{ w: false, r: true },
        "RW" => Access{ w: true, r: true },
        _    => return Err(format!("Unsupproted access meta: {}", &access)),
    };
    Ok(access)
}


