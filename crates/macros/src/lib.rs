extern crate proc_macro;
use {
    imp::*,
    proc_macro::TokenStream as TS,
    proc_macro2::{Span, TokenStream},
    proc_macro_crate::crate_name,
    quote::quote_spanned,
    std::{env, path::PathBuf},
    syn,
};

fn compile_error(s: impl ToString, span: Span) -> TokenStream {
    let s = s.to_string();
    quote_spanned!(span=> compile_error! { #s })
}

#[proc_macro_attribute]
pub fn test(attr: TS, item: TS) -> TS {
    // always re-emit the notated item
    let mut tts: TokenStream = item.clone().into();

    // emit as many compile errors at possible at once
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .map_err(|e| {
            compile_error(
                format!("expected $CARGO_MANIFEST_DIR; {}", e),
                Span::call_site(),
            )
        });
    let me = crate_name("conformance")
        .map(|name| quote::format_ident!("{}", name))
        .map_err(|e| compile_error(e, Span::call_site()));
    let args = syn::parse(attr).map_err(|e| e.to_compile_error());
    let fun = syn::parse(item).map_err(|e| e.to_compile_error());

    match (me, args, fun, manifest_dir) {
        (Ok(me), Ok(args), Ok(fun), Ok(manifest_dir)) => {
            tts.extend(build_libtest_tests(&me, args, fun, &manifest_dir))
        }
        (Err(a), Err(b), Err(c), Err(d)) => tts.extend(vec![a, b, c, d]),
        (Err(a), Err(b), Err(c), _)
        | (Err(a), Err(b), _, Err(c))
        | (Err(a), _, Err(b), Err(c))
        | (_, Err(a), Err(b), Err(c)) => tts.extend(vec![a, b, c]),
        (Err(a), Err(b), _, _)
        | (Err(a), _, Err(b), _)
        | (_, Err(a), Err(b), _)
        | (Err(a), _, _, Err(b))
        | (_, Err(a), _, Err(b))
        | (_, _, Err(a), Err(b)) => tts.extend(vec![a, b]),
        (Err(a), _, _, _) | (_, Err(a), _, _) | (_, _, Err(a), _) | (_, _, _, Err(a)) => {
            tts.extend(vec![a])
        }
    }

    tts.into()
}
