use parserc::{
    ensure_char, ensure_keyword, FromSrc, IntoParser, ParseContext, Parser, ParserExt, Result,
};

use crate::lang::ir::{ApplyTo, ChildrenOf, Group, Ident};

use super::{
    utils::{parse_prefix, skip_ws},
    ApplyToKind, ChildrenOfKind, GroupKind, ParseError, TupleKind,
};

fn parse_tuple_idents(ctx: &mut ParseContext<'_>) -> Result<Vec<Ident>> {
    ensure_char('(')
        .fatal(ParseError::Tuple(TupleKind::BodyStart), ctx.span())
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
        .fatal(ParseError::Tuple(TupleKind::BodyEnd), ctx.span())
        .parse(ctx)?;

    Ok(children)
}

impl FromSrc for Group {
    fn parse(ctx: &mut ParseContext<'_>) -> Result<Self>
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
            .fatal(ParseError::Group(GroupKind::Assign), ctx.span())
            .parse(ctx)?;

        skip_ws(ctx)?;

        let children = parse_tuple_idents(ctx)?;

        skip_ws(ctx)?;

        let end = ensure_char(';')
            .fatal(ParseError::Group(GroupKind::End), ctx.span())
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
    fn parse(ctx: &mut ParseContext<'_>) -> Result<Self>
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
            .fatal(ParseError::ApplyTo(ApplyToKind::To), ctx.span())
            .parse(ctx)?;

        skip_ws(ctx)?;

        let to = Ident::into_parser()
            .map(|v| vec![v])
            .or(parse_tuple_idents)
            .fatal(ParseError::ApplyTo(ApplyToKind::Target), ctx.span())
            .parse(ctx)?;

        skip_ws(ctx)?;

        let end = ensure_char(';')
            .fatal(ParseError::ApplyTo(ApplyToKind::End), ctx.span())
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
    fn parse(ctx: &mut ParseContext<'_>) -> Result<Self>
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
            .fatal(ParseError::ChildrenOf(ChildrenOfKind::From), ctx.span())
            .parse(ctx)?;

        skip_ws(ctx)?;

        ensure_keyword("of")
            .fatal(ParseError::ChildrenOf(ChildrenOfKind::From), ctx.span())
            .parse(ctx)?;

        skip_ws(ctx)?;

        let to = Ident::into_parser()
            .map(|v| vec![v])
            .or(parse_tuple_idents)
            .fatal(ParseError::ChildrenOf(ChildrenOfKind::To), ctx.span())
            .parse(ctx)?;

        skip_ws(ctx)?;

        let end = ensure_char(';')
            .fatal(ParseError::ChildrenOf(ChildrenOfKind::End), ctx.span())
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
