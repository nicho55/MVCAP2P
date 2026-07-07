use anyhow::Result;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use syn::{File, Item, ItemEnum, ItemFn, ItemImpl, ItemStruct, ItemTrait, ItemUse};
use walkdir::WalkDir;

#[derive(Debug, Serialize, Clone)]
pub struct ModuleData {
    pub name: String,
    pub path: String,
    pub doc: Vec<String>,
    pub structs: Vec<StructInfo>,
    pub enums: Vec<EnumInfo>,
    pub fns: Vec<FnInfo>,
    pub traits: Vec<TraitInfo>,
    pub impls: Vec<ImplInfo>,
    pub uses: Vec<String>,
    pub submodules: Vec<String>,
    pub bevy_plugins: Vec<String>,
    pub resources: Vec<String>,
    pub components: Vec<String>,
    pub systems: Vec<String>,
    pub events: Vec<String>,
    pub consts: Vec<ConstInfo>,
    pub types: Vec<TypeAliasInfo>,
}

#[derive(Debug, Serialize, Clone)]
pub struct StructInfo {
    pub name: String,
    pub doc: Vec<String>,
    pub fields: Vec<FieldInfo>,
    pub generics: String,
    pub derives: Vec<String>,
    pub attributes: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct FieldInfo {
    pub name: String,
    pub ty: String,
    pub doc: Vec<String>,
    pub attributes: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct EnumInfo {
    pub name: String,
    pub doc: Vec<String>,
    pub variants: Vec<VariantInfo>,
    pub derives: Vec<String>,
    pub generics: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct VariantInfo {
    pub name: String,
    pub doc: Vec<String>,
    pub fields: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct FnInfo {
    pub name: String,
    pub doc: Vec<String>,
    pub inputs: Vec<String>,
    pub output: String,
    pub generics: String,
    pub asyncness: bool,
    pub visibility: String,
    pub is_system: bool,
    pub is_handler: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct TraitInfo {
    pub name: String,
    pub doc: Vec<String>,
    pub methods: Vec<FnInfo>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ImplInfo {
    pub ty: String,
    pub trait_name: Option<String>,
    pub methods: Vec<FnInfo>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ConstInfo {
    pub name: String,
    pub ty: String,
    pub value: String,
    pub doc: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct TypeAliasInfo {
    pub name: String,
    pub ty: String,
    pub doc: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct CrateData {
    pub name: String,
    pub modules: HashMap<String, ModuleData>,
    pub entry_point: Vec<String>,
}

pub fn extract_crate(crate_name: &str, crate_path: &str) -> Result<CrateData> {
    let src_path = Path::new(crate_path).join("src");
    let mut modules = HashMap::new();

    for entry in WalkDir::new(&src_path).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let content = fs::read_to_string(path)?;
        let syntax: File = syn::parse_file(&content)?;
        let rel_path = path
            .strip_prefix(&src_path)
            .unwrap()
            .to_string_lossy()
            .to_string();

        let module_name = rel_path.replace(".rs", "").replace("/", "::");
        let data = extract_module(&rel_path, &syntax);
        modules.insert(module_name.clone(), data);
    }

    Ok(CrateData {
        name: crate_name.to_string(),
        modules,
        entry_point: Vec::new(),
    })
}

fn extract_module(path: &str, syntax: &File) -> ModuleData {
    let mut data = ModuleData {
        name: path.to_string(),
        path: format!("src/{}", path),
        doc: Vec::new(),
        structs: Vec::new(),
        enums: Vec::new(),
        fns: Vec::new(),
        traits: Vec::new(),
        impls: Vec::new(),
        uses: Vec::new(),
        submodules: Vec::new(),
        bevy_plugins: Vec::new(),
        resources: Vec::new(),
        components: Vec::new(),
        systems: Vec::new(),
        events: Vec::new(),
        consts: Vec::new(),
        types: Vec::new(),
    };

    for attr in &syntax.attrs {
        if attr.path().is_ident("doc") {
            if let Some(s) = format_doc_attr(attr) {
                if !s.is_empty() {
                    data.doc.push(s);
                }
            }
        }
    }

    for item in &syntax.items {
        match item {
            Item::Struct(s) => {
                let info = extract_struct(s);
                data.structs.push(info);
            }
            Item::Enum(e) => {
                let info = extract_enum(e);
                data.enums.push(info);
            }
            Item::Fn(f) => {
                let info = extract_fn(f);
                data.fns.push(info);
            }
            Item::Trait(t) => {
                let info = extract_trait(t);
                data.traits.push(info);
            }
            Item::Impl(i) => {
                let info = extract_impl(i);
                data.impls.push(info);
            }
            Item::Use(u) => {
                data.uses.push(format_use(u));
            }
            Item::Mod(m) => {
                if let Some(ref _i) = m.content {
                    data.submodules.push(m.ident.to_string());
                }
            }
            Item::Const(c) => {
                data.consts.push(ConstInfo {
                    name: c.ident.to_string(),
                    ty: quote::quote!(#c.ty).to_string(),
                    value: quote::quote!(#c.expr).to_string(),
                    doc: extract_doc_attrs(&c.attrs),
                });
            }
            Item::Type(t) => {
                data.types.push(TypeAliasInfo {
                    name: t.ident.to_string(),
                    ty: quote::quote!(#t.ty).to_string(),
                    doc: extract_doc_attrs(&t.attrs),
                });
            }
            _ => {}
        }
    }

    // Detect Bevy-specific patterns
    for s in &data.structs {
        if has_derive(&s.derives, "Resource") || has_attr(&s.attributes, "resource") {
            data.resources.push(s.name.clone());
        }
        if has_derive(&s.derives, "Component") || has_attr(&s.attributes, "component") {
            data.components.push(s.name.clone());
        }
        if has_derive(&s.derives, "Event") || has_attr(&s.attributes, "event") {
            data.events.push(s.name.clone());
        }
    }

    // Detect Bevy plugins and systems
    for f in &data.fns {
        if f.name.ends_with("Plugin") && has_trait_impl(&data.impls, &f.name, "Plugin") {
            data.bevy_plugins.push(f.name.clone());
        }
    }
    for i in &data.impls {
        if i.trait_name.as_deref() == Some("Plugin") {
            data.bevy_plugins.push(format!("impl Plugin for {}", i.ty));
        }
    }

    // Mark systems (functions used in add_systems)
    for f in &data.fns {
        if is_system_fn(f) {
            data.systems.push(f.name.clone());
        }
    }

    data
}

fn extract_doc_attrs(attrs: &[syn::Attribute]) -> Vec<String> {
    attrs
        .iter()
        .filter(|a| a.path().is_ident("doc"))
        .filter_map(|a| format_doc_attr(a))
        .collect()
}

fn format_doc_attr(attr: &syn::Attribute) -> Option<String> {
    if attr.path().is_ident("doc") {
        if let syn::Expr::Assign(syn::ExprAssign {
            right: expr, ..
        }) = &attr.meta.require_list().ok()?.parse_args::<syn::Expr>().ok()?
        {
            if let syn::Expr::Lit(lit) = expr.as_ref() {
                if let syn::Lit::Str(s) = &lit.lit {
                    return Some(s.value());
                }
            }
        }
    }
    None
}

fn extract_struct(s: &ItemStruct) -> StructInfo {
    let derives = s
        .attrs
        .iter()
        .filter(|a| {
            let path = a.path();
            path.is_ident("derive")
                || path.is_ident("derive")
        })
        .flat_map(|a| {
            if let syn::Meta::List(list) = &a.meta {
                list.tokens
                    .to_string()
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect::<Vec<_>>()
            } else {
                vec![]
            }
        })
        .collect();

    let fields = match &s.fields {
        syn::Fields::Named(named) => named
            .named
            .iter()
            .map(|f| {
                let ft = &f.ty;
                let fa = &f.attrs;
                FieldInfo {
                    name: f.ident.as_ref().map(|i| i.to_string()).unwrap_or_default(),
                    ty: quote::quote!(#ft).to_string(),
                    doc: extract_doc_attrs(fa),
                    attributes: fa.iter().map(|a| quote::quote!(#a).to_string()).collect(),
                }
            })
            .collect(),
        syn::Fields::Unnamed(_) => vec![],
        syn::Fields::Unit => vec![],
    };

    let sg = &s.generics;
    let sa = &s.attrs;
    StructInfo {
        name: s.ident.to_string(),
        doc: extract_doc_attrs(sa),
        fields,
        generics: quote::quote!(#sg).to_string(),
        derives,
        attributes: sa.iter().map(|a| quote::quote!(#a).to_string()).collect(),
    }
}

fn extract_enum(e: &ItemEnum) -> EnumInfo {
    let ea = &e.attrs;
    let eg = &e.generics;
    EnumInfo {
        name: e.ident.to_string(),
        doc: extract_doc_attrs(ea),
        variants: e
            .variants
            .iter()
            .map(|v| {
                let va = &v.attrs;
                let fields = match &v.fields {
                    syn::Fields::Named(n) => n
                        .named
                        .iter()
                        .map(|f| {
                            let fty = &f.ty;
                            let fname = f.ident.as_ref().map(|i| i.to_string()).unwrap_or_default();
                            format!("{}: {}", fname, quote::quote!(#fty))
                        })
                        .collect(),
                    syn::Fields::Unnamed(u) => u
                        .unnamed
                        .iter()
                        .map(|f| {
                            let fty = &f.ty;
                            quote::quote!(#fty).to_string()
                        })
                        .collect(),
                    syn::Fields::Unit => vec![],
                };
                VariantInfo {
                    name: v.ident.to_string(),
                    doc: extract_doc_attrs(va),
                    fields,
                }
            })
            .collect(),
        derives: e
            .attrs
            .iter()
            .filter(|a| a.path().is_ident("derive"))
            .flat_map(|a| {
                if let syn::Meta::List(list) = &a.meta {
                    list.tokens.to_string().split(',').map(|s| s.trim().to_string()).collect()
                } else {
                    vec![]
                }
            })
            .collect(),
        generics: quote::quote!(#eg).to_string(),
    }
}

fn extract_fn(f: &ItemFn) -> FnInfo {
    let doc = extract_doc_attrs(&f.attrs);
    let inputs: Vec<String> = f
        .sig
        .inputs
        .iter()
        .map(|i| quote::quote!(#i).to_string())
        .collect();
    let output = match &f.sig.output {
        syn::ReturnType::Default => "()".to_string(),
        syn::ReturnType::Type(_, ty) => {
            let t = ty;
            quote::quote!(#t).to_string()
        }
    };
    let fgen = &f.sig.generics;
    let fvis = &f.vis;
    let fn_info = FnInfo {
        name: f.sig.ident.to_string(),
        doc,
        inputs,
        output,
        generics: quote::quote!(#fgen).to_string(),
        asyncness: f.sig.asyncness.is_some(),
        visibility: quote::quote!(#fvis).to_string(),
        is_system: false,
        is_handler: f.attrs.iter().any(|a| a.path().is_ident("handler")),
    };

    FnInfo {
        is_system: is_system_fn(&fn_info),
        ..fn_info
    }
}

fn extract_trait(t: &ItemTrait) -> TraitInfo {
    TraitInfo {
        name: t.ident.to_string(),
        doc: extract_doc_attrs(&t.attrs),
        methods: t
            .items
            .iter()
            .filter_map(|i| {
                if let syn::TraitItem::Fn(m) = i {
                    Some(FnInfo {
                        name: m.sig.ident.to_string(),
                        doc: extract_doc_attrs(&m.attrs),
                        inputs: m.sig.inputs.iter().map(|i| quote::quote!(#i).to_string()).collect(),
                        output: match &m.sig.output {
                            syn::ReturnType::Default => "()".to_string(),
                            syn::ReturnType::Type(_, ty) => quote::quote!(#ty).to_string(),
                        },
                        generics: quote::quote!(#m.sig.generics).to_string(),
                        asyncness: m.sig.asyncness.is_some(),
                        visibility: "pub".to_string(),
                        is_system: false,
                        is_handler: false,
                    })
                } else {
                    None
                }
            })
            .collect(),
    }
}

fn extract_impl(i: &ItemImpl) -> ImplInfo {
    ImplInfo {
        ty: quote::quote!(#i.self_ty).to_string(),
        trait_name: i.trait_.as_ref().map(|(_, t, _)| quote::quote!(#t).to_string()),
        methods: i
            .items
            .iter()
            .filter_map(|item| {
                if let syn::ImplItem::Fn(m) = item {
                    Some(FnInfo {
                        name: m.sig.ident.to_string(),
                        doc: extract_doc_attrs(&m.attrs),
                        inputs: m.sig.inputs.iter().map(|i| quote::quote!(#i).to_string()).collect(),
                        output: match &m.sig.output {
                            syn::ReturnType::Default => "()".to_string(),
                            syn::ReturnType::Type(_, ty) => quote::quote!(#ty).to_string(),
                        },
                        generics: quote::quote!(#m.sig.generics).to_string(),
                        asyncness: m.sig.asyncness.is_some(),
                        visibility: "pub".to_string(),
                        is_system: false,
                        is_handler: false,
                    })
                } else {
                    None
                }
            })
            .collect(),
    }
}

fn format_use(u: &ItemUse) -> String {
    quote::quote!(#u).to_string()
}

fn has_derive(derives: &[String], name: &str) -> bool {
    derives.iter().any(|d| {
        let d = d.trim();
        d == name || d.ends_with(name)
    })
}

fn has_attr(attrs: &[String], name: &str) -> bool {
    attrs.iter().any(|a| a.contains(name))
}

fn is_system_fn(f: &FnInfo) -> bool {
    // Bevy systems have specific parameter patterns
    let sys_indicators = [
        "Res<", "ResMut<", "Query<", "EventReader<", "EventWriter<",
        "Commands", "NextState<", "Gizmos", "ButtonInput<",
    ];
    f.inputs.iter().any(|i| sys_indicators.iter().any(|ind| i.contains(ind)))
        || f.name.contains("_system")
}

fn has_trait_impl(impls: &[ImplInfo], _ty: &str, trait_name: &str) -> bool {
    impls.iter().any(|i| i.trait_name.as_deref() == Some(trait_name))
}