#![allow(clippy::explicit_counter_loop)]

/// original module: https://github.com/rust-lang/rust/blob/master/compiler/rustc_macros/src/symbols.rs
use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::collections::HashMap;
use syn::parse::{Parse, ParseStream, Result};
use syn::{braced, punctuated::Punctuated, Ident, LitStr, Token};

struct Keyword {
    name: Ident,
    value: LitStr,
}

impl Parse for Keyword {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![:]>()?;
        let value = input.parse()?;

        Ok(Keyword { name, value })
    }
}

pub struct Keywords(Punctuated<Keyword, Token![,]>);

impl Parse for Keywords {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let content;
        braced!(content in input);
        let keywords = content.parse_terminated(Keyword::parse, Token![,])?;

        Ok(Keywords(keywords))
    }
}

#[derive(Default)]
struct Errors {
    list: Vec<syn::Error>,
}

impl Errors {
    fn error(&mut self, span: Span, message: String) {
        self.list.push(syn::Error::new(span, message));
    }
}

pub fn symbols(input: TokenStream) -> TokenStream {
    let (mut output, errors) = symbols_with_errors(input);

    // If we generated any errors, then report them as compiler_error!() macro calls.
    // This lets the errors point back to the most relevant span. It also allows us
    // to report as many errors as we can during a single run.
    output.extend(errors.into_iter().map(|e| e.to_compile_error()));

    output
}

fn symbols_with_errors(input: TokenStream) -> (TokenStream, Vec<syn::Error>) {
    let mut errors = Errors::default();

    let keywords = match syn::parse2::<Keywords>(input) {
        Ok(keywords) => keywords.0,
        Err(e) => {
            // This allows us to display errors at the proper span, while minimizing
            // unrelated errors caused by bailing out (and not generating code).
            errors.list.push(e);
            Default::default()
        }
    };

    let mut keyword_stream = quote! {};
    let mut prefill_stream = quote! {};
    let mut counter = 0u32;
    let mut keys = HashMap::<String, Span>::with_capacity(keywords.len() + 10);
    // let mut prev_key: Option<(Span, String)> = None;

    let mut check_dup = |span: Span, str: &str, errors: &mut Errors| {
        if let Some(prev_span) = keys.get(str) {
            errors.error(span, format!("Symbol `{str}` is duplicated"));
            errors.error(*prev_span, "location of previous definition".to_string());
        } else {
            keys.insert(str.to_string(), span);
        }
    };

    // let mut check_order = |span: Span, str: &str, errors: &mut Errors| {
    //     if let Some((prev_span, ref prev_str)) = prev_key {
    //         if str < prev_str {
    //             errors.error(span, format!("Symbol `{str}` must precede `{prev_str}`"));
    //             errors.error(
    //                 prev_span,
    //                 format!("location of previous symbol `{prev_str}`"),
    //             );
    //         }
    //     }
    //     prev_key = Some((span, str.to_string()));
    // };

    // Generate the listed keywords.
    for keyword in keywords.iter() {
        let name = &keyword.name;
        let value = &keyword.value;
        let value_string = value.value();
        check_dup(keyword.name.span(), &value_string, &mut errors);
        prefill_stream.extend(quote! {
            #value,
        });
        keyword_stream.extend(quote! {
            pub const #name: Symbol = Symbol(#counter);
        });
        counter += 1;
    }

    // Generate symbols for the strings "0", "1", ..., "9".
    // let digits_base = counter;
    // counter += 10;
    // for n in 0..10 {
    //     let n = n.to_string();
    //     check_dup(Span::call_site(), &n, &mut errors);
    //     prefill_stream.extend(quote! {
    //         #n,
    //     });
    // }

    let output = quote! {
        // const SYMBOL_DIGITS_BASE: u32 = #digits_base;
        // const PREINTERNED_SYMBOLS_COUNT: u32 = #counter;

        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        pub mod kw {
            use super::Symbol;
            #keyword_stream
        }

        impl Interner {
            pub(crate) fn fresh() -> Self {
                Interner::prefill(&[
                    #prefill_stream
                ])
            }
        }
    };

    (output, errors.list)
}
