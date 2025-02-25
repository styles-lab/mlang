use parserc::{
    ControlFlow, FromSrc, IntoParser, ParseContext, Parser, ParserExt, Result, Span, ensure_char,
    ensure_keyword, take_till, take_while,
};

use crate::lang::ir::{Comment, Ident, LitUint, Property, Type};

use super::ParseError;

impl FromSrc for Ident {
    type Error = ParseError;
    fn parse(ctx: &mut parserc::ParseContext<'_>) -> parserc::Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let (c, start) = ctx.next();

        if let Some(c) = c {
            if c != '_' && !c.is_alphabetic() {
                return Err(ControlFlow::Recoverable(ParseError::Ident));
            }
        } else {
            return Err(ControlFlow::Incomplete(ParseError::Ident));
        }

        let body = take_while(|c| c == '_' || c.is_alphanumeric()).parse(ctx)?;

        let span = if let Some(body) = body {
            start.extend_to_inclusive(body)
        } else {
            start
        };

        assert!(span.len() > 0);

        let ident = ctx.as_str(span);

        assert_eq!(ident.len(), span.len());

        Ok(Ident(span, ident.to_string()))
    }
}

pub(super) fn skip_ws(ctx: &mut ParseContext<'_>) -> Result<Option<Span>, ParseError> {
    let span = take_while(|c| c.is_whitespace()).parse(ctx)?;

    Ok(span)
}

impl FromSrc for Comment {
    type Error = ParseError;
    fn parse(input: &mut ParseContext<'_>) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let start = ensure_keyword("///").parse(input)?;

        let content = take_till(|c| c == '\n').parse(input)?;

        let (content, span) = if let Some(content) = content {
            (
                input.as_str(content).trim(),
                start.extend_to_inclusive(content),
            )
        } else {
            ("", start)
        };

        Ok(Comment(span, content.to_string()))
    }
}

impl FromSrc for Type {
    type Error = ParseError;
    fn parse(input: &mut ParseContext<'_>) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let ty = ensure_keyword("bool")
            .map(|span| Type::Bool(span))
            .or(ensure_keyword("string").map(|span| Type::String(span)))
            .or(ensure_keyword("byte").map(|span| Type::Byte(span)))
            .or(ensure_keyword("ubyte").map(|span| Type::Ubyte(span)))
            .or(ensure_keyword("short").map(|span| Type::Short(span)))
            .or(ensure_keyword("ushort").map(|span| Type::Ushort(span)))
            .or(ensure_keyword("int").map(|span| Type::Int(span)))
            .or(ensure_keyword("uint").map(|span| Type::Uint(span)))
            .or(ensure_keyword("long").map(|span| Type::Long(span)))
            .or(ensure_keyword("ulong").map(|span| Type::Ulong(span)))
            .or(ensure_keyword("float").map(|span| Type::Float(span)))
            .or(ensure_keyword("double").map(|span| Type::Double(span)))
            .ok()
            .parse(input)?;

        if let Some(ty) = ty {
            return Ok(ty);
        }

        if let Some(start) = ensure_keyword("vec").ok().parse(input)? {
            skip_ws(input)?;

            ensure_char('[')
                .fatal(ParseError::Type(super::TypeKind::SquareBracketStart))
                .parse(input)?;

            skip_ws(input)?;

            let component = Type::into_parser().parse(input)?;

            skip_ws(input)?;

            let end = ensure_char(']')
                .fatal(ParseError::Type(super::TypeKind::SquareBracketEnd))
                .parse(input)?;

            return Ok(Type::ListOf(
                Box::new(component),
                start.extend_to_inclusive(end),
            ));
        }

        if let Some(start) = ensure_char('[').ok().parse(input)? {
            skip_ws(input)?;

            let component = Type::into_parser().parse(input)?;

            skip_ws(input)?;

            ensure_char(';')
                .fatal(ParseError::Type(super::TypeKind::Semicolons))
                .parse(input)?;

            skip_ws(input)?;

            let len = LitUint::into_parser()
                .fatal(ParseError::Type(super::TypeKind::Uint))
                .parse(input)?;

            skip_ws(input)?;

            let end = ensure_char(']')
                .fatal(ParseError::Type(super::TypeKind::SquareBracketEnd))
                .parse(input)?;

            return Ok(Type::ArrayOf(
                Box::new(component),
                len,
                start.extend_to_inclusive(end),
            ));
        }

        // try parse as ident at last.
        Ident::into_parser()
            .parse(input)
            .map(|ident| Type::Data(ident))
    }
}

pub(super) enum Prefix {
    Property(Property),
    Comment(Comment),
}

pub(super) fn parse_prefix(
    input: &mut ParseContext<'_>,
) -> Result<(Vec<Comment>, Vec<Property>), ParseError> {
    let mut properties = vec![];
    let mut comments = vec![];

    skip_ws(input)?;

    let property_parser = Property::into_parser().map(|p| Prefix::Property(p));

    let comment_parser = Comment::into_parser().map(|p| Prefix::Comment(p));

    while let Some(prefix) = property_parser
        .clone()
        .or(comment_parser.clone())
        .ok()
        .parse(input)?
    {
        match prefix {
            Prefix::Property(property) => properties.push(property),
            Prefix::Comment(comment) => comments.push(comment),
        }

        skip_ws(input)?;
    }

    Ok((comments, properties))
}
