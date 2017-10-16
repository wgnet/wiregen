// Copyright (C) 2018 Wargaming.net Limited. All rights reserved.

extern crate linear_map;

use std::collections::HashMap;
use self::linear_map::LinearMap;
use syn::{Item, ItemKind, Generics, Ty, Attribute, Path, PathSegment, PathParameters,
          AngleBracketedParameterData, MetaItem, AttrStyle, NestedMetaItem, Lit, StrStyle};
use quote::ToTokens;

//

#[derive(Debug)]
enum ConstExpr {
    F32(f32)
}

//

type TypeName = String;
type FieldName = String;

#[derive(Debug, PartialEq, Eq, Hash)]
enum TypeId {
    Named(TypeName)
}

#[derive(Debug)]
enum FloatIntervalQuantKind {
    Linspace,
    Wrap
}

#[derive(Debug)]
enum FloatQuant {
    None,
    Interval {
        from: ConstExpr,
        to: ConstExpr,
        step: ConstExpr,
        kind: FloatIntervalQuantKind
    },
}

#[derive(Debug)]
enum IntBound {
    None,
    Interval {
        from: ConstExpr,
        to: ConstExpr,
    }
}

#[derive(Debug)]
enum PrimFloatKind {
    F32,
    F64
}

#[derive(Debug)]
enum PrimIntKind {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64
}

#[derive(Debug)]
enum TypeKind {
    PrimFloat{
        base: PrimFloatKind,
        quant: FloatQuant
    },
    PrimInt {
        base: PrimIntKind,
        bound: IntBound
    },
    Struct {
        fields: LinearMap<FieldName, TypeId>
    },
    Dummy
}

#[derive(Debug)]
struct Type {
    name: TypeId,
    kind: TypeKind
}

#[derive(Debug)]
pub struct Model {
    types: HashMap<TypeId, TypeId>
}

//

fn any_generic(g: &Generics) -> bool {
    !g.lifetimes.is_empty() || !g.ty_params.is_empty() || !g.where_clause.predicates.is_empty()
}

fn err_format<I: ToTokens>(i: I, text: &str) -> String {
    format!("{}: {}", text, quote!(#i))
}

fn admissible_path(p: &Ty) -> Result<TypeName, String> {
    if let &Ty::Path(None, Path{ global: false, ref segments }) = p {
        if segments.len() != 1 {
            return Err(err_format(p, "Path segments are not supported"));
        }

        match segments[0] {
            PathSegment{
                ref ident,
                parameters: PathParameters::AngleBracketed(AngleBracketedParameterData{
                    ref lifetimes, ref types, ref bindings
                })
            } if lifetimes.is_empty() && types.is_empty() && bindings.is_empty() => {
                Ok(ident.as_ref().to_owned())
            },
            _ => Err(err_format(p, "Path segment should have no lifetimes, types and bindings"))
        }

    } else {
        Err(err_format(p, "UFC syntax and the like are not supported"))

    }
}

fn outer_attr(attr: &Attribute) -> Result<&MetaItem, ()> {
    match attr {
        &Attribute{ style: AttrStyle::Outer, is_sugared_doc: false, ref value } => Ok(value),
        _ => Err(())
    }
}

fn list_meta_item(meta_item: &MetaItem) -> Result<(&str, &Vec<NestedMetaItem>), ()> {
    match meta_item {
        &MetaItem::List(ref ident, ref nested_items) => Ok((ident.as_ref(), nested_items)),
        _ => Err(())
    }
}

fn list2<F>(vec: &Vec<F>) -> Result<(&F, &F), ()> {
    if vec.len() == 2 {
        Ok((&vec[0], &vec[1]))
    } else {
        Err(())
    }
}

fn str_literal(nmi: &NestedMetaItem) -> Result<&str, ()> {
    match nmi {
        &NestedMetaItem::Literal(Lit::Str(ref lit, StrStyle::Cooked)) => Ok(lit),
        _ => Err(())
    }
}

//
impl IntBound {
    fn try_parse(meta_item: &MetaItem) -> Result<IntBound, ()> {
        let (tag, vec) = list_meta_item(meta_item)?;
        let (p1, p2) = list2(vec)?;
        Ok(IntBound::None)
    }
}

impl PrimFloatKind {
    fn try_parse(s: &str) -> Result<PrimFloatKind, ()> {
        match s {
            "f32" => Ok(PrimFloatKind::F32),
            "f64" => Ok(PrimFloatKind::F64),
            _ => Err(())
        }
    }
}

impl PrimIntKind {
    fn try_parse(s: &str) -> Result<PrimIntKind, ()> {
        match s {
            "u8"  => Ok(PrimIntKind::U8),
            "u16" => Ok(PrimIntKind::U16),
            "u32" => Ok(PrimIntKind::U32),
            "u64" => Ok(PrimIntKind::U64),
            "i8"  => Ok(PrimIntKind::I8),
            "i16" => Ok(PrimIntKind::I16),
            "i32" => Ok(PrimIntKind::I32),
            "i64" => Ok(PrimIntKind::I64),
            _ => Err(())
        }
    }
}

fn type_kind(ty: &Ty, attrs: &Vec<Attribute>) -> Result<TypeKind, String> {
    let prim_int = |path: &str| -> Result<_, ()> {
        let kind = PrimIntKind::try_parse(path)?;
        Ok(TypeKind::PrimInt{ base: kind, bound: IntBound::None })
    };

    let prim_float = |path: &str| -> Result<_, ()> {
        let kind = PrimFloatKind::try_parse(path)?;
        Ok(TypeKind::PrimFloat{ base: kind, quant: FloatQuant::None })
    };

    let prims = || {
        let path = admissible_path(ty)?;

        prim_int(path.as_ref())
            .or_else(|_| prim_float(path.as_ref()))
            .map_err(|_| err_format(ty, "Unsupported primitive type"))
    };

    prims()
}

fn type_alias(item: &Item) -> Option<Result<Type, String>> {
    if let &Item{ node: ItemKind::Ty(ref ty, ref generics), ref ident, ref attrs, .. } = item {

        let r = if any_generic(&generics) {
            Err(err_format(item, "Generics are not allowed"))
        } else {
            type_kind(ty, attrs).map(|tkind| Type{ name: TypeId::Named(ident.as_ref().to_owned()), kind: tkind })
        };

        Some(r)

    } else {
        None

    }

}

//

impl Model {
    fn new() -> Model {
        Model{ types: HashMap::new() }
    }

    pub fn from_crate(krate: Vec<Item>) -> () {
        let m = Model::new();

        for item in &krate {
            if let Some(Ok(x)) = type_alias(item) {
                println!("alias: {:?}", x);
            }
        }

        for item in &krate {
            println!("---");
            println!("{:?}", item);
            println!("{}", quote!(#item));
        }

        ()
    }



}

//
