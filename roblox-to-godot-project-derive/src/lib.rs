use convert_case::Casing;
use parse::{parse_lua_fn_attr, Instance, InstanceConfig, InstanceConfigAttr, LuaFunctionData, LuaPropertyData};
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{parse::Parser, parse_macro_input, punctuated::Punctuated, spanned::Spanned, Error, Field, Ident, ImplItemFn, ItemImpl, LitStr, Path, PathSegment, Token, TraitBound, TypeParamBound, TypeTraitObject};

mod parse;

macro_rules! error_cattr {
    ($span:expr, $err:literal) => {
        syn::Error::new($span, $err).into_compile_error()
    };
}

#[proc_macro_attribute]
pub fn instance(item: proc_macro::TokenStream, ts: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item: Punctuated<InstanceConfigAttr, Token![,]> = match Punctuated::parse_terminated.parse(item) {
        Ok(s) => s,
        Err(e) => {
            return e.into_compile_error().into()
        }
    };

    // convert punct to conf
    let mut no_clone = None;
    let mut parent_locked = None;
    let mut hierarchy: Option<Vec<syn::Path>> = None;
    let mut custom_new = None;

    for ca in item {
        match ca {
            InstanceConfigAttr::NoClone(b, _, span) => if no_clone.is_none() { no_clone = Some(b) }
                                                                    else { return error_cattr!(span, "`no_clone` specified twice").into() },
            InstanceConfigAttr::ParentLocked(b, _eq, span) => if parent_locked.is_none() { parent_locked = Some(b) }
                                                                    else { return error_cattr!(span, "`parent_locked` specified twice").into() },
            InstanceConfigAttr::Hierarchy(_eq, _bracket, punctuated, span) => if hierarchy.is_none() { hierarchy = Some(punctuated.into_iter().collect()) }
            else { return error_cattr!(span, "`hierarchy` specified twice").into() },
            InstanceConfigAttr::CustomNew(b, _eq, span) => if custom_new.is_none() { custom_new = Some(b) }
                                                                    else { return error_cattr!(span, "`custom_new` specified twice").into() },
        }
    }

    let ic = InstanceConfig {
        no_clone: no_clone.unwrap_or(false),
        parent_locked: parent_locked.unwrap_or(false),
        hierarchy: hierarchy.unwrap_or(vec![]),
        custom_new: custom_new.unwrap_or(false),
    };
    let ts1 = ts.clone(); // temporary
    let mut inst: Instance = parse_macro_input!(ts1);

    let mut lua_fns: Vec<LuaFunctionData> = vec![];
    let mut taken = vec![];
    let mut i = 0;
    while i < inst.attrs.len() {
        if inst.attrs[i].path().is_ident("method") {
            taken.push(inst.attrs.swap_remove(i)); // Swap and remove the matching item
        } else {
            i += 1; // Only increment if no removal
        }
    };

    for attr in taken {
        lua_fns.push(match parse_lua_fn_attr(attr) { Ok(s) => s, Err(e) => return e.into_compile_error().into() });
    }

    let mut rust_fields: Vec<Field> = vec![];
    let mut lua_fields: Vec<LuaPropertyData> = vec![];

    for i in inst.contents.named {
        match i {
            parse::InstanceContent::RustField { rust_field } => rust_fields.push(rust_field),
            parse::InstanceContent::LuaField { lua_field, rust_field } => { lua_fields.push(lua_field); rust_fields.push(rust_field); },
        }
    }

    //let dbg = format!("{inst:?}");

    //let ls = LitStr::new(&dbg, Span::call_site());

    let tso = ts.clone();
    let code = format!("{tso}");

    let code = LitStr::new(&code, Span::call_site());

    let moar_code = format!("{ic:?}");

    let moar = LitStr::new(&moar_code, Span::call_site());

    let lua_fns = format!("{lua_fns:?}");

    let lua_fns = LitStr::new(&lua_fns, Span::call_site());

    let ts: proc_macro2::TokenStream = ts.into();

    let (attr, vis, struct_token, gens, ident) = (inst.attrs, inst.vis, inst.struct_token, inst.generics, inst.ident);

    let component_name = Ident::new(&(ident.to_string() + "Component"), ident.span());
    let trait_name = Ident::new(&("I".to_owned() + &ident.to_string()), ident.span());

    let snake = ident.to_string().to_case(convert_case::Case::Snake);
    let snake_id = Ident::new(&snake, ident.span());
    let mut component_get_name = String::from("get_");
    component_get_name.push_str(&snake);
    component_get_name.push_str("_component");

    let mut component_get_mut_name = String::from("get_");
    component_get_mut_name.push_str(&snake);
    component_get_mut_name.push_str("_component_mut");

    let (cgn, cgmn) = (Ident::new(&component_get_name, ident.span()), Ident::new(&component_get_mut_name, ident.span()));

    let inherited_names: Vec<LitStr> = ic.hierarchy.iter().map(|i| {
        LitStr::new(&i.require_ident().unwrap().to_string(), i.span())
    }).collect();

    let inherited: Vec<syn::Path> = ic.hierarchy.into_iter().into_iter().map(|i| {
        syn::Path {
            leading_colon: i.leading_colon,
            segments: {
                let mut punct: Punctuated<PathSegment, Token![::]> = Punctuated::new();

                let len = i.segments.len();

                for (i, mut element) in i.segments.into_iter().enumerate() {
                    if i < len - 1 {
                        punct.push(element);
                    } else {
                        element.ident = Ident::new(&("I".to_owned() + &element.ident.to_string()), element.ident.span());
                        punct.push(element);
                    }
                }

                punct
            },
        }
    }).collect();

    let s = LitStr::new(&ident.to_string(), ident.span());

    quote! {
        //static DEBUG: &str = #ls;
        static IC: &str = #moar;
        static RECEIVED_CODE: &str = #code;
        static FNS: &str = #lua_fns;
        #(#attr)* #vis #struct_token #gens #component_name {
            #(#rust_fields),*
        }

        impl crate::instance::IInstanceComponent for #component_name {
            fn new(_ptr: instance::WeakManagedInstance, _class_name: &'static str) -> Self {
                todo!()
            }
        }
        
        trait #trait_name {
            fn #cgn(&self) -> crate::core::RwLockReadGuard<'_, #component_name>;
            fn #cgmn(&self) -> crate::core::RwLockWriteGuard<'_, #component_name>;
        }

        struct #ident {
            // base
            instance: crate::core::RwLock<crate::instance::InstanceComponent>,
            // all elements in hierarchy
            service_provider: crate::core::RwLock<crate::instance::ServiceProviderComponent>,
            // self
            #snake_id: crate::core::RwLock<#component_name>,
        }

        impl crate::core::InheritanceBase for #ident {
            fn inheritance_table(&self) -> crate::core::InheritanceTable {
                crate::core::InheritanceTableBuilder::new()
                    .insert_type::<#ident, dyn crate::instance::IObject>(|x| x, |x| x)
                    .insert_type::<#ident, crate::instance::DynInstance>(|x| x, |x| x)
                    #(.insert_type::<#ident, dyn #inherited>(|x| x, |x| x))*
                    .insert_type::<#ident, dyn #trait_name>(|x| x, |x| x)
                    .output()
            }
        }

        impl crate::instance::IObject for #ident {
            fn is_a(&self, class_name: &String) -> bool {
                match class_name.as_str() {
                    "DataModel" => true,
                    //"ServiceProvider" => true,
                    "Instance" => true,
                    "Object" => true,
                    #(#inherited_names => true),*
                    #s => true,
                    _ => false
                }
            }
            fn lua_get(&self, lua: &r2g_mlua::Lua, name: String) -> r2g_mlua::prelude::LuaResult<r2g_mlua::prelude::LuaValue> {
                
            }
            fn get_changed_signal(&self) -> crate::userdata::ManagedRBXScriptSignal {
                use crate::instance::IInstance;
                self.get_instance_component().changed.clone()
            }
            fn get_property_changed_signal(&self, property: String) -> crate::userdata::ManagedRBXScriptSignal {
                use crate::instance::IInstance;
                self.get_instance_component().get_property_changed_signal(property).unwrap()
            }
            fn get_class_name(&self) -> &'static str { #s }
        }

        impl #trait_name for #ident {
            fn #cgn(&self) -> crate::core::RwLockReadGuard<'_, #component_name> {
                self.#snake_id.read().unwrap()
            }

            fn #cgmn(&self) -> crate::core::RwLockWriteGuard<'_, #component_name> {
                self.#snake_id.write().unwrap()
            }
        }

        impl crate::instance::IInstance for #ident {

        }

        impl crate::instance::IServiceProvider for #ident {

        }
    }.into_token_stream().into()
}

// #[proc_macro_attribute]
// pub fn property(_item: proc_macro::TokenStream, ts: proc_macro::TokenStream) -> proc_macro::TokenStream {
//     ts
// }

#[proc_macro_attribute]
pub fn methods(_item: proc_macro::TokenStream, ts: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut impl_block: ItemImpl = parse_macro_input!(ts);

    if let syn::Type::Path(ref p) = *impl_block.self_ty {
        let Some(id) = p.path.get_ident() else { return Error::new(impl_block.self_ty.span(), "name not ident").into_compile_error().into() };
        let ident = "I".to_owned() + &id.to_string();

        let mut path = Punctuated::new();
        path.push(PathSegment { ident: Ident::new(&ident, id.span()), arguments: syn::PathArguments::None });

        let mut p = Punctuated::new();
        p.push(TypeParamBound::Trait(TraitBound { paren_token: None, modifier: syn::TraitBoundModifier::None, lifetimes: None, path: Path { leading_colon: None, segments: path } }));

        impl_block.self_ty = Box::new(syn::Type::TraitObject(TypeTraitObject {
            dyn_token: Some(Token![dyn](id.span())),
            bounds: p,
        }));
       
        return impl_block.to_token_stream().into()
    } else {
        return Error::new(impl_block.self_ty.span(), "unknown type name kind").into_compile_error().into()
    }
}