//! semantic analyzer for `mlang`.

use std::collections::HashMap;

use parserc::Span;

use super::ir::*;

const ANALYZER_ERROR: &str = "MLANG_ANALYZER";

/// Error report by semantic analyze step.
#[derive(Debug, thiserror::Error)]
pub enum AnalyzerError {
    #[error("duplicate symbol `{0}`, previous declaration is here {1}")]
    Duplicate(String, Span),

    #[error("Unknown symbol `{0}`.")]
    Unknown(String),

    #[error("Use group `{0}` as field type declaration, group declaration is here {1}")]
    Group(String, Span),

    #[error("Unable merge mixin({0})'s fields into node, mixin declaration is here {1}.")]
    Merge(String, Span),

    #[error("Custom property `{0}`, expect empty call list.")]
    VariableOption(String),

    #[error("Custom property `rename`, expect one `literial str` as call list.")]
    Rename,
}

#[derive(Default)]
struct SymbolTable(HashMap<String, (Span, usize)>);

impl SymbolTable {
    /// Add a new symbol to the checker.
    fn add(&mut self, index: usize, ident: &Ident) -> bool {
        if let Some((span, _)) = self.0.insert(ident.1.clone(), (ident.0, index)) {
            log::error!(target: ANALYZER_ERROR, span:serde; "{}", AnalyzerError::Duplicate(ident.1.clone(), span));
            false
        } else {
            true
        }
    }

    /// Search symbol.
    fn lookup(&self, ident: &Ident) -> Option<usize> {
        self.0.get(&ident.1).map(|(_, index)| *index)
    }
}

#[derive(Default)]
struct MixinTable {
    mixin: HashMap<String, (Span, usize)>,
}

impl MixinTable {
    /// add new mixin item.
    ///
    /// This function delegate symbol conflict check to `SymbolChecker`
    fn add(&mut self, index: usize, ident: &Ident) {
        self.mixin.insert(ident.1.clone(), (ident.0, index));
    }

    fn lookup(&self, ident: &Ident) -> Option<usize> {
        self.mixin.get(ident.1.as_str()).map(|(_, index)| *index)
    }
}

#[derive(Default)]
struct GroupTable {
    groups: HashMap<String, (Span, usize)>,
}

impl GroupTable {
    /// See a group item.
    fn add(&mut self, index: usize, ident: &Ident) {
        self.groups.insert(ident.1.clone(), (ident.0, index));
    }

    fn lookup(&self, ident: &Ident) -> Option<usize> {
        self.groups.get(ident.1.as_str()).map(|(_, index)| *index)
    }
}

/// A semantic analyzer for `mlang`.
#[derive(Default)]
struct SemanticAnalyzer {
    /// A symbol index database.
    symbol_table: SymbolTable,
    /// A mixin fields merger.
    merger: MixinTable,
    /// `apply..to..` `chidlren..of..` syntax checker.
    digraph_analyzer: GroupTable,
    /// report errors.
    errors: usize,
}

impl SemanticAnalyzer {
    fn analyze(mut self, opcodes: &mut [Stat]) -> bool {
        self.build_index(opcodes);
        self.check(opcodes);
        self.errors == 0
    }

    fn build_index(&mut self, opcodes: &mut [Stat]) {
        for (index, opcode) in opcodes.iter().enumerate() {
            match opcode {
                Stat::Element(node) | Stat::Leaf(node) | Stat::Attr(node) | Stat::Data(node) => {
                    if !self.symbol_table.add(index, &node.ident) {
                        self.errors += 1;
                    }
                }
                Stat::Mixin(node) => {
                    if !self.symbol_table.add(index, &node.ident) {
                        self.errors += 1;
                    }
                    self.merger.add(index, &node.ident);
                }
                Stat::Enum(node) => {
                    if !self.symbol_table.add(index, &node.ident) {
                        self.errors += 1;
                    }
                }
                Stat::Group(node) => {
                    if !self.symbol_table.add(index, &node.ident) {
                        self.errors += 1;
                    }
                    self.digraph_analyzer.add(index, &node.ident);
                }
                Stat::ApplyTo(_) => {}
                Stat::ChildrenOf(_) => {}
            }
        }
    }

    fn check(&mut self, opcodes: &mut [Stat]) {
        let mut updates = vec![];
        for (index, opcode) in opcodes.iter().enumerate() {
            match opcode {
                Stat::Element(node) => {
                    if let Some(node) = self.node_check(opcodes, node) {
                        updates.push((index, Stat::Element(Box::new(node))));
                    }
                }
                Stat::Leaf(node) => {
                    if let Some(node) = self.node_check(opcodes, node) {
                        updates.push((index, Stat::Leaf(Box::new(node))));
                    }
                }
                Stat::Attr(node) => {
                    if let Some(node) = self.node_check(opcodes, node) {
                        updates.push((index, Stat::Attr(Box::new(node))));
                    }
                }
                Stat::Mixin(node) => {
                    assert_eq!(
                        self.node_check(opcodes, node),
                        None,
                        "Mixin: inner error, mixin can't mixin other one."
                    );
                }
                Stat::Data(node) => {
                    if let Some(node) = self.node_check(opcodes, node) {
                        updates.push((index, Stat::Data(Box::new(node))));
                    }
                }
                Stat::Enum(node) => {
                    self.enum_check(opcodes, node);
                }
                Stat::Group(group) => {
                    self.group_check(opcodes, group);
                }
                Stat::ApplyTo(apply_to) => {
                    if let Some(opcode) = self.apply_to_check(opcodes, apply_to) {
                        updates.push((index, opcode));
                    }
                }
                Stat::ChildrenOf(children_of) => {
                    if let Some(opcode) = self.children_of_check(opcodes, children_of) {
                        updates.push((index, opcode));
                    }
                }
            }
        }

        for (index, update) in updates {
            opcodes[index] = update;
        }
    }

    fn symbol_check(&mut self, opcodes: &[Stat], ident: &Ident, expect_type: bool) -> bool {
        if let Some(index) = self.symbol_table.lookup(ident) {
            if let Stat::Group(group) = &opcodes[index] {
                if expect_type {
                    self.errors += 1;
                    log::error!(
                        target: ANALYZER_ERROR, span:serde = ident.0;
                        "{}", AnalyzerError::Group(group.ident.1.clone(), group.ident.0)
                    );
                }

                return false;
            }

            return true;
        } else {
            self.errors += 1;
            log::error!(
                target: ANALYZER_ERROR, span:serde = ident.0;
                "{}", AnalyzerError::Unknown(ident.1.clone())
            );
            return false;
        }
    }

    fn type_check(&mut self, opcodes: &[Stat], ty: &Type) {
        match ty {
            Type::Data(ident) => {
                self.symbol_check(opcodes, ident, true);
            }

            Type::ListOf(component, _) => {
                self.type_check(opcodes, component);
            }
            Type::ArrayOf(component, _, _) => {
                self.type_check(opcodes, component);
            }
            _ => {}
        }
    }

    fn node_check(&mut self, opcodes: &[Stat], node: &Node) -> Option<Node> {
        for field in node.fields.iter() {
            self.type_check(opcodes, &field.ty());
        }

        for property in &node.properties {
            for call in &property.calls {
                match call.target.1.as_str() {
                    "option" | "variable" | "init" => {
                        if call.params.len() != 0 {
                            self.errors += 1;
                            log::error!(
                                target: ANALYZER_ERROR, span:serde = call.target.0;
                                "{}", AnalyzerError::VariableOption(call.target.1.clone())
                            );
                        }
                    }
                    "rename" => {
                        if call.params.len() != 1 {
                            log::error!(
                                target: ANALYZER_ERROR,
                                span:serde = call.target.0; "{}", AnalyzerError::Rename
                            );
                        }
                    }
                    _ => {}
                }
            }
        }

        if let Some(mixin) = &node.mixin {
            if let Some(index) = self.merger.lookup(mixin) {
                if let Stat::Mixin(mixin) = &opcodes[index] {
                    let expand = mixin.fields.clone();
                    let fields = node.fields.clone();

                    let fields = match fields.append(expand) {
                        Ok(fields) => fields,
                        Err(fields) => {
                            log::error!(
                                target: ANALYZER_ERROR,
                                span:serde = node.ident.0;
                                "{}",
                                AnalyzerError::Merge(mixin.ident.1.clone(), mixin.ident.0)
                            );
                            fields
                        }
                    };

                    return Some(Node {
                        span: node.span,
                        comments: node.comments.clone(),
                        mixin: None,
                        properties: node.properties.clone(),
                        ident: node.ident.clone(),
                        fields,
                    });
                } else {
                    panic!("node_check(mxin): inner error.");
                }
            } else {
                log::error!(
                    target: ANALYZER_ERROR,
                    span:serde = mixin.0; "{}", AnalyzerError::Unknown(mixin.1.clone())
                );
                return None;
            }
        }

        return None;
    }

    fn enum_check(&mut self, opcodes: &[Stat], node: &Enum) {
        for field_node in &node.fields {
            for field in field_node.fields.iter() {
                self.type_check(opcodes, field.ty());
            }
        }
    }

    fn group_check(&mut self, opcodes: &[Stat], node: &Group) {
        for ident in &node.children {
            self.symbol_check(opcodes, ident, true);
        }
    }

    fn expand_with_group(&self, opcodes: &[Stat], ident: &Ident) -> Option<Vec<Ident>> {
        if let Some(index) = self.digraph_analyzer.lookup(ident) {
            if let Stat::Group(group) = &opcodes[index] {
                return Some(group.children.clone());
            } else {
                panic!("expand_with_group: inner error.");
            }
        } else {
            log::error!(
                target: ANALYZER_ERROR,
                span:serde = ident.0; "{}", AnalyzerError::Unknown(ident.1.clone())
            );
            None
        }
    }

    fn apply_to_check(&mut self, opcodes: &[Stat], node: &ApplyTo) -> Option<Stat> {
        let mut from_expand = vec![];

        for ident in &node.from {
            if !self.symbol_check(opcodes, ident, false) {
                if let Some(mut expand) = self.expand_with_group(opcodes, ident) {
                    from_expand.append(&mut expand);
                }
            } else {
                from_expand.push(ident.clone());
            }
        }

        let mut to_expand = vec![];

        for ident in &node.to {
            if !self.symbol_check(opcodes, ident, false) {
                if let Some(mut expand) = self.expand_with_group(opcodes, ident) {
                    to_expand.append(&mut expand);
                }
            } else {
                to_expand.push(ident.clone());
            }
        }

        Some(Stat::ApplyTo(Box::new(ApplyTo {
            from: from_expand,
            to: to_expand,
            span: node.span,
            comments: node.comments.clone(),
            properties: node.properties.clone(),
        })))
    }

    fn children_of_check(&mut self, opcodes: &[Stat], node: &ChildrenOf) -> Option<Stat> {
        let mut from_expand = vec![];

        for ident in &node.from {
            if !self.symbol_check(opcodes, ident, false) {
                if let Some(mut expand) = self.expand_with_group(opcodes, ident) {
                    from_expand.append(&mut expand);
                }
            } else {
                from_expand.push(ident.clone());
            }
        }

        let mut to_expand = vec![];

        for ident in &node.to {
            if !self.symbol_check(opcodes, ident, false) {
                if let Some(mut expand) = self.expand_with_group(opcodes, ident) {
                    to_expand.append(&mut expand);
                }
            } else {
                to_expand.push(ident.clone());
            }
        }

        Some(Stat::ChildrenOf(Box::new(ChildrenOf {
            from: from_expand,
            to: to_expand,
            span: node.span,
            comments: node.comments.clone(),
            properties: node.properties.clone(),
        })))
    }
}

/// Process semantic analyze on `opcodes` slice.
pub fn semantic_analyze(opcodes: &mut [Stat]) -> bool {
    SemanticAnalyzer::default().analyze(opcodes)
}
