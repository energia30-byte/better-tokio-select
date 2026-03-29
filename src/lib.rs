#![doc = concat!("[![crates.io](https://img.shields.io/crates/v/", env!("CARGO_PKG_NAME"), "?style=flat-square&logo=rust)](https://crates.io/crates/", env!("CARGO_PKG_NAME"), ")")]
#![doc = concat!("[![docs.rs](https://img.shields.io/docsrs/", env!("CARGO_PKG_NAME"), "?style=flat-square&logo=docs.rs)](https://docs.rs/", env!("CARGO_PKG_NAME"), ")")]
#![doc = "![license](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue?style=flat-square)"]
#![doc = concat!("![msrv](https://img.shields.io/badge/msrv-", env!("CARGO_PKG_RUST_VERSION"), "-blue?style=flat-square&logo=rust)")]
//! [![github](https://img.shields.io/github/stars/nik-rev/better-tokio-select)](https://github.com/nik-rev/better-tokio-select)
//!
//! This crate exports the macro `tokio_select!`, which, unlike [`tokio::select!`](https://docs.rs/tokio/latest/tokio/macro.select.html) -- can be formatted by `rustfmt`.
//!
//! ```toml
#![doc = concat!(env!("CARGO_PKG_NAME"), " = ", "\"", env!("CARGO_PKG_VERSION_MAJOR"), ".", env!("CARGO_PKG_VERSION_MINOR"), "\"")]
//! ```
//!
//! # Examples

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;
use syn::Arm;
use syn::ExprMatch;
use syn::MacroDelimiter;
use syn::Pat;

#[proc_macro_attribute]
pub fn tokio_select(args: TokenStream, input: TokenStream) -> TokenStream {
    let biased_kw = parse_macro_input!(args as Option<kw::biased>).into_iter();
    let mut select_arms = quote! { #(#biased_kw;)* };

    let input = parse_macro_input!(input as ExprMatch);

    for Arm {
        pat, guard, body, ..
    } in input.arms
    {
        match pat {
            Pat::Or(or) if or.cases.len() == 2 => {
                let pat = &or.cases[0];

                let precondition = guard.as_ref().map(|guard| &guard.1).into_iter();

                match &or.cases[1] {
                    Pat::Macro(macr)
                        if macr.mac.path.is_ident("poll")
                            && matches!(macr.mac.delimiter, MacroDelimiter::Paren(_)) =>
                    {
                        let fut = &macr.mac.tokens;

                        select_arms.extend(quote! {
                            #pat = #fut #(, if #precondition)* => #body,
                        });
                    }
                    _ => {
                        return syn::Error::new_spanned(
                            pat,
                            "expected format: pattern | poll!(future)",
                        )
                        .to_compile_error()
                        .into();
                    }
                }
            }
            Pat::Wild(_) => {
                select_arms.extend(quote! {
                    else => #body
                });
            }
            _ => {
                return syn::Error::new_spanned(pat, "expected format: pattern | poll!(future)")
                    .to_compile_error()
                    .into();
            }
        }
    }

    quote! {
        ::tokio::select! {
            #select_arms
        }
    }
    .into()
}

mod kw {
    syn::custom_keyword!(biased);
}
