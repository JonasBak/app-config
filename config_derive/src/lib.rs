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
            fn #ident(mut self, value: #ty) -> Self {
                self.#ident = Some(value);
                self
            }
        }
    });
    quote! {
        struct #builder_struct_name {
            #(#declare_fields, )*
        }
        impl #builder_struct_name {
            fn new() -> #builder_struct_name {
                #builder_struct_name {
                    #(#field_defaults, )*
                }
            }
            fn try_build(self) -> Result<#struct_name, Vec<&'static str>> {
                let mut missing_fields = Vec::new();
                #(#check_missing_fields )*
                if missing_fields.len() > 0 {
                    return Err(missing_fields);
                }
                Ok(#struct_name {
                    #(#assign_fields )*
                })
            }
            #(#field_functions )*
        }
    }
}

#[cfg(test)]
mod tests {
}
