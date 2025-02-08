// immutable field
// missing field
// unhandled case
// abs escape
// general mismatch

use std::collections::HashSet;
use std::u32;

use crate::ast;
use crate::ast::InstantiateSourceKind;
use crate::ast::StringId;
use crate::core::*;
use crate::spans::Span;
use crate::spans::SpannedError;

#[derive(Debug, Clone, Copy)]
pub enum HoleSrc {
    /// An explicit _ in a type annotation in the source code.
    Explicit(Span),
    /// Insert :_ after span (pattern, return type, or mut field with missing optional annotation)
    OptAscribe(Span),
    /// span -> span before name after (used for polymorphic instantiation parameters)
    Instantiation((Span, InstantiateSourceKind), StringId),
    /// Wrap the given expr in an explicit type annotation
    CheckedExpr(Span),
    BareFuncArg(Span), // Same as CheckedExpr but with higher priority
}
impl HoleSrc {
    fn priority(&self) -> usize {
        use HoleSrc::*;
        match self {
            Explicit(..) => 100,
            OptAscribe(..) => 81,
            BareFuncArg(..) => 81,
            Instantiation(..) => 64,
            CheckedExpr(..) => 0,
        }
    }
}

pub struct PartialTypeError(SpannedError, ScopeLvl, Vec<Span>);
impl PartialTypeError {
    fn new() -> Self {
        Self(SpannedError::new(), ScopeLvl(u32::MAX), Vec::new())
    }

    fn push(&mut self, msg: String, span: Span) {
        self.0.push_str(msg);
        self.0.push_span(span);
        // Separately track added spans so we can check whether a root matches an already reported span
        self.2.push(span);
    }

    pub fn into<T>(self) -> Result<T, SpannedError> {
        Err(self.0)
    }

    pub fn add_hole_int(&mut self, core: &TypeCheckerCore, strings: &mut lasso::Rodeo, pair: (Value, Use)) {
        // First follow the FlowReasons backwards to get a list of holes (inference variables) and
        // roots involved in the detected type contradiction.
        let mut seen = HashSet::new();
        let mut holes = Vec::new();
        let mut roots = Vec::new();
        backtrack_hole_list_sub(core, &mut seen, &mut holes, &mut roots, pair);

        // For type escape errors, only consider holes in outer scopes
        holes.retain(|v| v.scopelvl <= self.1);
        // println!("{:?} found {} holes {:?}", pair, holes.len(), holes);

        let n = holes.len();
        let best = holes
            .into_iter()
            .enumerate()
            .max_by_key(|&(i, v)| v.src.priority() + i * (n - i));

        if let Some(hole) = best {
            self.0.push_str(
                "Hint: To narrow down the cause of the type mismatch, consider adding an explicit type annotation here:",
            );

            use HoleSrc::*;
            match hole.1.src {
                Explicit(span) => self.0.push_span(span),
                OptAscribe(span) => self.0.push_insert("", span, ": _"),
                Instantiation((span, kind), name) => {
                    let name = strings.resolve(&name);

                    use InstantiateSourceKind::*;
                    let s = match kind {
                        ImplicitCall => format!("[{}=_]", name),
                        ImplicitRecord => format!(" type {}=_; ", name),
                        ExplicitParams(is_empty) => format!("{}{}=_", if is_empty { "" } else { "; " }, name),
                    };

                    self.0.push_insert("", span, s);
                }
                CheckedExpr(span) | BareFuncArg(span) => self.0.push_insert("(", span, ": _)"),
            }
        } else {
            // If there were no type inference variables we could hint for, try flow roots instead
            // println!("roots {:?}", roots);
            // println!("spans {:?}", self.2);

            for span in roots {
                if !self.2.contains(&span) {
                    self.0
                        .push_str("Note: Type mismatch was detected starting from this expression:");
                    self.0.push_span(span);
                    break;
                }
            }
        }
    }
}

enum TMsg {
    BeA(String),
    HaveTy(String, Option<Span>),
}
impl TMsg {
    fn print(&self, show_ctors: bool) -> String {
        match self {
            TMsg::BeA(s) => format!("be a {}", s),
            TMsg::HaveTy(s, c) => {
                if show_ctors && c.is_none() {
                    format!("have builtin type {}", s)
                } else {
                    format!("have type {}", s)
                }
            }
        }
    }
}

fn be_a(s: &str) -> TMsg {
    TMsg::BeA(s.to_owned())
}

pub fn type_mismatch_err(
    strings: &mut lasso::Rodeo,
    type_ctors: &[TypeCtor],
    lhs: &VTypeNode,
    rhs: &UTypeNode,
) -> PartialTypeError {
    use TMsg::*;
    use UTypeHead::*;
    use VTypeHead::*;

    let found = match lhs.0 {
        VUnion(_) => unreachable!(),
        VInstantiateExist { .. } => unreachable!(),
        VPolyHead(..) => unreachable!(),
        VTop => HaveTy("any".to_owned(), None),
        VFunc { .. } => be_a("function"),
        VObj { .. } => be_a("record"),
        VCase { .. } => be_a("variant"),
        VAbstract { ty, .. } => {
            let tycon = &type_ctors[ty.0];
            let name = strings.resolve(&tycon.name);
            HaveTy(name.to_owned(), tycon.span)
        }
        // VAbstract { ty, .. } => &type_ctors[ty.0].debug,
        VTypeVar(tv) => BeA(format!("type parameter {}", strings.resolve(&tv.name))),
        VDisjointIntersect(..) => be_a("intersection"),
    };

    let expected = match rhs.0 {
        UIntersection(_) => unreachable!(),
        UInstantiateUni { .. } => unreachable!(),
        UPolyHead(..) => unreachable!(),
        UBot => HaveTy("never".to_owned(), None),
        UFunc { .. } => be_a("function"),
        UObj { .. } => be_a("record"),
        UCase { .. } => be_a("variant"),
        UAbstract { ty, .. } => {
            let tycon = &type_ctors[ty.0];
            let name = strings.resolve(&tycon.name);
            HaveTy(name.to_owned(), tycon.span)
        }
        // VAbstract { ty, .. } => &type_ctors[ty.0].debug,
        UTypeVar(tv) => BeA(format!("type parameter {}", strings.resolve(&tv.name))),
        UDisjointUnion(..) => be_a("union"),
    };

    let show_ctors = match (&found, &expected) {
        (HaveTy(s, _), HaveTy(s2, _)) if s == s2 => true,
        _ => false,
    };

    let mut parts = PartialTypeError::new();
    parts.push(
        format!("TypeError: Value is required to {} here:", expected.print(show_ctors)),
        rhs.1,
    );
    match expected {
        HaveTy(s, Some(span)) if show_ctors => {
            parts.push(format!("Where {} is the abstract type defined here:", s), span);
        }
        _ => {}
    }

    parts.push(
        format!("However, that value may {} originating here:", found.print(show_ctors)),
        lhs.1,
    );
    match found {
        HaveTy(s, Some(span)) if show_ctors => {
            parts.push(format!("Where {} is the abstract type defined here:", s), span);
        }
        _ => {}
    }

    parts
}

pub fn unhandled_variant_err(lhs: &VTypeNode, rhs: &UTypeNode, name: &str) -> PartialTypeError {
    let mut parts = PartialTypeError::new();
    parts.push(
        format!("TypeError: Unhandled variant {}\nNote: Value originates here:", name),
        lhs.1,
    );
    parts.push(format!("But it is not handled here:"), rhs.1);
    parts
}

pub fn missing_field_err(lhs_span: Span, rhs_span: Span, name: &str) -> PartialTypeError {
    let mut parts = PartialTypeError::new();
    parts.push(
        format!("TypeError: Missing field {}\nNote: Field {} is accessed here:", name, name),
        rhs_span,
    );
    parts.push(format!("But the record is defined without that field here:"), lhs_span);
    parts
}

pub fn immutable_field_err(lhs_span: Span, rhs_span: Span, name: &str) -> PartialTypeError {
    let mut parts = PartialTypeError::new();
    parts.push(
        format!(
            "TypeError: Can't set immutable field {}.\nNote: Field is required to be mutable here:",
            name
        ),
        rhs_span,
    );
    parts.push(format!("But the record is defined with that field immutable here:"), lhs_span);
    parts
}

pub fn type_escape_error(
    strings: &mut lasso::Rodeo,
    ty_ctor: &TypeCtor,
    lhs: &VTypeNode,
    rhs: &UTypeNode,
    scopelvl: ScopeLvl,
) -> PartialTypeError {
    let mut parts = PartialTypeError::new();
    parts.1 = scopelvl;

    parts.push(
        format!(
            "TypeError: Type {} defined here escapes its containing function or loop",
            strings.resolve(&ty_ctor.name),
        ),
        ty_ctor.span.unwrap(),
    );
    parts.push(format!("Note: A value of this type originates here:"), lhs.1);
    parts.push(format!("and is consumed here after escaping the defining scope:"), rhs.1);
    parts
}

pub fn poisoned_poly_err(span: Span) -> PartialTypeError {
    let mut parts = PartialTypeError::new();
    parts
        .0
        .push_str("TypeError: Repeated instantiation of nested polymorphic type requires intervening type annotation:");
    parts.0.push_insert("(", span, ": <type here>)");
    parts
}

fn backtrack_hole_list_sub(
    core: &TypeCheckerCore,
    seen: &mut HashSet<(Value, Use)>,
    holes_list: &mut Vec<InferenceVarData>,
    roots_list: &mut Vec<Span>,
    mut pair: (Value, Use),
) {
    while !seen.contains(&pair) {
        // println!("checking {} {}", pair.0.0.0, pair.1.0.0);
        seen.insert(pair);
        let reason = core.r.get_edge(pair.0.0, pair.1.0).unwrap().reason;
        // println!("reason {:?}", reason);
        match reason {
            FlowReason::Root(span) => {
                roots_list.push(span);
                break;
            }
            FlowReason::Transitivity(h) => {
                backtrack_hole_list_sub(core, seen, holes_list, roots_list, (pair.0, Use(h)));

                match core.r.get(h) {
                    Some(&TypeNode::Var(data)) => holes_list.push(data),
                    _ => unreachable!(),
                }

                pair = (Value(h), pair.1);
            }
            FlowReason::Check(v, u) => pair = (v, u),
        }
    }
}
