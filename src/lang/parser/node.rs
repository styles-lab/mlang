use parserc::{
    FromSrc, IntoParser, ParseContext, Parser, ParserExt, Result, ensure_char, ensure_keyword,
};

use crate::lang::{
    ir::{Enum, Fields, Ident, Node, Stat},
    parser::{
        EnumKind, NodeKind, ParseError,
        utils::{parse_prefix, skip_ws},
    },
};

fn parse_node_body(ctx: &mut ParseContext<'_>) -> Result<Node, ParseError> {
    let start = ctx.span();

    let ident = Ident::parse(ctx)?;

    skip_ws(ctx)?;

    let mixin = if let Some(_) = ensure_keyword("mixin").ok().parse(ctx)? {
        skip_ws(ctx)?;
        let mixin = Ident::into_parser()
            .fatal(ParseError::Node(NodeKind::MixinIdent))
            .parse(ctx)?;

        skip_ws(ctx)?;

        Some(mixin)
    } else {
        None
    };

    let fields = Fields::into_parser()
        .fatal(ParseError::Node(NodeKind::Fields))
        .parse(ctx)?;

    let end = ctx.span();

    Ok(Node {
        span: start.extend_to(end),
        comments: vec![],
        mixin,
        properties: vec![],
        ident,
        fields,
    })
}

pub(super) fn parse_node(ctx: &mut ParseContext<'_>) -> Result<Stat, ParseError> {
    let (comments, properties) = parse_prefix(ctx)?;

    let keyword = ensure_keyword("el")
        .or(ensure_keyword("leaf"))
        .or(ensure_keyword("attr"))
        .or(ensure_keyword("data"))
        .or(ensure_keyword("mixin"))
        .parse(ctx)?;

    skip_ws(ctx)?;

    let mut node = parse_node_body(ctx)?;

    node.comments = comments;
    node.properties = properties;

    if node.fields.is_tuple() {
        ensure_char(';')
            .fatal(ParseError::Node(NodeKind::End))
            .parse(ctx)?;
    }

    match ctx.as_str(keyword) {
        "el" => Ok(Stat::Element(Box::new(node))),
        "leaf" => Ok(Stat::Leaf(Box::new(node))),
        "attr" => Ok(Stat::Attr(Box::new(node))),
        "data" => Ok(Stat::Data(Box::new(node))),
        "mixin" => Ok(Stat::Mixin(Box::new(node))),
        _ => panic!("inner errro."),
    }
}

impl FromSrc for Enum {
    type Error = ParseError;
    fn parse(ctx: &mut ParseContext<'_>) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let start = ctx.span();

        let (comments, properties) = parse_prefix(ctx)?;

        ensure_keyword("enum").parse(ctx)?;

        skip_ws(ctx)?;

        let ident = Ident::into_parser()
            .fatal(ParseError::Enum(EnumKind::Ident))
            .parse(ctx)?;

        skip_ws(ctx)?;

        ensure_char('{')
            .fatal(ParseError::Enum(EnumKind::BodyStart))
            .parse(ctx)?;

        let mut fields = vec![];

        loop {
            let (comments, properties) = parse_prefix(ctx)?;

            if let Some(mut field) = parse_node_body.ok().parse(ctx)? {
                field.comments = comments;
                field.properties = properties;

                fields.push(field);

                skip_ws(ctx)?;

                skip_ws.parse(ctx)?;

                if ensure_char(',').ok().parse(ctx)?.is_none() {
                    break;
                }

                skip_ws.parse(ctx)?;
            } else {
                break;
            }
        }

        let end = ensure_char('}')
            .fatal(ParseError::Enum(EnumKind::BodyEnd))
            .parse(ctx)?;

        Ok(Enum {
            span: start.extend_to_inclusive(end),
            comments,
            properties,
            ident,
            fields,
        })
    }
}

#[cfg(test)]
mod tests {
    use parserc::{FromSrc, ParseContext};

    use crate::lang::ir::Enum;

    #[test]
    fn test_enum() {
        Enum::parse(&mut ParseContext::from("enum Hello {a,b,c} ")).unwrap();

        Enum::parse(&mut ParseContext::from(
            "enum Hello { A { value: uint, name: string },b,c} ",
        ))
        .unwrap();
    }
}
