use parserc::{FromSrc, IntoParser, ParseContext, Parser, ParserExt, Result, ensure_char};

use crate::lang::{
    ir::{Fields, Ident, NamedField, Type, UnnamedField},
    parser::{FieldsKind, ParseError},
};

use super::{
    NamedFieldKind,
    utils::{parse_prefix, skip_ws},
};

impl FromSrc for NamedField {
    type Error = ParseError;
    fn parse(ctx: &mut ParseContext<'_>) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let start = ctx.span();

        let (comments, properties) = parse_prefix(ctx)?;

        skip_ws(ctx)?;

        let ident = Ident::into_parser().parse(ctx)?;

        skip_ws.parse(ctx)?;

        ensure_char(':')
            .fatal(ParseError::NamedField(NamedFieldKind::SemiColons))
            .parse(ctx)?;

        skip_ws.parse(ctx)?;

        let ty = Type::into_parser()
            .fatal(ParseError::NamedField(NamedFieldKind::SemiColons))
            .parse(ctx)?;

        Ok(NamedField {
            span: start.extend_to_inclusive(*ty.span()),
            comments,
            properties,
            ident,
            ty,
        })
    }
}

impl FromSrc for UnnamedField {
    type Error = ParseError;
    fn parse(ctx: &mut ParseContext<'_>) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let start = ctx.span();

        let (comments, properties) = parse_prefix(ctx)?;

        skip_ws.parse(ctx)?;

        let ty = Type::into_parser().parse(ctx)?;

        Ok(UnnamedField {
            span: start.extend_to_inclusive(*ty.span()),
            comments,
            properties,
            ty,
        })
    }
}

impl FromSrc for Fields {
    type Error = ParseError;
    fn parse(ctx: &mut parserc::ParseContext<'_>) -> parserc::Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let start = ensure_char('(')
            .map(|span| (span, true))
            .or(ensure_char('{').map(|span| (span, false)))
            .ok()
            .parse(ctx)?;

        let (_, is_tuple) = match start {
            Some(v) => v,
            _ => return Ok(Fields::None),
        };

        if is_tuple {
            let mut fields = vec![];

            while let Some(field) = UnnamedField::into_parser().catch_fatal().parse(ctx)? {
                fields.push(field);

                skip_ws(ctx)?;

                if ensure_char(',').ok().parse(ctx)?.is_none() {
                    break;
                }

                skip_ws.parse(ctx)?;
            }

            ensure_char(')')
                .fatal(ParseError::Fields(FieldsKind::EndTag(')')))
                .parse(ctx)?;

            Ok(Fields::Unnamed(fields))
        } else {
            let mut fields = vec![];

            while let Some(field) = NamedField::into_parser().catch_fatal().parse(ctx)? {
                fields.push(field);

                skip_ws.parse(ctx)?;

                if ensure_char(',').ok().parse(ctx)?.is_none() {
                    break;
                }

                skip_ws.parse(ctx)?;
            }

            ensure_char('}')
                .fatal(ParseError::Fields(FieldsKind::EndTag('}')))
                .parse(ctx)?;

            Ok(Fields::Named(fields))
        }
    }
}

#[cfg(test)]
mod tests {
    use parserc::{FromSrc, ParseContext, Span};

    use crate::lang::ir::{CallExpr, Fields, Ident, LitStr, NamedField, Property, Type};

    #[test]
    fn test_fields() {
        assert_eq!(Fields::parse(&mut ParseContext::from("")), Ok(Fields::None));

        assert_eq!(
            Fields::parse(&mut ParseContext::from(
                "{ name: string, \n#[option,xml('hello')]\nlen: uint,\n}"
            )),
            Ok(Fields::Named(vec![
                NamedField {
                    span: Span::new(1, 13, 1, 2),
                    comments: vec![],
                    properties: vec![],
                    ident: Ident(Span::new(2, 4, 1, 3), "name".to_string()),
                    ty: Type::String(Span::new(8, 6, 1, 9))
                },
                NamedField {
                    span: Span::new(17, 32, 2, 1),
                    comments: vec![],
                    properties: vec![Property {
                        span: Span::new(17, 22, 2, 1),
                        calls: vec![
                            CallExpr {
                                span: Span::new(19, 6, 2, 3),
                                target: Ident(Span::new(19, 6, 2, 3), "option".to_string()),
                                params: vec![]
                            },
                            CallExpr {
                                span: Span::new(26, 12, 2, 10),
                                target: Ident(Span::new(26, 3, 2, 10), "xml".to_string()),
                                params: vec![LitStr(Span::new(30, 7, 2, 14), "hello".to_string())]
                            }
                        ]
                    }],
                    ident: Ident(Span::new(40, 3, 3, 1), "len".to_string()),
                    ty: Type::Uint(Span::new(45, 4, 3, 6))
                }
            ]))
        );
    }
}
