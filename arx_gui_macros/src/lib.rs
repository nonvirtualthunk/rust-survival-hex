extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;

#[macro_use]
extern crate quote;

extern crate itertools;
use itertools::Itertools;
//use proc_macro2::TokenStream;
use syn::{DeriveInput, Data, Fields};
use syn::*;

#[proc_macro_derive(WidgetContainer)]
pub fn derive_widget_container(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the input tokens into a syntax tree.
    let input: DeriveInput = syn::parse(input).unwrap();

    // Used in the quasi-quotation below as `#name`.
    let name = input.ident;

    let (exec_tokens, reapply_all_tokens) = match input.data {
        Data::Struct(ref data) => {
            match data.fields {
                Fields::Named(ref fields) => {
                    // Expands to an expression like
                    //
                    //      (func)(&mut self.foo);
                    //      (func)(&mut self.bar);
                    //
                    let widget_fnames = fields.named.iter()
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
                        .map(|f| { &f.ident })
                        .collect_vec();

                    let container_fnames = fields.named.iter()
                        .filter(|f| if let Type::Path(path) = &f.ty {
//                            println!("{:?} Type: {:?}", f.ident, path.path);
                            if let Some(last_segment) = path.path.segments.last() {
                                last_segment.value().ident.to_string() != "Widget"
                            } else {
                                println!("Wat? No last segment");
                                false
                            }
                        } else {
                            false
                        })
                        .map(|f| { &f.ident })
                        .collect_vec();

                    let widget_fnames_1 = widget_fnames.clone();
                    let container_fnames_1 = container_fnames.clone();
                    (quote! {
                        #(
                            (func)(&mut self.#widget_fnames_1);
                        )*

                        #(
                            //self.#container_fnames.for_each_widget(func);
                            (func)(&mut self.#container_fnames_1.as_widget());
                        )*
                    },
                     quote! {
                        #(
                            self.#widget_fnames.reapply(gui);
                        )*

                        #(
                            //self.#container_fnames.for_each_widget(func);
                            self.#container_fnames.for_each_widget(|w| w.reapply(gui));
                        )*
                    })
                }
                Fields::Unnamed(ref fields) => {
                    // Expands to an expression like
                    //
                    //      (func)(&mut self.0);
                    //      (func)(&mut self.1);
                    //
                    let indices = 0..fields.unnamed.len();
                    let indices_1 = 0..fields.unnamed.len();
                    (quote! {
                        # (
                            (func)(&self.#indices_1)
                        )* },
                     quote! {
                        # (
                            self.#indices.reaply(gui)
                        )*
                    })
                }
                Fields::Unit => {
                    // Unit structs cannot own more than 0 bytes of heap memory.
                    (quote!(), quote!())
                }
            }
        }
        _ => {
            panic!("Can only derive widget container for structs");
        }
    };

    let expanded = quote! {
        impl WidgetContainer for #name {
            fn for_each_widget<F: FnMut(&mut Widget)>(&mut self, mut func: F) {
                #exec_tokens
            }

            fn reapply_all(&mut self, gui : &mut GUI) {
                #reapply_all_tokens
            }
        }
    };

    expanded.into()
}


#[proc_macro_derive(DelegateToWidget)]
pub fn derive_delegate_to_widget(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();

    let name = input.ident;

    let raw = quote! {
impl DelegateToWidget for # name {
fn as_widget( & mut self ) -> & mut Widget { & mut self.body }

fn as_widget_immut( & self ) -> & Widget { & self.body }
}
};

    raw.into()
}