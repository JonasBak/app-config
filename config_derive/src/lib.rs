extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_macro_input, Attribute, Data, DeriveInput, Field, Fields, Ident, Lit, Type};

#[proc_macro_derive(AppConfig, attributes(builder_derive, config_field, nested_field))]
pub fn app_config_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident;

    let derives = get_builder_derives(&input.attrs);

    let builder_struct_name = format_ident!("{}Builder", struct_name);

    let builder_struct =
        declare_impl_builder_struct(&struct_name, &builder_struct_name, &input.data, derives);

    let gen = quote! {
        #builder_struct
        impl AppConfig for #struct_name {
            type Builder = #builder_struct_name;

            fn builder() -> #builder_struct_name {
                #builder_struct_name::new()
            }
        }
    };
    gen.into()
}

fn declare_impl_builder_struct(
    struct_name: &Ident,
    builder_struct_name: &Ident,
    data: &Data,
    derives: Option<TokenStream>,
) -> TokenStream {
    let fields = match *data {
        Data::Struct(ref data) => match data.fields {
            Fields::Named(ref fields) => &fields.named,
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    };
    let declare_fields = fields.iter().map(|f| {
        let ty = &f.ty;
        let ident = &f.ident;
        if let Some(NestedField::Nested) = is_nested_field(f) {
            quote! {
                pub #ident: <#ty as AppConfig>::Builder,
            }
        } else if let Some(NestedField::NestedOptional(ty)) = is_nested_field(f) {
            quote! {
                pub #ident: <#ty as AppConfig>::Builder,
            }
        } else {
            quote! {
                pub #ident: Option<#ty>,
            }
        }
    });
    let field_empty = fields.iter().map(|f| {
        let ty = &f.ty;
        let ident = &f.ident;
        if let Some(NestedField::Nested) = is_nested_field(f) {
            quote_spanned! {f.span()=>
                #ident: <#ty as AppConfig>::Builder::new(),
            }
        } else if let Some(NestedField::NestedOptional(ty)) = is_nested_field(f) {
            quote_spanned! {f.span()=>
                #ident: <#ty as AppConfig>::Builder::new(),
            }
        } else if is_optional_field(f).is_some() {
            quote_spanned! {f.span()=>
                #ident: Some(None),
            }
        } else {
            quote! {
                #ident: None,
            }
        }
    });
    let field_defaults = fields.iter().map(|f| {
        let ty = &f.ty;
        let ident = &f.ident;
        if let Some(NestedField::Nested) = is_nested_field(f) {
            quote_spanned! {f.span()=>
                #ident: <#ty as AppConfig>::Builder::new_default(),
            }
        } else if let Some(NestedField::NestedOptional(ty)) = is_nested_field(f) {
            quote_spanned! {f.span()=>
                #ident: <#ty as AppConfig>::Builder::new(),
            }
        } else if let Some(default_value) = default_field_value(f) {
            quote_spanned! {f.span()=>
                #ident: Some(#default_value.into()),
            }
        } else if is_optional_field(f).is_some() {
            quote_spanned! {f.span()=>
                #ident: Some(None),
            }
        } else {
            quote! {
                #ident: None,
            }
        }
    });
    let fields_not_set = fields.iter().map(|f| {
        let ident = &f.ident;
        if let Some(NestedField::Nested) = is_nested_field(f) {
            quote! {
                if !self.#ident.is_empty() {
                    return false;
                }
            }
        } else if let Some(NestedField::NestedOptional(_)) = is_nested_field(f) {
            quote! {
                if !self.#ident.is_empty() {
                    return false;
                }
            }
        } else if is_optional_field(f).is_some() {
            quote_spanned! {f.span()=>
                if !self.#ident.as_ref().map(|b| b.is_none()).unwrap_or(true) {
                    return false;
                }
            }
        } else {
            quote! {
                if self.#ident.is_some() {
                    return false;
                }
            }
        }
    });
    let check_missing_fields = fields.iter().map(|f| {
        let ident = &f.ident;
        if let Some(NestedField::Nested) = is_nested_field(f) {
            quote! {}
        } else if let Some(NestedField::NestedOptional(_)) = is_nested_field(f) {
            quote! {}
        } else {
            quote! {
                if self.#ident.is_none() {
                    missing_fields.push(stringify!(#ident));
                }
            }
        }
    });
    let assign_fields = fields.iter().map(|f| {
        let ident = &f.ident;
        if let Some(NestedField::Nested) = is_nested_field(f) {
            quote! {
                #ident: self.#ident.try_build()?,
            }
        } else if let Some(NestedField::NestedOptional(_)) = is_nested_field(f) {
            quote! {
                #ident: if self.#ident.is_empty() {
                        None
                    } else {
                        Some(self.#ident.try_build()?)
                    },
            }
        } else {
            quote! {
                #ident: self.#ident.unwrap(),
            }
        }
    });
    let combine_fields = fields.iter().map(|f| {
        let ident = &f.ident;
        if let Some(NestedField::Nested) = is_nested_field(f) {
            quote! {
                self.#ident = self.#ident.combine(other.#ident);
            }
        } else if let Some(NestedField::NestedOptional(_)) = is_nested_field(f) {
            quote! {
                self.#ident = self.#ident.combine(other.#ident);
            }
        } else {
            quote! {
                self.#ident = self.#ident.or(other.#ident);
            }
        }
    });
    let field_functions = fields.iter().map(|f| {
        let ty = &f.ty;
        let ident = f.ident.as_ref().unwrap();
        let map_ident = format_ident!("map_{}", &ident);
        if let Some(NestedField::Nested) = is_nested_field(f) {
            quote! {
                pub fn #ident(mut self, value: <#ty as AppConfig>::Builder) -> Self {
                    self.#ident = value;
                    self
                }
                pub fn #map_ident(mut self, map: fn(<#ty as AppConfig>::Builder) -> <#ty as AppConfig>::Builder) -> Self {
                    self.#ident = (map)(self.#ident);
                    self
                }
            }
        } else if let Some(NestedField::NestedOptional(ty)) = is_nested_field(f) {
            quote! {
                pub fn #ident(mut self, value: <#ty as AppConfig>::Builder) -> Self {
                    self.#ident = value;
                    self
                }
                pub fn #map_ident(mut self, map: fn(<#ty as AppConfig>::Builder) -> <#ty as AppConfig>::Builder) -> Self {
                    self.#ident = (map)(self.#ident);
                    self
                }
            }
        } else {
            quote! {
                pub fn #ident(mut self, value: #ty) -> Self {
                    self.#ident = Some(value);
                    self
                }
            }
        }
    });
    let field_from_env_functions = fields.iter().map(|f| {
        let ty = &f.ty;
        let ident = f.ident.as_ref().unwrap();
        let fn_name = format_ident!("{}_from_env", ident);
        if let Some(NestedField::Nested) = is_nested_field(f) {
            quote_spanned! {f.span()=>
                pub fn #fn_name(&mut self, prefix: &str) -> Result<(), Vec<String>> {
                    let prefix = format!("{}_{}", prefix, stringify!(#ident));
                    self.#ident = <#ty as AppConfig>::builder().from_env_prefixed(&prefix)?;
                    Ok(())
                }
            }
        } else if let Some(NestedField::NestedOptional(ty)) = is_nested_field(f) {
            quote! {
                pub fn #fn_name(&mut self, prefix: &str) -> Result<(), Vec<String>> {
                    let prefix = format!("{}_{}", prefix, stringify!(#ident));
                    self.#ident = <#ty as AppConfig>::builder().from_env_prefixed(&prefix)?;
                    Ok(())
                }
            }
        } else {
            let (ty, set_value) = if let Some(inner) = is_optional_field(f) {
                let ty = inner;
                let set_value = quote! {
                    self.#ident = Some(Some(value));
                };
                (ty, set_value)
            } else {
                let ty = ty.clone();
                let set_value = quote! {
                    self.#ident = Some(value);
                };
                (ty, set_value)
            };
            quote_spanned! {f.span()=>
                pub fn #fn_name(&mut self, prefix: &str) -> Result<(), String> {
                    let env_name = format!("{}_{}", prefix, stringify!(#ident));
                    match std::env::var(&env_name).map(|value| (<#ty as std::str::FromStr>::from_str(&value), value)) {
                        Ok((Ok(value), _)) => {
                            #set_value
                            Ok(())
                        },
                        Ok((Err(_), raw)) => Err(format!(
                            "could not parse environment varaible {}={}",
                            env_name, raw
                        )),
                        Err(std::env::VarError::NotPresent) => Ok(()),
                        _ => Err(format!("could not read environment varaible {}", env_name)),
                    }
                }
            }
        }
    });
    let load_field_from_env = fields.iter().map(|f| {
        let ident = f.ident.as_ref().unwrap();
        let fn_name = format_ident!("{}_from_env", &ident);
        if let Some(NestedField::Nested) = is_nested_field(f) {
            quote_spanned! {f.span()=>
                if let Err(mut e) = builder.#fn_name(prefix) {
                    failed_fields.append(&mut e);
                }
            }
        } else if let Some(NestedField::NestedOptional(_ty)) = is_nested_field(f) {
            quote! {
                if let Err(mut e) = builder.#fn_name(prefix) {
                    failed_fields.append(&mut e);
                }
            }
        } else {
            quote! {
                if let Err(e) = builder.#fn_name(prefix) {
                    failed_fields.push(e);
                }
            }
        }
    });
    // Some functions (default, from_env, from_env_prefixed) take `self` but just returns
    // a new struct without using or changing `self`. This is because I wanted all "entrypoints"
    // to the builder struct to be `MyConfigStruct::builder()`, so you'd do
    // `MyConfigStruct::builder().default()` instead of `MyConfigStruct::Builder::default()`
    // even though that may be more "correct"
    quote! {
        #[allow(dead_code)]
        #derives
        struct #builder_struct_name {
            #(#declare_fields )*
        }
        #[allow(dead_code)]
        impl #builder_struct_name {
            pub fn new() -> #builder_struct_name {
                #builder_struct_name {
                    #(#field_empty )*
                }
            }
            pub fn new_default() -> #builder_struct_name {
                #builder_struct_name {
                    #(#field_defaults )*
                }
            }
            pub fn default(self) -> #builder_struct_name {
                Self::new_default()
            }
            pub fn is_empty(&self) -> bool {
                #(#fields_not_set )*
                true
            }
            pub fn try_build(self) -> Result<#struct_name, Vec<&'static str>> {
                let mut missing_fields = Vec::new();
                #(#check_missing_fields )*
                if missing_fields.len() > 0 {
                    return Err(missing_fields);
                }
                Ok(#struct_name {
                    #(#assign_fields )*
                })
            }
            pub fn combine(mut self, other: Self) -> Self {
                #(#combine_fields )*
                self

            }
            #(#field_functions )*
            #(#field_from_env_functions )*
            pub fn new_from_env() -> Result<#builder_struct_name, Vec<String>> {
                Self::new_from_env_prefixed("CONFIG")
            }
            pub fn from_env(self) -> Result<#builder_struct_name, Vec<String>> {
                Self::new_from_env()
            }
            pub fn new_from_env_prefixed(prefix: &str) -> Result<#builder_struct_name, Vec<String>> {
                let mut builder = #builder_struct_name::new();
                let mut failed_fields = Vec::new();
                #(#load_field_from_env )*
                if failed_fields.len() > 0 {
                    return Err(failed_fields);
                }
                Ok(builder)
            }
            pub fn from_env_prefixed(self, prefix: &str) -> Result<#builder_struct_name, Vec<String>> {
                Self::new_from_env_prefixed(prefix)
            }
        }
    }
}

fn get_builder_derives(attrs: &Vec<Attribute>) -> Option<TokenStream> {
    if let Some(attr) = attrs
        .iter()
        .find(|attr| attr.path.is_ident("builder_derive"))
    {
        let tokens = &attr.tokens;
        let derives = quote! {
            #[derive #tokens]
        };
        Some(derives)
    } else {
        None
    }
}

fn default_field_value(field: &Field) -> Option<Lit> {
    field
        .attrs
        .iter()
        .filter(|attr| attr.path.is_ident("config_field"))
        .filter_map(|attr| attr.parse_args::<syn::MetaNameValue>().ok())
        .find_map(|meta| {
            if meta.path.is_ident("default") {
                Some(meta.lit)
            } else {
                None
            }
        })
}

enum NestedField {
    Nested,
    NestedOptional(Type),
}

fn is_nested_field(field: &Field) -> Option<NestedField> {
    field
        .attrs
        .iter()
        .find(|attr| attr.path.is_ident("nested_field"))
        .map(|_| match is_optional_field(field) {
            Some(ty) => NestedField::NestedOptional(ty),
            None => NestedField::Nested,
        })
}

fn is_optional_field(field: &Field) -> Option<Type> {
    match &field.ty {
        Type::Path(type_path) => {
            match type_path
                .path
                .segments
                .first()
                .map(|s| (&s.ident, &s.arguments))
            {
                Some((
                    ident,
                    syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                        args,
                        ..
                    }),
                )) if ident == "Option" => match args.first() {
                    Some(syn::GenericArgument::Type(ty)) => Some(ty.clone()),
                    _ => None,
                },
                _ => None,
            }
        }
        _ => None,
    }
}
