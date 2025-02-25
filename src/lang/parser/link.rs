use parserc::{
    FromSrc, IntoParser, ParseContext, Parser, ParserExt, Result, ensure_char, ensure_keyword,
};

use crate::lang::ir::{ApplyTo, ChildrenOf, Group, Ident};

use super::{
    ApplyToKind, ChildrenOfKind, GroupKind, ParseError, TupleKind,
    utils::{parse_prefix, skip_ws},
};

fn parse_tuple_idents(ctx: &mut ParseContext<'_>) -> Result<Vec<Ident>, ParseError> {
    ensure_char('(')
        .fatal(ParseError::Tuple(TupleKind::BodyStart))
        .parse(ctx)?;

    skip_ws(ctx)?;

    let mut children = vec![];

    while let Some(ident) = Ident::into_parser().ok().parse(ctx)? {
        children.push(ident);

        skip_ws(ctx)?;

        if ensure_char(',').ok().parse(ctx)?.is_none() {
            break;
        }

        skip_ws(ctx)?;
    }

    ensure_char(')')
        .fatal(ParseError::Tuple(TupleKind::BodyEnd))
        .parse(ctx)?;

    Ok(children)
}

impl FromSrc for Group {
    type Error = ParseError;
    fn parse(ctx: &mut ParseContext<'_>) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let (comments, properties) = parse_prefix(ctx)?;

        skip_ws(ctx)?;

        let start = ensure_keyword("group").parse(ctx)?;

        skip_ws.parse(ctx)?;

        let ident = Ident::into_parser().parse(ctx)?;

        skip_ws.parse(ctx)?;

        ensure_keyword(":=")
            .fatal(ParseError::Group(GroupKind::Assign))
            .parse(ctx)?;

        skip_ws(ctx)?;

        let children = parse_tuple_idents(ctx)?;

        skip_ws(ctx)?;

        let end = ensure_char(';')
            .fatal(ParseError::Group(GroupKind::End))
            .parse(ctx)?;

        Ok(Self {
            comments,
            properties,
            span: start.extend_to_inclusive(end),
            ident,
            children,
        })
    }
}

impl FromSrc for ApplyTo {
    type Error = ParseError;
    fn parse(ctx: &mut ParseContext<'_>) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let (comments, properties) = parse_prefix(ctx)?;

        skip_ws(ctx)?;

        let start = ensure_keyword("apply").parse(ctx)?;

        skip_ws(ctx)?;

        let from = Ident::into_parser()
            .map(|v| vec![v])
            .or(parse_tuple_idents)
            .parse(ctx)?;

        skip_ws(ctx)?;

        ensure_keyword("to")
            .fatal(ParseError::ApplyTo(ApplyToKind::To))
            .parse(ctx)?;

        skip_ws(ctx)?;

        let to = Ident::into_parser()
            .map(|v| vec![v])
            .or(parse_tuple_idents)
            .fatal(ParseError::ApplyTo(ApplyToKind::Target))
            .parse(ctx)?;

        skip_ws(ctx)?;

        let end = ensure_char(';')
            .fatal(ParseError::ApplyTo(ApplyToKind::End))
            .parse(ctx)?;

        Ok(Self {
            properties,
            comments,
            span: start.extend_to_inclusive(end),
            from,
            to,
        })
    }
}

impl FromSrc for ChildrenOf {
    type Error = ParseError;
    fn parse(ctx: &mut ParseContext<'_>) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let (comments, properties) = parse_prefix(ctx)?;

        skip_ws(ctx)?;

        let start = ensure_keyword("children").parse(ctx)?;

        skip_ws(ctx)?;

        let from = Ident::into_parser()
            .map(|v| vec![v])
            .or(parse_tuple_idents)
            .fatal(ParseError::ChildrenOf(ChildrenOfKind::From))
            .parse(ctx)?;

        skip_ws(ctx)?;

        ensure_keyword("of")
            .fatal(ParseError::ChildrenOf(ChildrenOfKind::From))
            .parse(ctx)?;

        skip_ws(ctx)?;

        let to = Ident::into_parser()
            .map(|v| vec![v])
            .or(parse_tuple_idents)
            .fatal(ParseError::ChildrenOf(ChildrenOfKind::To))
            .parse(ctx)?;

        skip_ws(ctx)?;

        let end = ensure_char(';')
            .fatal(ParseError::ChildrenOf(ChildrenOfKind::End))
            .parse(ctx)?;

        Ok(Self {
            properties,
            comments,
            span: start.extend_to_inclusive(end),
            from,
            to,
        })
    }
}
