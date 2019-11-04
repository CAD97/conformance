#![allow(unused)]

use {
    nom::{
        self,
        bytes::complete::{is_not, tag, take_until, take_while1},
        character::complete::{char, line_ending, multispace0, not_line_ending},
        combinator::{all_consuming, iterator},
        error::{context, convert_error, VerboseError},
        multi::{many0, many0_count},
    },
    std::{
        env, fmt,
        fs::File,
        io::prelude::*,
        path::{Path, PathBuf},
    },
};

pub struct Test<'a> {
    pub name: &'a str,
    pub description: &'a str,
    pub input: &'a str,
    pub output: &'a str,
}

impl<'a> Test<'a> {
    pub fn parse(source: &'a str) -> Result<Vec<Test<'a>>, String> {
        type PResult<'a, O> = nom::IResult<&'a str, O, VerboseError<&'a str>>;

        fn tests(source: &str) -> PResult<Vec<Test>> {
            let mut v = vec![];
            let (mut source, _) = multispace0(source)?;
            while !source.is_empty() {
                let (s, test) = test(source)?;
                let (s, _) = multispace0(s)?;
                source = s;
                v.push(test);
            }
            Ok((source, v))
        }

        #[rustfmt::skip]
        fn test(source: &str) -> PResult<Test> {
            context("test", |source: &str| {
                let (source, separator) = {
                    let (s, len) = many0_count(tag("="))(source)?;
                    (s, &source[0..len])
                };
                let (source, _)             = line_ending               (source)?;
                let (source, name)          = is_not("\r\n")        (source)?;
                let (source, _)             = line_ending               (source)?;
                let (source, description)   = take_until(separator) (source)?;
                let (source, _)             = tag(separator)            (source)?;
                let (source, _)             = line_ending               (source)?;
                let (source, input)         = take_until(separator) (source)?;
                let (source, _)             = tag(separator)            (source)?;
                let (source, _)             = line_ending               (source)?;
                let (source, output)        = take_until(separator) (source)?;
                let (source, _)             = tag(separator)            (source)?;
                let (source, _)             = line_ending               (source)?;
                Ok((source, Test { name, description, input, output }))
            })(source)
        }

        match tests(source) {
            Ok((_, tests)) => Ok(tests),
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => Err(convert_error(source, e)),
            Err(nom::Err::Incomplete(_)) => unreachable!(),
        }
    }

    pub fn assert<E>(
        &self,
        tested: impl FnOnce(&str) -> Result<String, E>,
        normalize: impl FnOnce(&str) -> Result<String, E>,
    ) -> Result<(), E> {
        let actual = tested(self.input)?;
        let expected = normalize(self.output)?;
        assert_eq!(actual, expected);
        Ok(())
    }
}
