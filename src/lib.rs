extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FnArg, ItemFn, Pat, PatType, PatIdent, Ident, LitStr};
//use crate::utils::jwt_utils::JwtValidator;

#[proc_macro_attribute]
pub fn require_token(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input_fn = parse_macro_input!(item as ItemFn);
    let context_name = parse_macro_input!(attr as Ident);

    let security_ctx_arg:FnArg = syn::parse_quote! { utils::jwt_utils::JwtValidator(#context_name):utils::jwt_utils::JwtValidator } ;
    input_fn.sig.inputs.insert(0, security_ctx_arg);
    let vis = input_fn.vis.clone();
    let sig = input_fn.sig.clone();
    let block = input_fn.block.clone();
    let expanded = quote! {
        #vis
        #sig
        {
            #block
        }
    };

//    println!("EXPANDED: {}", expanded.to_string());

    expanded.into()
}