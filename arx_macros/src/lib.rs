#![recursion_limit="128"]

extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;

#[macro_use]
extern crate quote;

use proc_macro2::TokenStream;
use syn::{DeriveInput, Data, Fields};
//use syn::*;
use syn::Visibility;

use std::env;

#[proc_macro_derive(Fields)]
pub fn derive_entity_data_fields(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree.
    let input: DeriveInput = syn::parse(input).unwrap();

    let derive_fields = internal_derive_entity_data_fields(&input);
    let derive_visit = internal_derive_visit(&input);
    (quote! {
        #derive_fields
        #derive_visit
    }).into()
}

#[proc_macro_derive(PrintFields)]
pub fn derive_print_entity_data_fields(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree.
    let input: DeriveInput = syn::parse(input).unwrap();

    if let Ok(value) = env::var("PRINT_FIELDS") {
        if value == "true" || value == input.ident.to_string() {
            println!("{}", internal_derive_entity_data_fields(&input));
        }
    }

    // throw in the visit derivation regardless
    internal_derive_visit(&input).into()
}

fn internal_derive_visit(input: &DeriveInput) -> TokenStream {
// Used in the quasi-quotation below as `#struct_name`.
    let struct_name = input.ident.clone();

    let (visit_all_tokens, visit_by_name_tokens): (Vec<TokenStream>,Vec<TokenStream>) = match input.data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    let visit_all_tokens = fields.named.iter()
                        .filter(|f| if let Visibility::Public(_) = f.vis { true } else { false })
                        .map(|f| {
                            let field_name = &f.ident;

                            quote! {
                                if let Some(res) = visitor.visit(& #struct_name::#field_name, arg) { return Some(res) }
                            }
                        }).collect();

                    let visit_by_name_tokens = fields.named.iter()
                        .filter(|f| if let Visibility::Public(_) = f.vis { true } else { false })
                        .map(|f| {
                            let field_name = &f.ident;

                            quote! {
                                stringify!(#field_name) => visitor.visit(& #struct_name::#field_name, arg),
                            }
                        }).collect();

                    (visit_all_tokens, visit_by_name_tokens)
                },
                _ => (vec![quote! {}], vec![quote!{}])
            }
        },
        _ => {
            panic!("Can only derive widget container for structs");
        }
    };

    quote! {

        impl entity::VisitableFields for #struct_name {
            fn visit_field_named<U, A, V : entity::FieldVisitor<Self, U, A>>(name : &str, visitor : V, arg: &mut A) -> Option<U> {
                match name {
                    #(
                        #visit_by_name_tokens
                    )*
                    _ => None
                }
            }

            fn visit_all_fields<U, A, V : entity::FieldVisitor<Self, U, A>>(visitor : V, arg : &mut A) -> Option<U> {
                #(
                    #visit_all_tokens
                )*
                None
            }
        }
    }
}


fn internal_derive_entity_data_fields(input: &DeriveInput) -> TokenStream {
    // Used in the quasi-quotation below as `#struct_name`.
    let struct_name = input.ident.clone();

    let exec_tokens: Vec<TokenStream> = match input.data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    // Expands to an expression like
                    //
                    //      const foo : Field<TestData,i32> = Field::new("foo",|t| &t.foo, |t,v| { t.foo = v; });
                    //
                    fields.named.iter()
                        .filter(|f| if let Visibility::Public(_) = f.vis { true } else { false })
                        .map(|f| {
                        let field_name = &f.ident;
                        let field_type = &f.ty;

                        quote! {
                            pub const #field_name : Field<#struct_name, #field_type> =
                                Field::new(stringify!(#field_name), |t| &t.#field_name, |t| &mut t.#field_name, |t,v| { t.#field_name = v; });
                        }
                    }).collect()
                },
                _ => vec![quote! {}]
            }
        },
        _ => {
            panic!("Can only derive widget container for structs");
        }
    };


    quote! {
        impl #struct_name {
            #(
                #exec_tokens
            )*
        }
    }
}