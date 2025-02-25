use parserc::{FromSrc, IntoParser, Parser, ParserExt, ensure_char, ensure_keyword};

use crate::lang::{
    ir::{CallExpr, Ident, LitStr, Property},
    parser::{CallKind, ParseError, PropKind, utils::skip_ws},
};

impl FromSrc for CallExpr {
    type Error = ParseError;
    fn parse(ctx: &mut parserc::ParseContext<'_>) -> parserc::Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let start = ctx.span();
        let target = Ident::parse(ctx)?;

        let mut params = vec![];

        skip_ws(ctx)?;

        let end = if let Some(_) = ensure_char('(').ok().parse(ctx)? {
            skip_ws(ctx)?;

            while let Some(call) = LitStr::into_parser().ok().parse(ctx)? {
                params.push(call);

                skip_ws(ctx)?;

                if ensure_char(',').ok().parse(ctx)?.is_none() {
                    break;
                }

                skip_ws(ctx)?;
            }

            ensure_char(')')
                .fatal(ParseError::Call(CallKind::ParamEnd))
                .parse(ctx)?
        } else {
            target.0
        };

        Ok(Self {
            span: start.extend_to_inclusive(end),
            target,
            params,
        })
    }
}

impl FromSrc for Property {
    type Error = ParseError;
    fn parse(ctx: &mut parserc::ParseContext<'_>) -> parserc::Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let start = ensure_keyword("#[").parse(ctx)?;

        let mut params = vec![];

        skip_ws(ctx)?;

        while let Some(call) = CallExpr::into_parser().ok().parse(ctx)? {
            params.push(call);

            skip_ws(ctx)?;

            if ensure_char(',').ok().parse(ctx)?.is_none() {
                break;
            }

            skip_ws(ctx)?;
        }

        let end = ensure_char(']')
            .fatal(ParseError::Prop(PropKind::MissEnd))
            .parse(ctx)?;

        Ok(Self {
            span: start.extend_to_inclusive(end),
            calls: params,
        })
    }
}

#[cfg(test)]
mod tests {
    use parserc::{FromSrc, ParseContext, Span};

    use crate::lang::ir::{CallExpr, Ident, LitStr, Property};

    #[test]
    fn test_props() {
        assert_eq!(
            Property::parse(&mut ParseContext::from("#[hello]")),
            Ok(Property {
                span: Span::new(0, 8, 1, 1),
                calls: vec![CallExpr {
                    span: Span::new(2, 5, 1, 3),
                    target: Ident(Span::new(2, 5, 1, 3), "hello".to_string()),
                    params: vec![]
                }]
            })
        );

        assert_eq!(
            Property::parse(&mut ParseContext::from(
                "#[hello('jjj',\t\n'kkk')\n,hello('jjj',\t\n'kkk')]"
            )),
            Ok(Property {
                span: Span::new(0, 45, 1, 1),
                calls: vec![
                    CallExpr {
                        span: Span::new(2, 20, 1, 3),
                        target: Ident(Span::new(2, 5, 1, 3), "hello".to_string()),
                        params: vec![
                            LitStr(Span::new(8, 5, 1, 9), "jjj".to_string()),
                            LitStr(Span::new(16, 5, 2, 1), "kkk".to_string())
                        ]
                    },
                    CallExpr {
                        span: Span::new(24, 20, 3, 2),
                        target: Ident(Span::new(24, 5, 3, 2), "hello".to_string()),
                        params: vec![
                            LitStr(Span::new(30, 5, 3, 8), "jjj".to_string()),
                            LitStr(Span::new(38, 5, 4, 1), "kkk".to_string())
                        ]
                    }
                ]
            })
        );
    }
}
