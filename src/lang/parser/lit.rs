use parserc::{
    ControlFlow, FromSrc, Parser, ParserExt, Result, ensure_char, ensure_keyword, take_till,
    take_while,
};

use crate::lang::{
    ir::{LitStr, LitUint},
    parser::{ParseError, UnitKind},
};

impl FromSrc for LitUint {
    type Error = ParseError;
    fn parse(ctx: &mut parserc::ParseContext<'_>) -> parserc::Result<Self, Self::Error>
    where
        Self: Sized,
    {
        if let Some(start) = ensure_keyword("0x").ok().parse(ctx)? {
            let body = take_while(|c| c.is_ascii_hexdigit())
                .parse(ctx)?
                .ok_or(ControlFlow::Fatal(ParseError::Uint(UnitKind::Prefix)))?;

            assert!(body.len() > 0);

            let numeric = ctx.as_str(body);

            assert_eq!(numeric.len(), body.len());

            let numeric = usize::from_str_radix(numeric, 16).unwrap();

            return Ok(Self(numeric, start.extend_to_inclusive(body)));
        }

        let span = take_while(|c| c.is_ascii_digit())
            .parse(ctx)?
            .ok_or(ControlFlow::Fatal(ParseError::Uint(UnitKind::MissBody)))?;

        let numeric = ctx.as_str(span);

        assert_eq!(numeric.len(), span.len());

        let numeric = usize::from_str_radix(numeric, 10).unwrap();

        Ok(Self(numeric, span))
    }
}

impl FromSrc for LitStr {
    type Error = ParseError;
    fn parse(input: &mut parserc::ParseContext<'_>) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let (double_quote, start) = ensure_char('"')
            .map(|span| (true, span))
            .or(ensure_char('\'').map(|span| (false, span)))
            .parse(input)?;

        let end_tag = if double_quote { '"' } else { '\'' };

        let body = take_till(|c| c == end_tag).parse(input)?;

        let end = ensure_char(end_tag).parse(input)?;

        let span = start.extend_to_inclusive(end);

        if let Some(body) = body {
            Ok(Self(span, input.as_str(body).to_string()))
        } else {
            Ok(Self(span, "".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use parserc::{FromSrc, ParseContext, Span};

    use crate::lang::ir::LitUint;

    #[test]
    fn test_num() {
        assert_eq!(
            LitUint::parse(&mut ParseContext::from("134")),
            Ok(LitUint(134, Span::new(0, 3, 1, 1)))
        );

        assert_eq!(
            LitUint::parse(&mut ParseContext::from("0x123")),
            Ok(LitUint(0x123, Span::new(0, 5, 1, 1)))
        );
    }
}
