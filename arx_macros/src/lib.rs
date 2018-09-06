#![recursion_limit="128"]

extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;
extern crate itertools;

#[macro_use]
extern crate quote;

use proc_macro2::TokenStream;
use syn::{DeriveInput, Data, Fields};
//use syn::*;
use syn::Visibility;
use itertools::Itertools;

use std::env;

#[proc_macro_derive(Fields)]
pub fn derive_entity_data_fields(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree.
    let input: DeriveInput = syn::parse(input).unwrap();

    let derive_fields = internal_derive_entity_data_fields(&input);
    let derive_visit = internal_derive_visit(&input);

    let struct_name = input.ident.clone();

    let mut auto_include_field_code = false;
    if env::var("PRINT_FIELDS").unwrap_or(String::from("false")) == "true" {
        println!("{}", derive_fields);
    } else if let Ok(path) = env::var("WRITE_FIELDS") {
        use ::std::io::Write;
        use ::std::io::Read;

        println!("Writing fields data for {}", struct_name);
        let mut old_field_file = ::std::fs::File::open(path.clone()).expect("Could not write fields, file did not exist");
        let mut old_field_contents = String::new();
        old_field_file.read_to_string(&mut old_field_contents).expect("could nor write fields, because old file could not be read");
        let mut new_field_contents = old_field_contents.lines().filter(|l| ! l.contains(format!("{}",struct_name).as_str())).join("\n");
        new_field_contents.push_str(format!("\n{}", derive_fields).as_str());

        let mut new_field_file = ::std::fs::File::create(path).expect("Could not write fields, file could not be written to");
        new_field_file.write_all(new_field_contents.as_bytes()).expect("Could not write fields, write failed");
    } else if env::var("GEN_FIELDS").unwrap_or(String::from("false")) == "true" {
        auto_include_field_code = true;
    }

    if auto_include_field_code {
        (quote! {
            #derive_fields
            #derive_visit
        }).into()
    } else {
        derive_visit.into()
    }
}

fn internal_derive_visit(input: &DeriveInput) -> TokenStream {
// Used in the quasi-quotation below as `#struct_name`.
    let struct_name = input.ident.clone();

    let (visit_all_tokens, visit_by_name_tokens): (Vec<TokenStream>,Vec<TokenStream>) = match input.data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    let visit_all_tokens = fields.named.iter()
                        .filter(|f| if let Visibility::Public(_) = f.vis { true } else if let Visibility::Restricted(_) = f.vis { true } else { false })
                        .map(|f| {
                            let field_name = &f.ident;

                            quote! {
                                if let Some(res) = visitor.visit(& #struct_name::#field_name, arg) { return Some(res) }
                            }
                        }).collect();

                    let visit_by_name_tokens = fields.named.iter()
                        .filter(|f| if let Visibility::Public(_) = f.vis { true } else if let Visibility::Restricted(_) = f.vis { true } else { false })
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
                        .filter(|f| if let Visibility::Public(_) = f.vis { true } else if let Visibility::Restricted(_) = f.vis { true } else { false })
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