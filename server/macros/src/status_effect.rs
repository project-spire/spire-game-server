use proc_macro::TokenStream;
use syn::{parse_macro_input, Data, DeriveInput, Meta};
use quote::quote;

// #[proc_macro_derive(StatusEffect, attributes(buff, debuff, passive, curse))]
// pub fn derive_status_effect(input: TokenStream) -> TokenStream {
//     let input =  parse_macro_input!(input as DeriveInput);
//     let name = input.ident;
//     let variants = match input.data {
//         Data::Enum(e) => e.variants,
//         _ =>  panic!("StatusEffect can only be derived for enums"),
//     };
// 
//     let mut get_type_arms = Vec::new();
//     for variant in variants {
//         let variant_name = &variant.ident;
//         let mut effect_type = None;
// 
//         for attr in variant.attrs {
//             if let Meta::Path(path) = attr.parse_meta().unwrap() {
// 
//             }
//         }
//     }
// 
//     let expanded = quote! {
//         impl #name {
//             pub fn get_type(&self) -> StateEffectType {
//                 match self {
//                     #(#get_type_arms)*
//                 }
//             }
//         }
//     };
// 
//     TokenStream::from(expanded)
// }
