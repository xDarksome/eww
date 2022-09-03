use std::collections::HashMap;

use simplexpr::{dynval::DynVal, SimplExpr};

use crate::{
    enum_parse,
    error::{DiagError, DiagResult},
    parser::{
        ast::Ast,
        ast_iterator::AstIterator,
        from_ast::{FromAst, FromAstElementContent},
    },
    value::Coords,
};

use super::{widget_use::WidgetUse, window_definition::EnumParseError};
use eww_shared_util::{AttrName, Span, VarName};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct WindowKeymap {
    pub binds: Vec<Bind>,
}

impl FromAstElementContent for WindowKeymap {
    const ELEMENT_NAME: &'static str = "keymap";

    fn from_tail<I: Iterator<Item = Ast>>(span: Span, mut iter: AstIterator<I>) -> DiagResult<Self> {
        let mut attrs = iter.expect_key_values()?;
        let inhibit = attrs.primitive_optional::<bool, _>("inhibit")?;

        let mut binds = Vec::new();
        for ast in iter {
            let mut bind = Bind::from_ast(ast)?;

            if bind.inhibit.is_none() {
                bind.inhibit = inhibit;
            }

            binds.push(bind);
        }

        Ok(Self { binds })
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct Bind {
    pub inhibit: Option<bool>,
    pub combination: String,
    pub combination_span: Span,
    pub cmd: String,
}

impl FromAstElementContent for Bind {
    const ELEMENT_NAME: &'static str = "bind";

    fn from_tail<I: Iterator<Item = Ast>>(span: Span, mut iter: AstIterator<I>) -> DiagResult<Self> {
        let mut attrs = iter.expect_key_values()?;
        let inhibit = attrs.primitive_optional("inhibit")?;

        let (combination_span, combination) = iter.expect_literal()?;
        let (_, cmd) = iter.expect_literal()?;

        iter.expect_done()?;

        Ok(Self { inhibit, combination: combination.to_string(), combination_span, cmd: cmd.to_string() })
    }
}
