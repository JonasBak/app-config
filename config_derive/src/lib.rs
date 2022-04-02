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

    let builder_struct_name = format_ident!("_{}Builder", struct_name);

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
        if is_nested_field(f) {
            quote! {
                pub #ident: <#ty as AppConfig>::Builder
            }
        } else {
            quote! {
                pub #ident: Option<#ty>
            }
        }
    });
    let field_defaults = fields.iter().map(|f| {
        let ty = &f.ty;
        let ident = &f.ident;
        if is_nested_field(f) {
            quote_spanned! {f.span()=>
                #ident: <#ty as AppConfig>::builder()
            }
        } else if let Some(default_value) = default_field_value(f) {
            quote_spanned! {f.span()=>
                #ident: Some(#default_value.into())
            }
        } else if is_optional_field(f).is_some() {
            quote_spanned! {f.span()=>
                #ident: Some(None)
            }
        } else {
            quote! {
                #ident: None
            }
        }
    });
    let check_missing_fields = fields.iter().map(|f| {
        let ident = &f.ident;
        if is_nested_field(f) {
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
        if is_nested_field(f) {
            quote! {
                #ident: self.#ident.try_build()?,
            }
        } else {
            quote! {
                #ident: self.#ident.unwrap(),
            }
        }
    });
    let combine_fields = fields.iter().map(|f| {
        let ident = &f.ident;
        if is_nested_field(f) {
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
        let ident = &f.ident;
        if is_nested_field(f) {
            quote! {
                pub fn #ident(mut self, value: <#ty as AppConfig>::Builder) -> Self {
                    self.#ident = value;
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
        if is_nested_field(f) {
            quote_spanned! {f.span()=>
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
        if is_nested_field(f) {
            quote_spanned! {f.span()=>
                if let Err(mut e) = self.#fn_name(prefix) {
                    failed_fields.append(&mut e);
                }
            }
        } else {
            quote! {
                if let Err(e) = self.#fn_name(prefix) {
                    failed_fields.push(e);
                }
            }
        }
    });
    quote! {
        #[allow(dead_code)]
        #derives
        struct #builder_struct_name {
            #(#declare_fields, )*
        }
        #[allow(dead_code)]
        impl #builder_struct_name {
            pub fn new() -> #builder_struct_name {
                #builder_struct_name {
                    #(#field_defaults, )*
                }
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
            pub fn from_env(mut self) -> Result<#builder_struct_name, Vec<String>> {
                self.from_env_prefixed("CONFIG")
            }
            pub fn from_env_prefixed(mut self, prefix: &str) -> Result<#builder_struct_name, Vec<String>> {
                let mut failed_fields = Vec::new();
                #(#load_field_from_env )*
                if failed_fields.len() > 0 {
                    return Err(failed_fields);
                }
                Ok(self)
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

fn is_nested_field(field: &Field) -> bool {
    field
        .attrs
        .iter()
        .find(|attr| attr.path.is_ident("nested_field"))
        .is_some()
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
