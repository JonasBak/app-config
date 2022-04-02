extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_macro_input, Data, DeriveInput, Field, Fields, Ident, Lit};

#[proc_macro_derive(AppConfig, attributes(config_field, nested_field))]
pub fn app_config_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident;

    let builder_struct_name = format_ident!("_{}Builder", struct_name);

    let builder_struct =
        declare_impl_builder_struct(&struct_name, &builder_struct_name, &input.data);

    let gen = quote! {
        #builder_struct
        impl #struct_name {
        }
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
        } else {
            quote! {
                #ident: None
            }
        }
    });
    let check_missing_fields = fields.iter().map(|f| {
        let ident = &f.ident;
        if is_nested_field(f) {
            quote! {
                // self.#ident.try_build()?;
            }
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
                pub fn #fn_name(&mut self, prefix: &str) -> Result<(), String> {
                    let prefix = format!("{}_{}", prefix, stringify!(#ident));
                    // self.#ident.from_env_prefixed(&prefix) TODO
                    Ok(())
                }
            }
        } else {
            quote_spanned! {f.span()=>
                pub fn #fn_name(&mut self, prefix: &str) -> Result<(), String> {
                    let env_name = format!("{}_{}", prefix, stringify!(#ident));
                    match std::env::var(&env_name).map(|value| (<#ty as std::str::FromStr>::from_str(&value), value)) {
                        Ok((Ok(value), _)) => {
                            self.#ident = Some(value);
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
        quote! {
            if let Err(e) = self.#fn_name(prefix) {
                failed_fields.push(e);
            }
        }
    });
    quote! {
        #[allow(dead_code)]
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

fn default_field_value(field: &Field) -> Option<Lit> {
    field
        .attrs
        .iter()
        .filter(|attr| attr.path.is_ident("config_field"))
        .filter_map(|attr| attr.parse_args::<syn::MetaNameValue>().ok())
        .filter_map(|meta| {
            if meta.path.is_ident("default") {
                Some(meta.lit)
            } else {
                None
            }
        })
        .next()
}

fn is_nested_field(field: &Field) -> bool {
    field
        .attrs
        .iter()
        .filter(|attr| attr.path.is_ident("nested_field"))
        // .filter_map(|attr| attr.parse_args::<syn::MetaNameValue>().ok())
        // .filter_map(|meta| {
        //     if path.is_ident("nested") {
        //         Some(meta.lit)
        //     } else {
        //         None
        //     }
        // })
        .next()
        .is_some()
}
