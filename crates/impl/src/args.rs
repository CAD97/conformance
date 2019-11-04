use {
    syn::{
        self,
        parse::{self, Parse, ParseStream},
    },
};

pub struct Args {
    pub(crate) mode: MatchMode,
    pub(crate) to_string: syn::ExprPath,
    pub(crate) from_str: syn::ExprPath,
    pub(crate) value: syn::Type,
    pub(crate) target: Target,
}

pub(crate) enum MatchMode {
    Exact,
}

pub(crate) enum Target {
    Glob(syn::LitStr),
    File(syn::LitStr),
}

impl Parse for Args {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        mod kw {
            syn::custom_keyword!(serde);
            syn::custom_keyword!(ser);
            syn::custom_keyword!(de);
            syn::custom_keyword!(value);
        }

        let mode = input.parse()?;
        let _: syn::Token![,] = input.parse()?;

        let mut la = input.lookahead1();
        let (to_string, from_str, value) = match () {
            () if la.peek(kw::serde) => {
                let _: kw::serde = input.parse()?;
                let _: syn::Token![=] = input.parse()?;
                let serde: syn::ExprPath = input.parse()?;
                let _: syn::Token![,] = input.parse()?;
                la = input.lookahead1();

                let to_string: syn::ExprPath = match () {
                    () if la.peek(kw::ser) => {
                        let _: kw::ser = input.parse()?;
                        let _: syn::Token![=] = input.parse()?;
                        let to_string: syn::ExprPath = input.parse()?;
                        let _: syn::Token![,] = input.parse()?;
                        la = input.lookahead1();
                        to_string
                    }
                    () => {
                        // FUTURE(rust-lang/rust#64797): use #[cfg(accessible)] to order fallback
                        // #serde::to_string_pretty ; #serde::to_string ; #serde::ser::to_string
                        syn::parse_quote!(#serde::to_string)
                    }
                };

                let from_str: syn::ExprPath = match () {
                    () if la.peek(kw::de) => {
                        let _: kw::de = input.parse()?;
                        let _: syn::Token![=] = input.parse()?;
                        let from_str: syn::ExprPath = input.parse()?;
                        let _: syn::Token![,] = input.parse()?;
                        la = input.lookahead1();
                        from_str
                    }
                    () => {
                        // FUTURE(rust-lang/rust#64797): use #[cfg(accessible)] to order fallback
                        // #serde::from_str ; #serde::de::from_str
                        syn::parse_quote!(#serde::from_str)
                    }
                };

                let value: syn::Type = match () {
                    () if la.peek(kw::value) => {
                        let _: kw::value = input.parse()?;
                        let _: syn::Token![=] = input.parse()?;
                        let value: syn::Type = input.parse()?;
                        let _: syn::Token![,] = input.parse()?;
                        la = input.lookahead1();
                        value
                    }
                    () => {
                        // FUTURE(rust-lang/rust#64797): use #[cfg(accessible)] to order fallback
                        // #serde::Value ; #serde::value::Value
                        syn::parse_quote!(#serde::Value)
                    }
                };

                if input.peek(kw::ser) | input.peek(kw::de) | input.peek(kw::value) {
                    return Err(
                        input.error("arguments must be in order `serde`, `ser`, `de`, `value`")
                    );
                }

                (to_string, from_str, value)
            }
            () if la.peek(kw::ser) => unimplemented!(),
            () => return Err(la.error()),
        };

        let target = Target::parse_with_la(input, la)?;

        Ok(Args {
            mode,
            to_string,
            from_str,
            value,
            target,
        })
    }
}

impl Parse for MatchMode {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        mod kw {
            syn::custom_keyword!(exact);
        }
        let _: kw::exact = input.parse()?;
        Ok(MatchMode::Exact)
    }
}

impl Target {
    fn parse_with_la(input: ParseStream, la: parse::Lookahead1) -> parse::Result<Self> {
        mod kw {
            syn::custom_keyword!(file);
            syn::custom_keyword!(glob);
        }
        match () {
            () if la.peek(kw::file) => {
                let _: kw::file = input.parse()?;
                let _: syn::Token![=] = input.parse()?;
                Ok(Target::File(input.parse()?))
            }
            () if la.peek(kw::glob) => {
                let _: kw::glob = input.parse()?;
                let _: syn::Token![=] = input.parse()?;
                Ok(Target::Glob(input.parse()?))
            }
            () => Err(la.error()),
        }
    }
}
