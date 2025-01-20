use parse::{Instance, InstanceConfig, InstanceConfigAttr};
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{parse::Parser, parse_macro_input, punctuated::Punctuated, Ident, LitStr, Token};

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
    let mut hierarchy: Option<Vec<Ident>> = None;
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
    let inst: Instance = parse_macro_input!(ts1);

    let dbg = format!("{inst:?}");

    let ls = LitStr::new(&dbg, Span::call_site());

    let tso = ts.clone();
    let code = format!("{tso}");

    let code = LitStr::new(&code, Span::call_site());

    let ts: proc_macro2::TokenStream = ts.into();

    quote! {
        static DEBUG: &str = #ls;
        static RECEIVED_CODE: &str = #code;
        #ts
    }.into_token_stream().into()
}

// #[proc_macro_attribute]
// pub fn property(_item: proc_macro::TokenStream, ts: proc_macro::TokenStream) -> proc_macro::TokenStream {
//     ts
// }