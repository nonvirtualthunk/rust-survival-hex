
extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;

#[macro_use]
extern crate quote;

//use proc_macro2::TokenStream;
use syn::{DeriveInput, Data, Fields};
use syn::*;

#[proc_macro_derive(WidgetContainer)]
pub fn derive_widget_container(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree.
    let input: DeriveInput = syn::parse(input).unwrap();

    // Used in the quasi-quotation below as `#name`.
    let name = input.ident;

    let exec_tokens = match input.data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    // Expands to an expression like
                    //
                    //      (func)(&mut self.foo);
                    //      (func)(&mut self.bar);
                    //
                    let fnames = fields.named.iter()
                        .filter(|f| if let Type::Path(path) = &f.ty {
//                            println!("{:?} Type: {:?}", f.ident, path.path);
                            if let Some(last_segment) = path.path.segments.last() {
                                last_segment.value().ident.to_string() == "Widget"
                            } else {
                                println!("Wat? No last segment");
                                false
                            }
                        } else {
                            false
                        })
                        .map(|f| { &f.ident });
                    quote! {
                        #(
                            (func)(&mut self.#fnames);
                        )*
                    }
                },
                Fields::Unnamed(ref fields) => {
                    // Expands to an expression like
                    //
                    //      (func)(&mut self.0);
                    //      (func)(&mut self.1);
                    //
                    let indices = 0..fields.unnamed.len();
                    quote! {
                        #(
                            (func)(&self.#indices)
                        )*
                    }
                },
                Fields::Unit => {
                // Unit structs cannot own more than 0 bytes of heap memory.
                    quote!()
                }
            }
        },
        _ => {
            panic!("Can only derive widget container for structs");
        }
    };

    let expanded = quote! {
        impl WidgetContainer for #name {
            fn for_all_widgets<F: FnMut(&mut Widget)>(&mut self, mut func: F) {
                #exec_tokens
            }
        }
    };

    expanded.into()
}