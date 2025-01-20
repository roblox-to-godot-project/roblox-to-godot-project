use proc_macro2::Span;
use syn::{braced, bracketed, meta::ParseNestedMeta, parenthesized, parse::{Parse, ParseStream}, punctuated::Punctuated, spanned::Spanned, token::{self, Brace, Bracket, Comma, Eq, Paren, Semi, Struct}, Attribute, Error, Field, Fields, Generics, Ident, ImplItemFn, LitBool, LitStr, Result, Token, Visibility, WhereClause};

#[derive(Debug)]
pub enum SecurityContext {
    None,
    PluginSecurity,
    LocalUserSecurity,
    RobloxScriptSecurity
}

#[derive(Debug)]
pub struct LuaPropertyData {
    pub name: String,
    pub readonly: bool,
    pub get: Option<String>,
    pub set: Option<String>,
    pub security_context: SecurityContext,
}

#[derive(Debug)]
pub struct LuaFunctionData {
    pub name: String,
    pub virt: bool,
    pub security_context: SecurityContext,
    pub asyn: bool,
}

#[derive(Debug)]
pub enum InstanceContent {
    RustField { rust_field: Field },
    LuaField { lua_field: LuaPropertyData, rust_field: Field },
    RustFunction { rust_function: ImplItemFn },
    LuaFunction { lua_function: LuaFunctionData, rust_function: ImplItemFn }
}

#[derive(Debug)]
pub struct InstanceContents {
    pub brace_token: Brace,
    pub named: Punctuated<InstanceContent, Comma>,
}

#[derive(Debug)]
pub struct Instance {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub struct_token: Struct,
    pub ident: Ident,
    pub generics: Generics,
    pub contents: InstanceContents,
    pub semi_token: Option<Semi>,
}

pub enum InstanceConfigAttr {
    NoClone(bool, Option<Eq>, Span),
    ParentLocked(bool, Option<Eq>, Span),
    Hierarchy(Eq, Bracket, Punctuated<Ident, Token![,]>, Span),
    CustomNew(bool, Option<Eq>, Span)
}

macro_rules! bool_arg {
    ($input:expr => $typ:ident | $span:expr) => {
        match $input.parse::<Token![=]>() {
            Ok(tok) => {
                let b = $input.parse::<LitBool>()?;
                Ok(Self::$typ(b.value(), Some(tok), $span))
            },
            Err(_) => {
                // treat as true
                Ok(Self::$typ(true, None, $span))
            },
        }
    };
}

impl Parse for InstanceConfigAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.parse::<Ident>()?;
        let name = ident.to_string();

        match name.as_str() {
            "no_clone" => { return bool_arg!(input => NoClone | ident.span()) },
            "parent_locked" => { return bool_arg!(input => ParentLocked | ident.span()) },
            "custom_new" => { return bool_arg!(input => CustomNew | ident.span()) },
            "hierarchy" => {
                // this, will be a nightmare.
                let equals = input.parse::<Token![=]>()?;
                let content;
                let brackets = bracketed!(content in input);
                
                let punct: Punctuated<Ident, Token![,]> = Punctuated::parse_terminated(&content)?;

                return Ok(InstanceConfigAttr::Hierarchy(equals, brackets, punct, ident.span()))
            },
            _ => {
                return Err(Error::new(ident.span(), "unknown attribute"))
            }
        }
    }
}

pub struct InstanceConfig {
    pub no_clone: bool,
    pub parent_locked: bool,
    pub hierarchy: Vec<Ident>,
    pub custom_new: bool
}

impl Parse for Instance {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis = input.parse::<Visibility>()?;
        let struct_token = input.parse::<Token![struct]>()?;
        let ident = input.parse::<Ident>()?;
        let generics = input.parse::<Generics>()?;
        let (where_clause, contents, semi_token) = data_struct(input)?;
        Ok(Instance {
            attrs,
            vis,
            struct_token,
            ident,
            generics: Generics {
                where_clause,
                ..generics
            },
            contents,
            semi_token,
        })
    }
}

fn search_attrs_fn(mut func: ImplItemFn) -> Result<InstanceContent> {
    let mut filtered: Vec<(usize, &Attribute)> = func.attrs.iter().enumerate().filter(|(_, a)| a.path().is_ident("method")).collect();

    if filtered.is_empty() {
        return Ok(InstanceContent::RustFunction { rust_function: func })
    }

    if filtered.len() != 1 {
        return Err(Error::new(func.span(), format!("`instance`: expected 1 `method` specifier, got {}", filtered.len())))
    } else {
        let (idx, attr) = filtered.pop().unwrap();

        let (mut name,
            mut virt,
            mut security_context,
            mut asyn): (Option<String>, Option<bool>, Option<SecurityContext>, Option<bool>) = (None, None, None, None);

        if let Err(e) = attr.parse_nested_meta(|nested_meta| {
            let ident = nested_meta.path.require_ident()?;
            let ident = ident.to_string();

            match ident.as_str() {
                "name" => {
                    if name.is_none() {
                        let nname: LitStr = nested_meta.value()?.parse()?;
                        let nname = nname.value();
                        if nname.is_empty() {
                            return Err(nested_meta.error("field name cannot be empty"));
                        }
                        name = Some(nname);
                    } else {
                        return Err(nested_meta.error("already specified"));
                    }
                },
                "virtual" => {
                    if let None = virt {
                        if let Ok(ts) = nested_meta.value() {
                            let b: LitBool = ts.parse()?;
                            virt = Some(b.value());
                        } else {
                            virt = Some(true);
                        }
                    } else {
                        return Err(nested_meta.error("already specified"));
                    }
                },
                "async" => {
                    if let None = asyn {
                        if let Ok(ts) = nested_meta.value() {
                            let b: LitBool = ts.parse()?;
                            asyn = Some(b.value());
                        } else {
                            asyn = Some(true);
                        }
                    } else {
                        return Err(nested_meta.error("already specified"));
                    }
                },
                "security_context" => {
                    if let None = security_context {
                        let value: Ident = nested_meta.value()?.parse()?;

                        let ident = value.to_string();

                        match ident.as_str() {
                            "None" => {
                                security_context = Some(SecurityContext::None);
                            },
                            "PluginSecurity" => {
                                security_context = Some(SecurityContext::PluginSecurity);
                            },
                            "LocalUserSecurity" => {
                                security_context = Some(SecurityContext::LocalUserSecurity);
                            },
                            "RobloxScriptSecurity" => {
                                security_context = Some(SecurityContext::RobloxScriptSecurity);
                            },
                            _ => {
                                return Err(nested_meta.error("unknown secuirty context"));
                            }
                        }
                    } else {
                        return Err(nested_meta.error("already specified"));
                    }
                },
                _ => {
                    return Err(nested_meta.error("unknown attribute"));
                }
            }
            Ok(())
        }) {
            return Err(e)
        };
        
        let lfa = LuaFunctionData {
            name: match name {
                Some(s) => s,
                None => return Err(Error::new(attr.span(), "name must be specified")),
            },
            security_context: security_context.unwrap_or(SecurityContext::None),
            virt: virt.unwrap_or(false),
            asyn: asyn.unwrap_or(false),
        };

        func.attrs.remove(idx);

        return Ok(InstanceContent::LuaFunction { lua_function: lfa, rust_function: func })
    }
}

fn search_attrs_field(mut field: Field) -> Result<InstanceContent> {
    let mut filtered: Vec<(usize, &Attribute)> = field.attrs.iter().enumerate().filter(|(_, a)| a.path().is_ident("property")).collect();

    if filtered.is_empty() {
        return Ok(InstanceContent::RustField { rust_field: field })
    }

    if filtered.len() != 1 {
        return Err(Error::new(field.span(), format!("`instance`: expected 1 `property` specifier, got {}", filtered.len())))
    } else {
        let (idx, attr) = filtered.pop().unwrap();
        let (mut name, mut readonly, mut get, mut set, mut security_context) = (None, None, None, None, None);
        if let Err(e) = attr.parse_nested_meta(|nested_meta| {
            let ident = nested_meta.path.require_ident()?;
            let ident = ident.to_string();

            match ident.as_str() {
                "name" => {
                    if name.is_none() {
                        let nname: LitStr = nested_meta.value()?.parse()?;
                        let nname = nname.value();
                        if nname.is_empty() {
                            return Err(nested_meta.error("field name cannot be empty"));
                        }
                        name = Some(nname);
                    } else {
                        return Err(nested_meta.error("already specified"));
                    }
                },
                "readonly" => {
                    if let None = readonly {
                        if let Ok(ts) = nested_meta.value() {
                            let b: LitBool = ts.parse()?;
                            readonly = Some(b.value());
                        } else {
                            readonly = Some(true);
                        }
                    } else {
                        return Err(nested_meta.error("already specified"));
                    }
                },
                "get" => {
                    if let None = get {
                        let name: LitStr = nested_meta.value()?.parse()?;
                        let name = name.value();
                        if name.is_empty() {
                            return Err(nested_meta.error("getter name cannot be empty"))?;
                        }
                        get = Some(name);
                    } else {
                        return Err(nested_meta.error("already specified"));
                    }
                },
                "set" => {
                    if let None = set {
                        let name: LitStr = nested_meta.value()?.parse()?;
                        let name = name.value();
                        if name.is_empty() {
                            return Err(nested_meta.error("setter name cannot be empty"));
                        }
                        set = Some(name);
                    } else {
                        return Err(nested_meta.error("already specified"));
                    }
                },
                "security_context" => {
                    if let None = security_context {
                        let value: Ident = nested_meta.value()?.parse()?;

                        let ident = value.to_string();

                        match ident.as_str() {
                            "None" => {
                                security_context = Some(SecurityContext::None);
                            },
                            "PluginSecurity" => {
                                security_context = Some(SecurityContext::PluginSecurity);
                            },
                            "LocalUserSecurity" => {
                                security_context = Some(SecurityContext::LocalUserSecurity);
                            },
                            "RobloxScriptSecurity" => {
                                security_context = Some(SecurityContext::RobloxScriptSecurity);
                            },
                            _ => {
                                return Err(nested_meta.error("unknown secuirty context"));
                            }
                        }
                    } else {
                        return Err(nested_meta.error("already specified"));
                    }
                },
                _ => {
                    return Err(nested_meta.error("unknown attribute"));
                }
            }
            Ok(())
        }) {
            return Err(e)
        };

        let lpa = LuaPropertyData {
            name: match name {
                Some(s) => s,
                None => return Err(Error::new(attr.span(), "name must be specified")),
            },
            readonly: readonly.unwrap_or(false),
            get,
            set,
            security_context: security_context.unwrap_or(SecurityContext::None),
        };

        field.attrs.remove(idx);

        return Ok(InstanceContent::LuaField { lua_field: lpa, rust_field: field })
    }
}

impl Parse for InstanceContent {
    fn parse(input: ParseStream) -> Result<Self> {
        let fork = input.fork(); // Create a fork of the input stream
        
        // Try parsing as function first without consuming tokens
        if fork.parse::<ImplItemFn>().is_ok() {
            // If it would succeed, now do the real parse
            let iifn = input.parse::<ImplItemFn>()?;
            return Ok(search_attrs_fn(iifn)?);
        }
        
        // If not a function, try as field
        if let Ok(field) = input.call(Field::parse_named) {
            return Ok(search_attrs_field(field)?);
        }
        
        Err(input.error("unknown struct field/method, you can only have functions and fields"))
    }
}

impl Parse for InstanceContents {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(InstanceContents {
            brace_token: braced!(content in input),
            named: content.parse_terminated(InstanceContent::parse, Token![,])?,
        })
    }
}

pub(crate) fn data_struct(
    input: ParseStream,
) -> Result<(Option<WhereClause>, InstanceContents, Option<Token![;]>)> {
    let mut lookahead = input.lookahead1();
    let mut where_clause = None;
    if lookahead.peek(Token![where]) {
        where_clause = Some(input.parse()?);
        lookahead = input.lookahead1();
    }

    if lookahead.peek(token::Brace) {
        let fields = input.parse()?;
        Ok((where_clause, fields, None))
    } else {
        Err(lookahead.error())
    }
}