use proc_macro::TokenStream;
use syn::{ItemStruct, parse_macro_input, AttributeArgs};
use quote::quote;

/// Add this macro to a struct, and provide a list of other structs to convert the struct into a 
/// PubSub.
/// 
/// # Example
/// 
/// This is a PubSubHub that  publishes `A`, `B`, and `C`.
/// ```compile_fail
/// struct A { }
/// struct B { }
/// struct C { }
/// 
/// #[publishes(A, B, C)]
/// struct PubSubHub { } 
/// ```
/// 
/// After macro expansion, the `PubSubHub` struct now has methods for `subscribe_A`, `subscribe_B`,
/// `publish_A`, etc. There are struct fields for `__subscriptions_A`, etc, which track the
/// subscribers to various events.
/// 
/// # Notes
/// The struct tagged with `publishes` will have additional fields and methods added to it.
/// Existing code will be preserved, so you can add additional fields to your PubSub hub, although
/// none of the generated code will use it.
#[proc_macro_attribute]
pub fn publishes(args: TokenStream, item: TokenStream) -> TokenStream {
    let item_struct: ItemStruct = parse_macro_input!(item as ItemStruct);
    let publishables = parse_macro_input!(args as AttributeArgs);
    let struct_ident = item_struct.ident.clone();

    let names_map = pubsubhub_macros::build_names_map(publishables);

    let all_together = pubsubhub_macros::construct_new_struct(&names_map, struct_ident, item_struct);

    all_together
}

/// `as_any` is a convenience macro that expands to the method
/// ```
/// # use std::any::Any;
/// # struct A { }
/// # impl A {
/// fn as_any(&self) -> &dyn Any {
///     self
/// }
/// # }
/// ```
/// 
/// Use it in a struct implementation to avoid having to write the method for each subscriber,
/// which is required by the `Subscriber` interface provided by the `PubSubHub` module.
#[proc_macro]
pub fn as_any(_item: TokenStream) -> TokenStream {
    quote! {
        fn as_any(&self) -> &dyn std::any::Any { self }
    }.into()
}

mod pubsubhub_macros {
    use std::collections::HashMap;
    use proc_macro::TokenStream;
    use quote::{quote, format_ident};
    use syn::{NestedMeta, Ident, Meta, parse::Parser, ItemStruct};

    pub(super) fn build_names_map(names: Vec<NestedMeta>) -> HashMap<Ident, usize> {
        let mut names_map: HashMap<Ident, usize> = HashMap::new();
        let mut count = 0;
        for i in names {
            match i {
                syn::NestedMeta::Meta(Meta::Path(m)) => {
                    match m.get_ident() {
                        None => panic!("An empty ident is not allowed."),
                        Some(ident) => {
                            if let Some(_) = names_map.get(&ident) {
                                panic!("The structs passed to `publishes` may not be repeated, but `{}` was repeated", ident);
                            }
                            names_map.insert(ident.clone(), count);
                            count += 1;
                        }
                    }
                }
                _ => 
                    panic!("Arguments to `publishes` must be `structs`")
            }
        }
        return names_map;
    }

    pub(super) fn construct_new_struct(names_map: &HashMap<Ident, usize>, struct_ident: Ident, mut item_struct: ItemStruct) -> TokenStream {
        let mut all_impls = quote! {};
        let mut constructor_impl_body = quote! {};

        for pair in names_map {
            let ident = pair.0;
            let subscriptions_ident = format_ident!("__subscriptions_{}", ident);
            let subscribe_fn_ident = format_ident!("subscribe_{}", ident);
            let publish_fn_ident = format_ident!("publish_{}", ident);
            let unsubscribe_fn_ident = format_ident!("unsubscribe_{}", ident);
    
            let struct_field_code = quote! {
                #subscriptions_ident: Vec<std::sync::Arc<std::sync::Mutex<Box<dyn Subscriber<#ident>>>>>
            }.into();
    
            let pubsub_functions_code = quote! {
                impl #struct_ident {
                    #[allow(non_snake_case)]
                    pub fn #subscribe_fn_ident(&mut self, s: Box<dyn Subscriber<#ident>>) -> std::sync::Arc<std::sync::Mutex<Box<dyn Subscriber<#ident>>>> {
                        let arced = std::sync::Arc::new(std::sync::Mutex::new(s));
                        self.#subscriptions_ident.push(arced.clone());
                        return arced;
                    }
    
                    #[allow(non_snake_case)]
                    pub fn #publish_fn_ident(&self, p: &#ident) {
                        for sub_container in self.#subscriptions_ident.iter() {
                            (*sub_container.lock().unwrap().as_mut()).receive(p);
                        }
                    }

                    #[allow(non_snake_case)]
                    fn #unsubscribe_fn_ident(&mut self, s_arc: &std::sync::Arc<std::sync::Mutex<Box<dyn Subscriber<#ident>>>>) {
                        let mut idx_to_remove = None;
                        for (idx, sub_container) in self.#subscriptions_ident.iter().enumerate() {
                            if std::sync::Arc::ptr_eq(&s_arc, sub_container) {
                                idx_to_remove = Some(idx);
                                break;
                            }
                        }
                        if let Some(idx) = idx_to_remove {
                            self.#subscriptions_ident.swap_remove(idx);
                        }
                    }
                }
            };
    
            let struct_init_code = quote! {
                #subscriptions_ident: Vec::new(),
            };
    
            // Now I need to add the new struct code to the struct.
            if let syn::Fields::Named(ref mut fields) = item_struct.fields {
                fields.named.push(
                    syn::Field::parse_named
                        .parse(struct_field_code)
                        .unwrap(),
                );
            }
    
            // Next, save the new impl code so it can be emitted later.
            all_impls = quote!{
                #all_impls
                #pubsub_functions_code
            };
    
            constructor_impl_body = quote!{
                #constructor_impl_body
                #struct_init_code
            };
        }
    
        let all_together = quote!{
            #[allow(non_snake_case)]
            #item_struct
            
            impl #struct_ident {
                pub fn new() -> Self {
                    Self {
                        #constructor_impl_body
                    }
                }
            }
    
            #all_impls
        }.into();
        return all_together;
    }
}