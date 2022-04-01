extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, parse_quote, Data, DeriveInput, Fields, GenericParam, Generics, Ident, Index,
};

#[proc_macro_derive(AppConfig)]
pub fn app_config_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident;

    let builder_struct_name = format_ident!("_{}Builder", struct_name);

    let builder_struct =
        declare_impl_builder_struct(&struct_name, &builder_struct_name, &input.data);

    let gen = quote! {
        #builder_struct
        impl #struct_name {
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
        quote! {
            pub #ident: Option<#ty>
        }
    });
    let field_defaults = fields.iter().map(|f| {
        let ident = &f.ident;
        quote! {
            #ident: None
        }
    });
    let check_missing_fields = fields.iter().map(|f| {
        let ident = &f.ident;
        quote! {
            if self.#ident.is_none() {
                missing_fields.push(stringify!(#ident));
            }
        }
    });
    let assign_fields = fields.iter().map(|f| {
        let ident = &f.ident;
        quote! {
            #ident: self.#ident.unwrap(),
        }
    });
    let field_functions = fields.iter().map(|f| {
        let ty = &f.ty;
        let ident = &f.ident;
        quote! {
            pub fn #ident(mut self, value: #ty) -> Self {
                self.#ident = Some(value);
                self
            }
        }
    });
    let field_from_env_functions = fields.iter().map(|f| {
        let ty = &f.ty;
        let ident = f.ident.as_ref().unwrap();
        let fn_name = format_ident!("{}_from_env", ident);
        let env_name = format!("CONFIG_{}", ident);
        quote! {
            pub fn #fn_name(&mut self) -> Result<(), String> {
                match std::env::var(#env_name).map(|value| (<#ty as std::str::FromStr>::from_str(&value), value)) {
                    Ok((Ok(value), _)) => {
                        self.#ident = Some(value);
                        Ok(())
                    },
                    Ok((Err(_), raw)) => Err(format!(
                        "could not parse environment varaible {}={}",
                        #env_name, raw
                    )),
                    Err(std::env::VarError::NotPresent) => Ok(()),
                    _ => Err(format!("could not read environment varaible {}", #env_name)),
                }
            }
        }
    });
    let load_field_from_env = fields.iter().map(|f| {
        let ident = f.ident.as_ref().unwrap();
        let fn_name = format_ident!("{}_from_env", &ident);
        quote! {
            if let Err(e) = self.#fn_name() {
                failed_fields.push(e);
            }
        }
    });
    quote! {
        struct #builder_struct_name {
            #(#declare_fields, )*
        }
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
            fn from_env(mut self) -> Result<#builder_struct_name, Vec<String>> {
                let mut failed_fields = Vec::new();
                #(#load_field_from_env )*
                if failed_fields.len() > 0 {
                    return Err(failed_fields);
                }
                Ok(self)
            }
            #(#field_functions )*
            #(#field_from_env_functions )*
        }
    }
}

#[cfg(test)]
mod tests {}
