use {
    glob::glob,
    proc_macro2::{Span, TokenStream},
    quote::{quote, quote_spanned},
    runtime::Test,
    simple_interner::{Interned, Interner},
    std::{
        collections::BTreeMap,
        fs::File,
        io::prelude::*,
        path::{Path, PathBuf},
    },
};

fn compile_error(s: impl ToString, span: Span) -> TokenStream {
    let s = s.to_string();
    quote_spanned!(span=> compile_error! { #s })
}

mod args;
pub use args::Args;
use args::*;

pub fn build_libtest_tests(me: &syn::Ident, args: Args, fun: syn::ItemFn, manifest_dir: &Path) -> TokenStream {
    build_libtest_tests_(me, args, fun, manifest_dir).unwrap_or_else(|x| x)
}

// Inner function to allow fatal error early exits with `?`
fn build_libtest_tests_(
    me: &syn::Ident,
    args: Args,
    fun: syn::ItemFn,
    manifest_dir: &Path,
) -> Result<TokenStream, TokenStream> {
    let Args {
        mode,
        to_string,
        from_str,
        value,
        target,
    } = args;
    let fn_name = &fun.sig.ident;

    match mode {
        MatchMode::Exact => (),
    }

    let mut tts = TokenStream::new();

    let mut tests = BTreeMap::new();
    let mut intern = Interner::new();
    let intern = &mut intern;
    let mut read_test = |path: PathBuf, span: Span| -> Result<(), TokenStream> {
        let mut f = File::open(&path).map_err(|e| compile_error(e, span))?;
        let mut s = String::new();
        s.reserve(f.metadata().map(|m| m.len() as usize + 1).unwrap_or(0));
        f.read_to_string(&mut s)
            .map_err(|e| compile_error(e, span))?;
        let path_d = path.display().to_string();
        let s = Interned::get(&intern.get_or_insert(s));
        tests.insert(
            path,
            Test::parse(s)
                .map_err(|e| compile_error(format!("Failed to parse {}: {}", path_d, e), span))?,
        );
        Ok(())
    };

    match target {
        Target::File(lit) => {
            let span = lit.span();
            let path = manifest_dir.join(lit.value());
            read_test(path, span)?;
        }
        Target::Glob(g) => {
            let span = g.span();
            let g = manifest_dir.join(g.value());
            for path in glob(&g.to_string_lossy())
                .map_err(|e| compile_error(format!("Invalid glob: {}", e), span))?
            {
                let path = path.map_err(|e| compile_error(e, span))?;
                match read_test(path, span) {
                    Ok(()) => (),
                    Err(ts) => tts.extend(ts),
                }
            }
        }
    }

    if !tts.is_empty() {
        return Err(tts);
    }

    for (path, tests) in tests {
        let filepath = path.to_string_lossy().to_string();

        tts.extend(quote! { const _: &str = include_str!(#filepath); });

        for Test {
            name,
            description,
            input,
            output,
        } in tests
        {
            let name = name.replace(' ', "_");
            let test_name = quote::format_ident!("{}", name);
            tts.extend(quote! {
                #[test]
                fn #test_name() -> Result<(), Box<dyn ::std::error::Error>> {
                    let test = #me::Test {
                        name: #name,
                        description: #description,
                        input: #input,
                        output: #output,
                    };
                    Ok(test.assert(
                        /* tested */ |s: &str| #to_string(&#fn_name(s)),
                        /* normalize */ |s: &str| #to_string(&#from_str::<#value>(s)?)
                    )?)
                }
            })
        }
    }

    Ok(tts)
}
