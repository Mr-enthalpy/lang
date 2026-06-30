//! v0.4 Normalized AST prototype.
//!
//! Value-side `NormExpr` and pattern-side `NormPattern` remain distinct. Raw
//! expression-shaped syntax is normalized as pattern material only when the
//! surrounding syntactic context is pattern, annotation, or extraction.
//! This prototype records that boundary; explicit bridge syntax/lowering is
//! future work unless it is already present in Raw AST.

use crate::{
    AliasBinderAst, AnnotationTermAst, AtomAst, AtomKind, BinderDeclAst, BinderNameAst,
    BindingAnnotationAst, BindingPatternAst, BindingSlotAst, BodyBlockAst, CanonicalNameRole,
    CanonicalProductElementAst, CanonicalSkeletonAst, ClosureAst, ClosureBodyAst, DeduceListAst,
    EntityRefAst, ErrorAst, ExprAst, ExprKind, FnHeadPrefixAst, FormAst, HeadClauseAst,
    LetAliasAst, LetAst, NavComponentAst, OperatorExprAst, OperatorExprKind, OperatorFixity,
    OperatorNameAst, ParamClauseAst, PipeExprAst, ProductElementAst, ProductExprAst,
    ProductExtractAst, ProductExtractElementAst, ProgramAst, ReturnClauseAst, SegmentAst,
    SegmentElementAst, SelectorAst, Span, WithClauseAst, WithClauseKind,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NormProgram {
    pub forms: Vec<NormForm>,
    pub origin: NormOrigin,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NormForm {
    Let(NormDecl),
    Alias(NormDecl),
    Expr(NormExpr),
    TailValue(NormExpr),
    ReturnEvent(NormReturnEvent),
    Error(NormError),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NormReturnEvent {
    pub value: NormExpr,
    pub target: NormReturnTargetSyntax,
    pub origin: NormOrigin,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NormReturnTargetSyntax {
    ImplicitNearest,
    Explicit(NormExpr),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NormDecl {
    Let {
        slot: NormBindingSlot,
        origin: NormOrigin,
    },
    Alias {
        policy: Option<Box<NormExpr>>,
        binder: NormAliasBinder,
        target: NormEntityRef,
        origin: NormOrigin,
    },
    Error(NormError),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NormExpr {
    Call {
        source: NormProduct,
        target: Box<NormExpr>,
        origin: NormOrigin,
    },
    Product(NormProduct),
    Name {
        text: String,
        origin: NormOrigin,
    },
    Literal {
        kind: NormLiteralKind,
        text: String,
        origin: NormOrigin,
    },
    Nav {
        components: Vec<NormNavComponent>,
        origin: NormOrigin,
    },
    Closure(NormClosure),
    OperatorTarget {
        spelling: String,
        fixity: NormOperatorFixity,
        arity: usize,
        origin: NormOrigin,
    },
    Error(NormError),
    Unsupported {
        raw_kind_summary: String,
        origin: NormOrigin,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NormProduct {
    pub elements: Vec<NormProductElem>,
    pub origin: NormOrigin,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NormProductElem {
    Expr(NormExpr),
    Unit { origin: NormOrigin },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NormPattern {
    Binder {
        name: String,
        origin: NormOrigin,
    },
    OperatorBinder {
        spelling: String,
        origin: NormOrigin,
    },
    Product {
        elements: Vec<NormPatternElem>,
        origin: NormOrigin,
    },
    Unit {
        origin: NormOrigin,
    },
    HoleRef {
        name: String,
        origin: NormOrigin,
    },
    Name {
        name: String,
        origin: NormOrigin,
    },
    Literal {
        text: String,
        origin: NormOrigin,
    },
    Nav {
        components: Vec<NormNavComponent>,
        origin: NormOrigin,
    },
    Sequence {
        elements: Vec<NormPattern>,
        origin: NormOrigin,
    },
    Skeleton {
        skeleton: NormSkeleton,
        origin: NormOrigin,
    },
    BindingSlot {
        slot: Box<NormBindingSlot>,
        origin: NormOrigin,
    },
    Error(NormError),
    Unsupported {
        raw_kind_summary: String,
        origin: NormOrigin,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NormPatternElem {
    Pattern(NormPattern),
    BindingSlot(NormBindingSlot),
    Unit { origin: NormOrigin },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NormAnnotation {
    pub pattern: NormPattern,
    pub origin: NormOrigin,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NormBindingSlot {
    pub policy: Option<Box<NormExpr>>,
    pub has_let: bool,
    pub deduce: Vec<NormHoleDecl>,
    pub value_pattern: NormPattern,
    pub annotation: Option<NormAnnotation>,
    pub with_clause: Option<NormWithClause>,
    pub initializer: Option<Box<NormExpr>>,
    pub origin: NormOrigin,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NormHoleDecl {
    pub name: String,
    pub annotation: Option<NormAnnotation>,
    pub origin: NormOrigin,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NormWithClause {
    pub names: Vec<String>,
    pub explicit_empty: bool,
    pub error: Option<NormError>,
    pub origin: NormOrigin,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NormClosure {
    pub kind: NormClosureKind,
    pub head: Option<NormClosureHead>,
    pub body: NormClosureBody,
    pub origin: NormOrigin,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NormClosureBody {
    Block(NormProgram),
    Delete(NormDeleteBody),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NormDeleteBody {
    pub message: Box<NormExpr>,
    pub origin: NormOrigin,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NormClosureKind {
    InPlace,
    Explicit,
    Generated { rule: NormRule },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NormClosureHead {
    pub deduce: Vec<NormHoleDecl>,
    pub captures: Vec<NormExpr>,
    pub params: Vec<NormPatternElem>,
    pub fn_item_trait: Option<NormAnnotation>,
    pub returns: Option<NormBindingSlot>,
    pub clauses: Vec<NormHeadClause>,
    pub origin: NormOrigin,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NormHeadClause {
    Require { expr: NormExpr, origin: NormOrigin },
    Pre { expr: NormExpr, origin: NormOrigin },
    Post { expr: NormExpr, origin: NormOrigin },
    LifetimePre { expr: NormExpr, origin: NormOrigin },
    LifetimePost { expr: NormExpr, origin: NormOrigin },
    Error(NormError),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NormEntityRef {
    pub components: Vec<NormNavComponent>,
    pub origin: NormOrigin,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NormAliasBinder {
    Name {
        name: String,
        origin: NormOrigin,
    },
    Operator {
        spelling: String,
        origin: NormOrigin,
    },
    Error(NormError),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NormNavComponent {
    Name {
        name: String,
        origin: NormOrigin,
    },
    Operator {
        spelling: String,
        origin: NormOrigin,
    },
    Group {
        expr: Box<NormExpr>,
        origin: NormOrigin,
    },
    Error(NormError),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NormSkeleton {
    Segment {
        elements: Vec<NormSkeleton>,
        origin: NormOrigin,
    },
    Product {
        elements: Vec<NormSkeletonElem>,
        origin: NormOrigin,
    },
    Wildcard {
        origin: NormOrigin,
    },
    Name {
        name: String,
        role: NormCanonicalNameRole,
        origin: NormOrigin,
    },
    Nav {
        names: Vec<String>,
        origin: NormOrigin,
    },
    Literal {
        text: String,
        origin: NormOrigin,
    },
    Error(NormError),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NormSkeletonElem {
    Skeleton(NormSkeleton),
    Unit { origin: NormOrigin },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NormCanonicalNameRole {
    Hole,
    NodeName,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NormLiteralKind {
    Int,
    Float,
    String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NormOperatorFixity {
    Prefix,
    Postfix,
    Binary,
    BracketCall,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NormError {
    pub message: String,
    pub origin: NormOrigin,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NormOrigin {
    Source(Span),
    Generated {
        rule: NormRule,
        span: Span,
    },
    Derived {
        rule: NormRule,
        span: Span,
        summary: String,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NormRule {
    ProductLift,
    ProductMerge,
    PipeFallback,
    SecondLegalityRepair,
    OperatorLowering,
    PrefixNegativeLowering,
    MemberLowering,
    DoubleDotLowering,
    BracketCallLowering,
    BranchNameExpansion,
    AliasPreserve,
    ClosureNormalize,
    PatternNormalize,
    Unsupported,
}

pub fn normalize_program(raw: &ProgramAst) -> NormProgram {
    NormProgram {
        forms: raw.forms.iter().map(normalize_form).collect(),
        origin: NormOrigin::Source(raw.span),
    }
}

pub fn dump_norm_program(program: &NormProgram) -> String {
    let mut output = String::new();
    line(
        &mut output,
        0,
        &format!("NormProgram {}", origin_inline(&program.origin)),
    );
    line(&mut output, 1, "forms:");
    for form in &program.forms {
        dump_norm_form(&mut output, form, 2);
    }
    output
}

fn normalize_form(form: &FormAst) -> NormForm {
    match form {
        FormAst::Let(let_ast) => NormForm::Let(normalize_let_decl(let_ast)),
        FormAst::AliasLet(alias) => NormForm::Alias(normalize_alias_decl(alias)),
        FormAst::Expr(expr) => NormForm::Expr(normalize_expr(expr)),
        FormAst::ReturnEvent(return_ev) => {
            let value = normalize_expr(&return_ev.value);
            let target = match &return_ev.target {
                crate::ReturnTargetAst::ImplicitNearest { .. } => {
                    NormReturnTargetSyntax::ImplicitNearest
                }
                crate::ReturnTargetAst::Explicit { target, .. } => {
                    NormReturnTargetSyntax::Explicit(normalize_expr(target))
                }
            };
            NormForm::ReturnEvent(NormReturnEvent {
                value,
                target,
                origin: NormOrigin::Source(return_ev.span),
            })
        }
        FormAst::Error(error) => NormForm::Error(normalize_error(error)),
    }
}

fn normalize_let_decl(let_ast: &LetAst) -> NormDecl {
    NormDecl::Let {
        slot: normalize_binding_slot(&let_ast.slot, &[]),
        origin: NormOrigin::Source(let_ast.span),
    }
}

fn normalize_alias_decl(alias: &LetAliasAst) -> NormDecl {
    let policy = alias
        .policy
        .as_ref()
        .map(|policy| Box::new(normalize_expr(policy)));
    let binder = match &alias.binder {
        AliasBinderAst::Name(name) => NormAliasBinder::Name {
            name: name.text.clone(),
            origin: NormOrigin::Source(name.span),
        },
        AliasBinderAst::Operator(operator) => NormAliasBinder::Operator {
            spelling: operator.spelling.clone(),
            origin: NormOrigin::Source(operator.span),
        },
        AliasBinderAst::Error(error) => NormAliasBinder::Error(normalize_error(error)),
    };

    NormDecl::Alias {
        policy,
        binder,
        target: normalize_entity_ref(&alias.target),
        origin: NormOrigin::Generated {
            rule: NormRule::AliasPreserve,
            span: alias.span,
        },
    }
}

fn normalize_expr(expr: &ExprAst) -> NormExpr {
    // Value-side entry point. This must never reinterpret expression material as
    // extraction/pattern material; pattern contexts use the dedicated
    // normalize_*_as_pattern path below.
    match &expr.kind {
        ExprKind::Pipe(pipe) => normalize_pipe(pipe),
        ExprKind::Product(product) => NormExpr::Product(normalize_product_expr(product, true)),
        ExprKind::Error(error) => NormExpr::Error(normalize_error(error)),
    }
}

fn normalize_pipe(pipe: &PipeExprAst) -> NormExpr {
    let mut segments = pipe.segments.iter();
    let Some(first) = segments.next() else {
        return NormExpr::Error(NormError {
            message: "empty pipe expression".to_string(),
            origin: NormOrigin::Source(pipe.span),
        });
    };

    let mut current = normalize_segment_without_incoming(first);
    for segment in segments {
        current = normalize_segment_with_incoming(current, segment);
    }
    current
}

fn normalize_segment_without_incoming(segment: &SegmentAst) -> NormExpr {
    let items = normalize_segment_items(segment);
    lower_item_chain(None, &items, segment.span)
}

fn normalize_segment_with_incoming(incoming: NormExpr, segment: &SegmentAst) -> NormExpr {
    let items = normalize_segment_items(segment);
    if items.is_empty() {
        return NormExpr::Error(NormError {
            message: "empty incoming pipe segment".to_string(),
            origin: NormOrigin::Source(segment.span),
        });
    }

    let product_index = (1..items.len()).find(|index| items[*index].source_product().is_some());

    if let Some(product_index) = product_index {
        let target = lower_item_chain(None, &items[..product_index], segment.span);
        let incoming_product = source_product_from_expr(incoming, segment.span);
        let continuation_product = items[product_index]
            .source_product()
            .expect("product_index selected a source product");
        let merged = merge_products(incoming_product, continuation_product, segment.span);
        let mut current = make_call(
            merged,
            target,
            NormOrigin::Derived {
                rule: NormRule::ProductMerge,
                span: segment.span,
                summary: "source-product continuation".to_string(),
            },
        );
        current = lower_item_chain(Some(current), &items[product_index + 1..], segment.span);
        current
    } else {
        let target = lower_item_chain(None, &items, segment.span);
        let source = source_product_from_expr(incoming, segment.span);
        make_call(
            source,
            target,
            NormOrigin::Derived {
                rule: NormRule::PipeFallback,
                span: segment.span,
                summary: "no following source product".to_string(),
            },
        )
    }
}

#[derive(Clone, Debug)]
enum SegmentItem {
    Expr {
        expr: NormExpr,
        source_product: Option<NormProduct>,
    },
    Product(NormProduct),
}

impl SegmentItem {
    fn expr(&self) -> Option<NormExpr> {
        match self {
            SegmentItem::Expr { expr, .. } => Some(expr.clone()),
            SegmentItem::Product(_) => None,
        }
    }

    fn source_product(&self) -> Option<NormProduct> {
        match self {
            SegmentItem::Expr { source_product, .. } => source_product.clone(),
            SegmentItem::Product(product) => Some(product.clone()),
        }
    }
}

fn normalize_segment_items(segment: &SegmentAst) -> Vec<SegmentItem> {
    segment
        .elements
        .iter()
        .map(|element| match element {
            SegmentElementAst::OperatorExpr(expr) => normalize_operator_expr_item(expr),
            SegmentElementAst::Product(product) => {
                SegmentItem::Product(normalize_product_expr(product, true))
            }
        })
        .collect()
}

fn normalize_operator_expr_item(expr: &OperatorExprAst) -> SegmentItem {
    match &expr.kind {
        OperatorExprKind::Product(product) => {
            SegmentItem::Product(normalize_product_expr(product, true))
        }
        OperatorExprKind::Atom(atom) => match &atom.kind {
            AtomKind::Group(inner) => {
                let lowered = normalize_expr(inner);
                let source_product = NormProduct {
                    elements: vec![NormProductElem::Expr(lowered.clone())],
                    origin: NormOrigin::Generated {
                        rule: NormRule::ProductLift,
                        span: atom.span,
                    },
                };
                SegmentItem::Expr {
                    expr: lowered,
                    source_product: Some(source_product),
                }
            }
            _ => SegmentItem::Expr {
                expr: normalize_operator_expr(expr),
                source_product: None,
            },
        },
        _ => SegmentItem::Expr {
            expr: normalize_operator_expr(expr),
            source_product: None,
        },
    }
}

fn lower_item_chain(
    initial: Option<NormExpr>,
    items: &[SegmentItem],
    fallback_span: Span,
) -> NormExpr {
    let mut current = initial;
    let mut index = 0;

    while index < items.len() {
        let expr = items[index].expr();
        let product = items[index].source_product();
        let next_expr = items.get(index + 1).and_then(SegmentItem::expr);
        let should_use_product =
            product.is_some() && (expr.is_none() || (current.is_some() && next_expr.is_some()));

        if should_use_product {
            let product = product.expect("should_use_product requires a product");
            if let Some(target) = next_expr {
                let repaired = make_call(
                    product,
                    target,
                    NormOrigin::Derived {
                        rule: NormRule::SecondLegalityRepair,
                        span: fallback_span,
                        summary: "repaired product-before-target".to_string(),
                    },
                );
                current = Some(match current {
                    Some(previous) => make_call(
                        source_product_from_expr(previous, fallback_span),
                        repaired,
                        NormOrigin::Derived {
                            rule: NormRule::SecondLegalityRepair,
                            span: fallback_span,
                            summary: "repaired product target in expression chain".to_string(),
                        },
                    ),
                    None => repaired,
                });
                index += 2;
                continue;
            }

            current = Some(match current {
                Some(previous) => make_call(
                    source_product_from_expr(previous, fallback_span),
                    NormExpr::Unsupported {
                        raw_kind_summary: "dangling product cannot be a call target".to_string(),
                        origin: NormOrigin::Generated {
                            rule: NormRule::Unsupported,
                            span: fallback_span,
                        },
                    },
                    NormOrigin::Generated {
                        rule: NormRule::Unsupported,
                        span: fallback_span,
                    },
                ),
                None => NormExpr::Product(product),
            });
            index += 1;
            continue;
        }

        if let Some(expr) = items[index].expr() {
            current = Some(match current {
                Some(previous) => make_call(
                    source_product_from_expr(previous, fallback_span),
                    expr,
                    NormOrigin::Derived {
                        rule: NormRule::PipeFallback,
                        span: fallback_span,
                        summary: "ordinary expression-chain growth".to_string(),
                    },
                ),
                None => expr,
            });
            index += 1;
            continue;
        }

        let product = items[index]
            .source_product()
            .expect("non-expression item must have a source product");
        if index + 1 < items.len() {
            if let Some(target) = items[index + 1].expr() {
                let repaired = make_call(
                    product,
                    target,
                    NormOrigin::Derived {
                        rule: NormRule::SecondLegalityRepair,
                        span: fallback_span,
                        summary: "repaired product-before-target".to_string(),
                    },
                );
                current = Some(match current {
                    Some(previous) => make_call(
                        source_product_from_expr(previous, fallback_span),
                        repaired,
                        NormOrigin::Derived {
                            rule: NormRule::SecondLegalityRepair,
                            span: fallback_span,
                            summary: "repaired product target in expression chain".to_string(),
                        },
                    ),
                    None => repaired,
                });
                index += 2;
                continue;
            }
        }

        current = Some(match current {
            Some(previous) => make_call(
                source_product_from_expr(previous, fallback_span),
                NormExpr::Unsupported {
                    raw_kind_summary: "dangling product cannot be a call target".to_string(),
                    origin: NormOrigin::Generated {
                        rule: NormRule::Unsupported,
                        span: fallback_span,
                    },
                },
                NormOrigin::Generated {
                    rule: NormRule::Unsupported,
                    span: fallback_span,
                },
            ),
            None => NormExpr::Product(product),
        });
        index += 1;
    }

    current.unwrap_or_else(|| {
        NormExpr::Error(NormError {
            message: "empty expression segment".to_string(),
            origin: NormOrigin::Source(fallback_span),
        })
    })
}

fn normalize_operator_expr(expr: &OperatorExprAst) -> NormExpr {
    match &expr.kind {
        OperatorExprKind::Atom(atom) => normalize_atom(atom),
        OperatorExprKind::Product(product) => {
            NormExpr::Product(normalize_product_expr(product, true))
        }
        OperatorExprKind::OperatorSugar {
            operator,
            fixity,
            args,
            span,
        } => normalize_operator_sugar(operator, *fixity, args, *span),
        OperatorExprKind::NavPath { components, span } => NormExpr::Nav {
            components: components.iter().map(normalize_nav_component).collect(),
            origin: NormOrigin::Source(*span),
        },
        OperatorExprKind::MemberSugar {
            object,
            selector,
            span,
        } => normalize_member_sugar(normalize_operator_expr(object), selector, *span),
        OperatorExprKind::DoubleDotSugar {
            object,
            selector,
            args,
            span,
        } => normalize_double_dot_sugar(normalize_operator_expr(object), selector, args, *span),
        OperatorExprKind::BracketCallSugar {
            object,
            operator,
            args,
            span,
        } => normalize_bracket_call_sugar(normalize_operator_expr(object), operator, args, *span),
        OperatorExprKind::Error(error) => NormExpr::Error(normalize_error(error)),
    }
}

fn normalize_atom(atom: &AtomAst) -> NormExpr {
    match &atom.kind {
        AtomKind::Name(name) => NormExpr::Name {
            text: name.text.clone(),
            origin: NormOrigin::Source(name.span),
        },
        AtomKind::IntLiteral(text) => NormExpr::Literal {
            kind: NormLiteralKind::Int,
            text: text.clone(),
            origin: NormOrigin::Source(atom.span),
        },
        AtomKind::FloatLiteral(text) => NormExpr::Literal {
            kind: NormLiteralKind::Float,
            text: text.clone(),
            origin: NormOrigin::Source(atom.span),
        },
        AtomKind::StringLiteral(text) => NormExpr::Literal {
            kind: NormLiteralKind::String,
            text: text.clone(),
            origin: NormOrigin::Source(atom.span),
        },
        AtomKind::Group(expr) => normalize_expr(expr),
        AtomKind::NavPath { components } => NormExpr::Nav {
            components: components.iter().map(normalize_nav_component).collect(),
            origin: NormOrigin::Source(atom.span),
        },
        AtomKind::MemberSugar { object, selector } => {
            normalize_member_sugar(normalize_atom(object), selector, atom.span)
        }
        AtomKind::DoubleDotSugar {
            object,
            selector,
            args,
        } => normalize_double_dot_sugar(normalize_atom(object), selector, args, atom.span),
        AtomKind::BracketCallSugar {
            object,
            operator,
            args,
        } => normalize_bracket_call_sugar(normalize_atom(object), operator, args, atom.span),
        AtomKind::Closure(closure) => NormExpr::Closure(normalize_closure(closure)),
        AtomKind::Error(error) => NormExpr::Error(normalize_error(error)),
    }
}

fn normalize_operator_sugar(
    operator: &OperatorNameAst,
    fixity: OperatorFixity,
    args: &[OperatorExprAst],
    span: Span,
) -> NormExpr {
    match fixity {
        OperatorFixity::Prefix if operator.spelling == "-" && args.len() == 1 => {
            let operand = normalize_operator_expr(&args[0]);
            let closure = generated_prefix_negative_closure(span);
            make_call(
                source_product_from_expr(operand, span),
                NormExpr::Closure(closure),
                NormOrigin::Generated {
                    rule: NormRule::PrefixNegativeLowering,
                    span,
                },
            )
        }
        OperatorFixity::Postfix if args.len() == 1 => {
            let source = source_product_from_expr(normalize_operator_expr(&args[0]), span);
            make_call(
                source,
                operator_target(operator, NormOperatorFixity::Postfix, 1),
                NormOrigin::Generated {
                    rule: NormRule::OperatorLowering,
                    span,
                },
            )
        }
        OperatorFixity::Binary if args.len() == 2 => {
            let source = NormProduct {
                elements: vec![
                    NormProductElem::Expr(normalize_operator_expr(&args[0])),
                    NormProductElem::Expr(normalize_operator_expr(&args[1])),
                ],
                origin: NormOrigin::Generated {
                    rule: NormRule::OperatorLowering,
                    span,
                },
            };
            make_call(
                source,
                operator_target(operator, NormOperatorFixity::Binary, 2),
                NormOrigin::Generated {
                    rule: NormRule::OperatorLowering,
                    span,
                },
            )
        }
        _ => NormExpr::Unsupported {
            raw_kind_summary: format!(
                "operator sugar fixity={} arity={}",
                raw_fixity_label(fixity),
                args.len()
            ),
            origin: NormOrigin::Generated {
                rule: NormRule::Unsupported,
                span,
            },
        },
    }
}

fn normalize_member_sugar(object: NormExpr, selector: &SelectorAst, span: Span) -> NormExpr {
    let selector_name = selector_name(selector);
    let closure = generated_receiver_closure(
        NormRule::MemberLowering,
        span,
        make_call(
            NormProduct {
                elements: vec![NormProductElem::Expr(generated_name(
                    "val",
                    span,
                    NormRule::MemberLowering,
                ))],
                origin: NormOrigin::Generated {
                    rule: NormRule::MemberLowering,
                    span,
                },
            },
            generated_nav(
                &[selector_name.as_str(), "T"],
                span,
                NormRule::MemberLowering,
            ),
            NormOrigin::Generated {
                rule: NormRule::MemberLowering,
                span,
            },
        ),
    );

    make_call(
        source_product_from_expr(object, span),
        NormExpr::Closure(closure),
        NormOrigin::Generated {
            rule: NormRule::MemberLowering,
            span,
        },
    )
}

fn normalize_double_dot_sugar(
    object: NormExpr,
    selector: &SelectorAst,
    args: &ProductExprAst,
    span: Span,
) -> NormExpr {
    let selector_name = selector_name(selector);
    let mut elements = vec![NormProductElem::Expr(generated_name(
        "val",
        span,
        NormRule::DoubleDotLowering,
    ))];
    elements.extend(normalize_product_elements(args, false));
    let body = make_call(
        NormProduct {
            elements,
            origin: NormOrigin::Generated {
                rule: NormRule::DoubleDotLowering,
                span,
            },
        },
        generated_nav(
            &[selector_name.as_str(), "T"],
            span,
            NormRule::DoubleDotLowering,
        ),
        NormOrigin::Generated {
            rule: NormRule::DoubleDotLowering,
            span,
        },
    );
    let closure = generated_receiver_closure(NormRule::DoubleDotLowering, span, body);

    make_call(
        source_product_from_expr(object, span),
        NormExpr::Closure(closure),
        NormOrigin::Generated {
            rule: NormRule::DoubleDotLowering,
            span,
        },
    )
}

fn normalize_bracket_call_sugar(
    object: NormExpr,
    operator: &OperatorNameAst,
    args: &ProductExprAst,
    span: Span,
) -> NormExpr {
    let mut elements = vec![NormProductElem::Expr(object)];
    elements.extend(normalize_product_elements(args, false));
    let source = NormProduct {
        elements,
        origin: NormOrigin::Generated {
            rule: NormRule::BracketCallLowering,
            span,
        },
    };

    make_call(
        source,
        operator_target(
            operator,
            NormOperatorFixity::BracketCall,
            args.elements.len() + 1,
        ),
        NormOrigin::Generated {
            rule: NormRule::BracketCallLowering,
            span,
        },
    )
}

fn normalize_product_expr(product: &ProductExprAst, empty_is_unit: bool) -> NormProduct {
    NormProduct {
        elements: normalize_product_elements(product, empty_is_unit),
        origin: NormOrigin::Source(product.span),
    }
}

fn normalize_product_elements(
    product: &ProductExprAst,
    empty_is_unit: bool,
) -> Vec<NormProductElem> {
    if product.elements.is_empty() && empty_is_unit {
        return vec![NormProductElem::Unit {
            origin: NormOrigin::Source(product.span),
        }];
    }

    product
        .elements
        .iter()
        .map(|element| match element {
            ProductElementAst::Expr(expr) => NormProductElem::Expr(normalize_expr(expr)),
            ProductElementAst::Unit { span } => NormProductElem::Unit {
                origin: NormOrigin::Source(*span),
            },
        })
        .collect()
}

fn source_product_from_expr(expr: NormExpr, span: Span) -> NormProduct {
    let expr_span = expr_span(&expr).unwrap_or(span);
    match expr {
        NormExpr::Product(product) => product,
        expr => NormProduct {
            elements: vec![NormProductElem::Expr(expr)],
            origin: NormOrigin::Generated {
                rule: NormRule::ProductLift,
                span: expr_span,
            },
        },
    }
}

fn merge_products(left: NormProduct, right: NormProduct, span: Span) -> NormProduct {
    let mut elements = left.elements;
    elements.extend(right.elements);
    NormProduct {
        elements,
        origin: NormOrigin::Derived {
            rule: NormRule::ProductMerge,
            span,
            summary: "merged source product with continuation product".to_string(),
        },
    }
}

fn make_call(source: NormProduct, target: NormExpr, origin: NormOrigin) -> NormExpr {
    NormExpr::Call {
        source,
        target: Box::new(target),
        origin,
    }
}

fn operator_target(
    operator: &OperatorNameAst,
    fixity: NormOperatorFixity,
    arity: usize,
) -> NormExpr {
    NormExpr::OperatorTarget {
        spelling: operator.spelling.clone(),
        fixity,
        arity,
        origin: NormOrigin::Source(operator.span),
    }
}

fn normalize_closure(closure: &ClosureAst) -> NormClosure {
    match closure {
        ClosureAst::InPlace(inner) => NormClosure {
            kind: NormClosureKind::InPlace,
            head: None,
            body: NormClosureBody::Block(normalize_body_block(&inner.body)),
            origin: NormOrigin::Source(inner.span),
        },
        ClosureAst::Explicit(inner) => {
            let body = match &inner.body {
                ClosureBodyAst::Block(block) => NormClosureBody::Block(normalize_body_block(block)),
                ClosureBodyAst::Delete(del) => NormClosureBody::Delete(NormDeleteBody {
                    message: Box::new(normalize_expr(&del.message)),
                    origin: NormOrigin::Source(del.span),
                }),
            };
            NormClosure {
                kind: NormClosureKind::Explicit,
                head: Some(normalize_closure_head(&inner.head)),
                body,
                origin: NormOrigin::Source(inner.span),
            }
        }
    }
}

fn normalize_body_block(body: &BodyBlockAst) -> NormProgram {
    let len = body.forms.len();
    let forms: Vec<NormForm> = body
        .forms
        .iter()
        .enumerate()
        .map(|(i, form)| {
            if i == len - 1 {
                match form {
                    FormAst::Expr(expr) => NormForm::TailValue(normalize_expr(expr)),
                    _ => normalize_form(form),
                }
            } else {
                normalize_form(form)
            }
        })
        .collect();
    NormProgram {
        forms,
        origin: NormOrigin::Source(body.span),
    }
}

fn normalize_closure_head(head: &FnHeadPrefixAst) -> NormClosureHead {
    let deduce = normalize_deduce_list(head.deduce.as_ref(), &[]);
    let hole_names = deduce
        .iter()
        .map(|hole| hole.name.clone())
        .collect::<Vec<_>>();
    let params = head
        .params
        .as_ref()
        .map(|params| normalize_param_clause(params, &hole_names))
        .unwrap_or_default();
    let captures = head
        .captures
        .as_ref()
        .map(|captures| {
            captures
                .items
                .iter()
                .map(|item| normalize_expr(&item.expr))
                .collect()
        })
        .unwrap_or_default();
    let fn_item_trait = head
        .fn_item_trait
        .as_ref()
        .map(|expr| normalize_annotation_expr(expr, &hole_names));
    let returns = head
        .returns
        .as_ref()
        .map(|returns| normalize_return_clause(returns, &hole_names));
    let clauses = head.clauses.iter().map(normalize_head_clause).collect();

    NormClosureHead {
        deduce,
        captures,
        params,
        fn_item_trait,
        returns,
        clauses,
        origin: NormOrigin::Generated {
            rule: NormRule::ClosureNormalize,
            span: head.span,
        },
    }
}

fn normalize_param_clause(params: &ParamClauseAst, hole_names: &[String]) -> Vec<NormPatternElem> {
    params
        .extract
        .elements
        .iter()
        .map(|element| match element {
            ProductExtractElementAst::Slot(slot) => {
                NormPatternElem::BindingSlot(normalize_binding_slot(slot, hole_names))
            }
            ProductExtractElementAst::Unit { span } => NormPatternElem::Unit {
                origin: NormOrigin::Source(*span),
            },
        })
        .collect()
}

fn normalize_return_clause(returns: &ReturnClauseAst, hole_names: &[String]) -> NormBindingSlot {
    normalize_binding_slot(&returns.slot, hole_names)
}

fn normalize_head_clause(clause: &HeadClauseAst) -> NormHeadClause {
    match clause {
        HeadClauseAst::Require { expr, span } => NormHeadClause::Require {
            expr: normalize_expr(expr),
            origin: NormOrigin::Source(*span),
        },
        HeadClauseAst::Pre { expr, span } => NormHeadClause::Pre {
            expr: normalize_expr(expr),
            origin: NormOrigin::Source(*span),
        },
        HeadClauseAst::Post { expr, span } => NormHeadClause::Post {
            expr: normalize_expr(expr),
            origin: NormOrigin::Source(*span),
        },
        HeadClauseAst::LifetimePre { expr, span } => NormHeadClause::LifetimePre {
            expr: normalize_expr(expr),
            origin: NormOrigin::Source(*span),
        },
        HeadClauseAst::LifetimePost { expr, span } => NormHeadClause::LifetimePost {
            expr: normalize_expr(expr),
            origin: NormOrigin::Source(*span),
        },
        HeadClauseAst::Error(error) => NormHeadClause::Error(normalize_error(error)),
    }
}

fn normalize_binding_slot(slot: &BindingSlotAst, inherited_holes: &[String]) -> NormBindingSlot {
    let deduce = normalize_deduce_list(slot.deduce.as_ref(), inherited_holes);
    let mut hole_names = inherited_holes.to_vec();
    hole_names.extend(deduce.iter().map(|hole| hole.name.clone()));

    NormBindingSlot {
        policy: slot
            .policy
            .as_ref()
            .map(|policy| Box::new(normalize_expr(policy))),
        has_let: slot.has_let,
        deduce,
        value_pattern: normalize_binding_pattern(&slot.pattern, &hole_names),
        annotation: slot
            .annotation
            .as_ref()
            .map(|annotation| normalize_binding_annotation(annotation, &hole_names)),
        with_clause: slot.with_clause.as_ref().map(normalize_with_clause),
        initializer: slot
            .initializer
            .as_ref()
            .map(|initializer| Box::new(normalize_expr(initializer))),
        origin: NormOrigin::Generated {
            rule: NormRule::PatternNormalize,
            span: slot.span,
        },
    }
}

fn normalize_deduce_list(
    deduce: Option<&DeduceListAst>,
    inherited_holes: &[String],
) -> Vec<NormHoleDecl> {
    deduce
        .map(|deduce| {
            deduce
                .binders
                .iter()
                .map(|binder| normalize_hole_decl(binder, inherited_holes))
                .collect()
        })
        .unwrap_or_default()
}

fn normalize_hole_decl(binder: &BinderDeclAst, inherited_holes: &[String]) -> NormHoleDecl {
    NormHoleDecl {
        name: binder.name.text.clone(),
        annotation: binder
            .annotation
            .as_ref()
            .map(|annotation| normalize_annotation_term(annotation, inherited_holes)),
        origin: NormOrigin::Generated {
            rule: NormRule::PatternNormalize,
            span: binder.span,
        },
    }
}

fn normalize_binding_pattern(pattern: &BindingPatternAst, holes: &[String]) -> NormPattern {
    // Extraction-side entry point. Binder/name/skeleton material stays in the
    // NormPattern family and is not treated as value-side call target material.
    match pattern {
        BindingPatternAst::Binder(BinderNameAst::Text(name)) => NormPattern::Binder {
            name: name.text.clone(),
            origin: NormOrigin::Source(name.span),
        },
        BindingPatternAst::Binder(BinderNameAst::Operator(operator)) => {
            NormPattern::OperatorBinder {
                spelling: operator.spelling.clone(),
                origin: NormOrigin::Source(operator.span),
            }
        }
        BindingPatternAst::Product(product) => normalize_product_extract_pattern(product, holes),
        BindingPatternAst::Skeleton(skeleton) => NormPattern::Skeleton {
            skeleton: normalize_canonical_skeleton(skeleton),
            origin: NormOrigin::Generated {
                rule: NormRule::PatternNormalize,
                span: skeleton_span(skeleton),
            },
        },
        BindingPatternAst::Error(error) => NormPattern::Error(normalize_error(error)),
    }
}

fn normalize_product_extract_pattern(product: &ProductExtractAst, holes: &[String]) -> NormPattern {
    let elements = product
        .elements
        .iter()
        .map(|element| match element {
            ProductExtractElementAst::Slot(slot) => {
                NormPatternElem::BindingSlot(normalize_binding_slot(slot, holes))
            }
            ProductExtractElementAst::Unit { span } => NormPatternElem::Unit {
                origin: NormOrigin::Source(*span),
            },
        })
        .collect();
    NormPattern::Product {
        elements,
        origin: NormOrigin::Generated {
            rule: NormRule::PatternNormalize,
            span: product.span,
        },
    }
}

fn normalize_binding_annotation(
    annotation: &BindingAnnotationAst,
    holes: &[String],
) -> NormAnnotation {
    // Annotation syntax is classifier/pattern material. It deliberately lowers
    // to NormAnnotation { pattern: NormPattern } rather than NormExpr.
    match annotation {
        BindingAnnotationAst::Expr(expr) => normalize_annotation_expr(expr, holes),
        BindingAnnotationAst::Compound { left, right, span } => {
            let left_pattern = normalize_annotation_term(left, holes).pattern;
            let right_pattern = normalize_annotation_expr(right, holes).pattern;
            NormAnnotation {
                pattern: NormPattern::Sequence {
                    elements: vec![left_pattern, right_pattern],
                    origin: NormOrigin::Generated {
                        rule: NormRule::PatternNormalize,
                        span: *span,
                    },
                },
                origin: NormOrigin::Generated {
                    rule: NormRule::PatternNormalize,
                    span: *span,
                },
            }
        }
        BindingAnnotationAst::Error(error) => NormAnnotation {
            pattern: NormPattern::Error(normalize_error(error)),
            origin: NormOrigin::Source(error.span),
        },
    }
}

fn normalize_annotation_term(term: &AnnotationTermAst, holes: &[String]) -> NormAnnotation {
    match term {
        AnnotationTermAst::Expr(expr) => normalize_annotation_expr(expr, holes),
        AnnotationTermAst::Hole { span } => NormAnnotation {
            pattern: NormPattern::HoleRef {
                name: "_".to_string(),
                origin: NormOrigin::Source(*span),
            },
            origin: NormOrigin::Generated {
                rule: NormRule::PatternNormalize,
                span: *span,
            },
        },
    }
}

fn normalize_annotation_expr(expr: &ExprAst, holes: &[String]) -> NormAnnotation {
    // Bridge from raw expression-shaped parser surface into pattern context.
    // This is not value-to-pattern conversion for runtime values.
    NormAnnotation {
        pattern: normalize_expr_as_pattern(expr, holes),
        origin: NormOrigin::Generated {
            rule: NormRule::PatternNormalize,
            span: expr.span,
        },
    }
}

fn normalize_expr_as_pattern(expr: &ExprAst, holes: &[String]) -> NormPattern {
    // Pattern-side lowering for raw expression-shaped syntax in annotation or
    // extraction contexts. Names become PatternName/HoleRef, not NormExpr::Name.
    match &expr.kind {
        ExprKind::Pipe(pipe) => normalize_pipe_as_pattern(pipe, holes),
        ExprKind::Product(product) => {
            let elements = product
                .elements
                .iter()
                .map(|element| match element {
                    ProductElementAst::Expr(expr) => {
                        NormPatternElem::Pattern(normalize_expr_as_pattern(expr, holes))
                    }
                    ProductElementAst::Unit { span } => NormPatternElem::Unit {
                        origin: NormOrigin::Source(*span),
                    },
                })
                .collect();
            NormPattern::Product {
                elements,
                origin: NormOrigin::Generated {
                    rule: NormRule::PatternNormalize,
                    span: product.span,
                },
            }
        }
        ExprKind::Error(error) => NormPattern::Error(normalize_error(error)),
    }
}

fn normalize_pipe_as_pattern(pipe: &PipeExprAst, holes: &[String]) -> NormPattern {
    // Pipe-shaped raw syntax in pattern context is preserved as pattern
    // sequence material. It does not participate in value-side call lowering.
    let mut elements = Vec::new();
    for segment in &pipe.segments {
        for element in &segment.elements {
            match element {
                SegmentElementAst::OperatorExpr(expr) => {
                    elements.push(normalize_operator_expr_as_pattern(expr, holes));
                }
                SegmentElementAst::Product(product) => {
                    let product_pattern = normalize_expr_as_pattern(
                        &ExprAst {
                            kind: ExprKind::Product(product.clone()),
                            span: product.span,
                        },
                        holes,
                    );
                    elements.push(product_pattern);
                }
            }
        }
    }

    if elements.len() == 1 {
        elements.remove(0)
    } else {
        NormPattern::Sequence {
            elements,
            origin: NormOrigin::Generated {
                rule: NormRule::PatternNormalize,
                span: pipe.span,
            },
        }
    }
}

fn normalize_operator_expr_as_pattern(expr: &OperatorExprAst, holes: &[String]) -> NormPattern {
    // Operator-expression raw syntax in pattern context stays in NormPattern.
    // Unsupported sugar is surfaced explicitly instead of silently becoming a
    // value-side expression.
    match &expr.kind {
        OperatorExprKind::Atom(atom) => normalize_atom_as_pattern(atom, holes),
        OperatorExprKind::Product(product) => normalize_expr_as_pattern(
            &ExprAst {
                kind: ExprKind::Product(product.clone()),
                span: product.span,
            },
            holes,
        ),
        OperatorExprKind::NavPath { components, span } => NormPattern::Nav {
            components: components.iter().map(normalize_nav_component).collect(),
            origin: NormOrigin::Source(*span),
        },
        OperatorExprKind::Error(error) => NormPattern::Error(normalize_error(error)),
        other => NormPattern::Unsupported {
            raw_kind_summary: annotation_operator_pattern_summary(other),
            origin: NormOrigin::Generated {
                rule: NormRule::Unsupported,
                span: expr.span,
            },
        },
    }
}

fn normalize_atom_as_pattern(atom: &AtomAst, holes: &[String]) -> NormPattern {
    // Atom raw syntax in pattern context remains bounded extraction material:
    // PatternName/PatternNav/HoleRef labels are intentionally distinct from
    // value-side Name/Nav dumps.
    match &atom.kind {
        AtomKind::Name(name) if holes.iter().any(|hole| hole == &name.text) => {
            NormPattern::HoleRef {
                name: name.text.clone(),
                origin: NormOrigin::Source(name.span),
            }
        }
        AtomKind::Name(name) => NormPattern::Name {
            name: name.text.clone(),
            origin: NormOrigin::Source(name.span),
        },
        AtomKind::IntLiteral(text)
        | AtomKind::FloatLiteral(text)
        | AtomKind::StringLiteral(text) => NormPattern::Literal {
            text: text.clone(),
            origin: NormOrigin::Source(atom.span),
        },
        AtomKind::Group(expr) => normalize_expr_as_pattern(expr, holes),
        AtomKind::NavPath { components } => NormPattern::Nav {
            components: components.iter().map(normalize_nav_component).collect(),
            origin: NormOrigin::Source(atom.span),
        },
        AtomKind::Error(error) => NormPattern::Error(normalize_error(error)),
        other => NormPattern::Unsupported {
            raw_kind_summary: annotation_atom_pattern_summary(other),
            origin: NormOrigin::Generated {
                rule: NormRule::Unsupported,
                span: atom.span,
            },
        },
    }
}

fn normalize_with_clause(with_clause: &WithClauseAst) -> NormWithClause {
    match &with_clause.kind {
        WithClauseKind::Empty => NormWithClause {
            names: Vec::new(),
            explicit_empty: true,
            error: None,
            origin: NormOrigin::Source(with_clause.span),
        },
        WithClauseKind::Items { items } => NormWithClause {
            names: items.iter().map(|item| item.text.clone()).collect(),
            explicit_empty: false,
            error: None,
            origin: NormOrigin::Source(with_clause.span),
        },
        WithClauseKind::Error(error) => NormWithClause {
            names: Vec::new(),
            explicit_empty: false,
            error: Some(normalize_error(error)),
            origin: NormOrigin::Source(with_clause.span),
        },
    }
}

fn normalize_canonical_skeleton(skeleton: &CanonicalSkeletonAst) -> NormSkeleton {
    match skeleton {
        CanonicalSkeletonAst::Segment { elements, span } => NormSkeleton::Segment {
            elements: elements.iter().map(normalize_canonical_skeleton).collect(),
            origin: NormOrigin::Source(*span),
        },
        CanonicalSkeletonAst::ProductExtract { elements, span } => NormSkeleton::Product {
            elements: elements
                .iter()
                .map(|element| match element {
                    CanonicalProductElementAst::Skeleton(skeleton) => {
                        NormSkeletonElem::Skeleton(normalize_canonical_skeleton(skeleton))
                    }
                    CanonicalProductElementAst::Unit { span } => NormSkeletonElem::Unit {
                        origin: NormOrigin::Source(*span),
                    },
                })
                .collect(),
            origin: NormOrigin::Source(*span),
        },
        CanonicalSkeletonAst::Wildcard { span } => NormSkeleton::Wildcard {
            origin: NormOrigin::Source(*span),
        },
        CanonicalSkeletonAst::Name { name, role, span } => NormSkeleton::Name {
            name: name.text.clone(),
            role: normalize_canonical_role(*role),
            origin: NormOrigin::Source(*span),
        },
        CanonicalSkeletonAst::NavPath { names, span } => NormSkeleton::Nav {
            names: names.iter().map(|name| name.text.clone()).collect(),
            origin: NormOrigin::Source(*span),
        },
        CanonicalSkeletonAst::Literal { text, span } => NormSkeleton::Literal {
            text: text.clone(),
            origin: NormOrigin::Source(*span),
        },
        CanonicalSkeletonAst::Error(error) => NormSkeleton::Error(normalize_error(error)),
    }
}

fn normalize_canonical_role(role: CanonicalNameRole) -> NormCanonicalNameRole {
    match role {
        CanonicalNameRole::Hole => NormCanonicalNameRole::Hole,
        CanonicalNameRole::NodeName => NormCanonicalNameRole::NodeName,
        CanonicalNameRole::Unknown => NormCanonicalNameRole::Unknown,
    }
}

fn normalize_entity_ref(entity_ref: &EntityRefAst) -> NormEntityRef {
    NormEntityRef {
        components: entity_ref
            .components
            .iter()
            .map(normalize_nav_component)
            .collect(),
        origin: NormOrigin::Generated {
            rule: NormRule::AliasPreserve,
            span: entity_ref.span,
        },
    }
}

fn normalize_nav_component(component: &NavComponentAst) -> NormNavComponent {
    match component {
        NavComponentAst::Text(name) => NormNavComponent::Name {
            name: name.text.clone(),
            origin: NormOrigin::Source(name.span),
        },
        NavComponentAst::Operator(operator) => NormNavComponent::Operator {
            spelling: operator.spelling.clone(),
            origin: NormOrigin::Source(operator.span),
        },
        NavComponentAst::Group(expr) => NormNavComponent::Group {
            expr: Box::new(normalize_expr(expr)),
            origin: NormOrigin::Source(expr.span),
        },
        NavComponentAst::Error(error) => NormNavComponent::Error(normalize_error(error)),
    }
}

fn generated_prefix_negative_closure(span: Span) -> NormClosure {
    let body_expr = make_call(
        NormProduct {
            elements: vec![
                NormProductElem::Expr(generated_nav(
                    &["zero", "T"],
                    span,
                    NormRule::PrefixNegativeLowering,
                )),
                NormProductElem::Expr(generated_name(
                    "val",
                    span,
                    NormRule::PrefixNegativeLowering,
                )),
            ],
            origin: NormOrigin::Generated {
                rule: NormRule::PrefixNegativeLowering,
                span,
            },
        },
        NormExpr::OperatorTarget {
            spelling: "-".to_string(),
            fixity: NormOperatorFixity::Binary,
            arity: 2,
            origin: NormOrigin::Generated {
                rule: NormRule::PrefixNegativeLowering,
                span,
            },
        },
        NormOrigin::Generated {
            rule: NormRule::PrefixNegativeLowering,
            span,
        },
    );
    generated_receiver_closure(NormRule::PrefixNegativeLowering, span, body_expr)
}

fn generated_receiver_closure(rule: NormRule, span: Span, body_expr: NormExpr) -> NormClosure {
    NormClosure {
        kind: NormClosureKind::Generated { rule },
        head: Some(NormClosureHead {
            deduce: vec![NormHoleDecl {
                name: "T".to_string(),
                annotation: Some(NormAnnotation {
                    pattern: NormPattern::Name {
                        name: "type".to_string(),
                        origin: NormOrigin::Generated { rule, span },
                    },
                    origin: NormOrigin::Generated { rule, span },
                }),
                origin: NormOrigin::Generated { rule, span },
            }],
            captures: Vec::new(),
            params: vec![NormPatternElem::BindingSlot(NormBindingSlot {
                policy: None,
                has_let: false,
                deduce: Vec::new(),
                value_pattern: NormPattern::Binder {
                    name: "val".to_string(),
                    origin: NormOrigin::Generated { rule, span },
                },
                annotation: Some(NormAnnotation {
                    pattern: NormPattern::HoleRef {
                        name: "T".to_string(),
                        origin: NormOrigin::Generated { rule, span },
                    },
                    origin: NormOrigin::Generated { rule, span },
                }),
                with_clause: None,
                initializer: None,
                origin: NormOrigin::Generated { rule, span },
            })],
            fn_item_trait: None,
            returns: None,
            clauses: Vec::new(),
            origin: NormOrigin::Generated { rule, span },
        }),
        body: NormClosureBody::Block(NormProgram {
            forms: vec![NormForm::Expr(body_expr)],
            origin: NormOrigin::Generated { rule, span },
        }),
        origin: NormOrigin::Generated { rule, span },
    }
}

fn generated_name(name: &str, span: Span, rule: NormRule) -> NormExpr {
    NormExpr::Name {
        text: name.to_string(),
        origin: NormOrigin::Generated { rule, span },
    }
}

fn generated_nav(names: &[&str], span: Span, rule: NormRule) -> NormExpr {
    NormExpr::Nav {
        components: names
            .iter()
            .map(|name| NormNavComponent::Name {
                name: (*name).to_string(),
                origin: NormOrigin::Generated { rule, span },
            })
            .collect(),
        origin: NormOrigin::Generated { rule, span },
    }
}

fn selector_name(selector: &SelectorAst) -> String {
    match selector {
        SelectorAst::Text(name) => name.text.clone(),
    }
}

fn normalize_error(error: &ErrorAst) -> NormError {
    NormError {
        message: error.message.clone(),
        origin: NormOrigin::Source(error.span),
    }
}

fn expr_span(expr: &NormExpr) -> Option<Span> {
    match expr {
        NormExpr::Call { origin, .. }
        | NormExpr::Name { origin, .. }
        | NormExpr::Literal { origin, .. }
        | NormExpr::Nav { origin, .. }
        | NormExpr::OperatorTarget { origin, .. }
        | NormExpr::Unsupported { origin, .. } => Some(origin_span(origin)),
        NormExpr::Product(product) => Some(origin_span(&product.origin)),
        NormExpr::Closure(closure) => Some(origin_span(&closure.origin)),
        NormExpr::Error(error) => Some(origin_span(&error.origin)),
    }
}

fn origin_span(origin: &NormOrigin) -> Span {
    match origin {
        NormOrigin::Source(span)
        | NormOrigin::Generated { span, .. }
        | NormOrigin::Derived { span, .. } => *span,
    }
}

fn skeleton_span(skeleton: &CanonicalSkeletonAst) -> Span {
    match skeleton {
        CanonicalSkeletonAst::Segment { span, .. }
        | CanonicalSkeletonAst::ProductExtract { span, .. }
        | CanonicalSkeletonAst::Wildcard { span }
        | CanonicalSkeletonAst::Name { span, .. }
        | CanonicalSkeletonAst::NavPath { span, .. }
        | CanonicalSkeletonAst::Literal { span, .. } => *span,
        CanonicalSkeletonAst::Error(error) => error.span,
    }
}

fn annotation_operator_pattern_summary(kind: &OperatorExprKind) -> String {
    match kind {
        OperatorExprKind::OperatorSugar { .. } => {
            "operator sugar in annotation pattern".to_string()
        }
        OperatorExprKind::MemberSugar { .. } => "member sugar in annotation pattern".to_string(),
        OperatorExprKind::DoubleDotSugar { .. } => {
            "double-dot sugar in annotation pattern".to_string()
        }
        OperatorExprKind::BracketCallSugar { .. } => {
            "bracket-call sugar in annotation pattern".to_string()
        }
        _ => "unsupported annotation operator pattern".to_string(),
    }
}

fn annotation_atom_pattern_summary(kind: &AtomKind) -> String {
    match kind {
        AtomKind::MemberSugar { .. } => "member sugar in annotation pattern".to_string(),
        AtomKind::DoubleDotSugar { .. } => "double-dot sugar in annotation pattern".to_string(),
        AtomKind::BracketCallSugar { .. } => "bracket-call sugar in annotation pattern".to_string(),
        AtomKind::Closure(_) => "closure in annotation pattern".to_string(),
        _ => "unsupported annotation atom pattern".to_string(),
    }
}

fn dump_norm_form(output: &mut String, form: &NormForm, indent: usize) {
    match form {
        NormForm::Let(decl) => {
            line(output, indent, "Form Let");
            dump_norm_decl(output, decl, indent + 1);
        }
        NormForm::Alias(decl) => {
            line(output, indent, "Form Alias");
            dump_norm_decl(output, decl, indent + 1);
        }
        NormForm::Expr(expr) => {
            line(output, indent, "Form Expr");
            dump_norm_expr(output, expr, indent + 1);
        }
        NormForm::TailValue(expr) => {
            line(output, indent, "Form TailValue");
            dump_norm_expr(output, expr, indent + 1);
        }
        NormForm::ReturnEvent(return_ev) => {
            line(output, indent, "Form ReturnEvent");
            line(output, indent + 1, "value");
            dump_norm_expr(output, &return_ev.value, indent + 2);
            line(output, indent + 1, "target");
            match &return_ev.target {
                NormReturnTargetSyntax::ImplicitNearest => {
                    line(output, indent + 2, "ImplicitNearest");
                }
                NormReturnTargetSyntax::Explicit(target) => {
                    line(output, indent + 2, "Explicit");
                    dump_norm_expr(output, target, indent + 3);
                }
            }
        }
        NormForm::Error(error) => {
            line(output, indent, "Form Error");
            dump_norm_error(output, error, indent + 1);
        }
    }
}

fn dump_norm_decl(output: &mut String, decl: &NormDecl, indent: usize) {
    match decl {
        NormDecl::Let { slot, origin } => {
            line(
                output,
                indent,
                &format!("Decl Let {}", origin_inline(origin)),
            );
            dump_binding_slot(output, slot, indent + 1);
        }
        NormDecl::Alias {
            policy,
            binder,
            target,
            origin,
        } => {
            line(
                output,
                indent,
                &format!("Decl Alias {}", origin_inline(origin)),
            );
            if let Some(policy) = policy {
                line(output, indent + 1, "policy:");
                dump_norm_expr(output, policy, indent + 2);
            }
            line(output, indent + 1, "binder:");
            dump_alias_binder(output, binder, indent + 2);
            line(output, indent + 1, "target:");
            dump_entity_ref(output, target, indent + 2);
        }
        NormDecl::Error(error) => {
            line(output, indent, "Decl Error");
            dump_norm_error(output, error, indent + 1);
        }
    }
}

fn dump_norm_expr(output: &mut String, expr: &NormExpr, indent: usize) {
    match expr {
        NormExpr::Call {
            source,
            target,
            origin,
        } => {
            line(output, indent, &format!("Call {}", origin_inline(origin)));
            line(output, indent + 1, "source:");
            dump_product(output, source, indent + 2);
            line(output, indent + 1, "target:");
            dump_norm_expr(output, target, indent + 2);
        }
        NormExpr::Product(product) => dump_product(output, product, indent),
        NormExpr::Name { text, origin } => line(
            output,
            indent,
            &format!("Name \"{}\" {}", escape_text(text), origin_inline(origin)),
        ),
        NormExpr::Literal { kind, text, origin } => line(
            output,
            indent,
            &format!(
                "Literal {} \"{}\" {}",
                literal_kind_label(*kind),
                escape_text(text),
                origin_inline(origin)
            ),
        ),
        NormExpr::Nav { components, origin } => {
            line(output, indent, &format!("Nav {}", origin_inline(origin)));
            line(output, indent + 1, "components:");
            for component in components {
                dump_nav_component(output, component, indent + 2);
            }
        }
        NormExpr::Closure(closure) => dump_closure(output, closure, indent),
        NormExpr::OperatorTarget {
            spelling,
            fixity,
            arity,
            origin,
        } => line(
            output,
            indent,
            &format!(
                "OperatorTarget spelling=\"{}\" fixity={} arity={} {}",
                escape_text(spelling),
                norm_fixity_label(*fixity),
                arity,
                origin_inline(origin)
            ),
        ),
        NormExpr::Error(error) => {
            line(output, indent, "Expr Error");
            dump_norm_error(output, error, indent + 1);
        }
        NormExpr::Unsupported {
            raw_kind_summary,
            origin,
        } => line(
            output,
            indent,
            &format!(
                "Unsupported \"{}\" {}",
                escape_text(raw_kind_summary),
                origin_inline(origin)
            ),
        ),
    }
}

fn dump_product(output: &mut String, product: &NormProduct, indent: usize) {
    line(
        output,
        indent,
        &format!("Product {}", origin_inline(&product.origin)),
    );
    line(output, indent + 1, "elements:");
    if product.elements.is_empty() {
        line(output, indent + 2, "(empty)");
    }
    for element in &product.elements {
        match element {
            NormProductElem::Expr(expr) => {
                line(output, indent + 2, "ExprElem");
                dump_norm_expr(output, expr, indent + 3);
            }
            NormProductElem::Unit { origin } => {
                line(
                    output,
                    indent + 2,
                    &format!("Unit {}", origin_inline(origin)),
                );
            }
        }
    }
}

fn dump_binding_slot(output: &mut String, slot: &NormBindingSlot, indent: usize) {
    line(
        output,
        indent,
        &format!(
            "BindingSlot let={} {}",
            slot.has_let,
            origin_inline(&slot.origin)
        ),
    );
    if let Some(policy) = &slot.policy {
        line(output, indent + 1, "policy:");
        dump_norm_expr(output, policy, indent + 2);
    }
    line(output, indent + 1, "deduce:");
    if slot.deduce.is_empty() {
        line(output, indent + 2, "None");
    } else {
        for hole in &slot.deduce {
            dump_hole_decl(output, hole, indent + 2);
        }
    }
    line(output, indent + 1, "value_pattern:");
    dump_pattern(output, &slot.value_pattern, indent + 2);
    line(output, indent + 1, "annotation:");
    match &slot.annotation {
        Some(annotation) => dump_annotation(output, annotation, indent + 2),
        None => line(output, indent + 2, "None"),
    }
    line(output, indent + 1, "with_clause:");
    match &slot.with_clause {
        Some(with_clause) => dump_with_clause(output, with_clause, indent + 2),
        None => line(output, indent + 2, "None"),
    }
    line(output, indent + 1, "initializer:");
    match &slot.initializer {
        Some(initializer) => dump_norm_expr(output, initializer, indent + 2),
        None => line(output, indent + 2, "None"),
    }
}

fn dump_hole_decl(output: &mut String, hole: &NormHoleDecl, indent: usize) {
    line(
        output,
        indent,
        &format!(
            "HoleDecl \"{}\" {}",
            escape_text(&hole.name),
            origin_inline(&hole.origin)
        ),
    );
    line(output, indent + 1, "annotation:");
    match &hole.annotation {
        Some(annotation) => dump_annotation(output, annotation, indent + 2),
        None => line(output, indent + 2, "None"),
    }
}

fn dump_annotation(output: &mut String, annotation: &NormAnnotation, indent: usize) {
    line(
        output,
        indent,
        &format!("AnnotationPattern {}", origin_inline(&annotation.origin)),
    );
    dump_pattern(output, &annotation.pattern, indent + 1);
}

fn dump_pattern(output: &mut String, pattern: &NormPattern, indent: usize) {
    match pattern {
        NormPattern::Binder { name, origin } => line(
            output,
            indent,
            &format!("Binder \"{}\" {}", escape_text(name), origin_inline(origin)),
        ),
        NormPattern::OperatorBinder { spelling, origin } => line(
            output,
            indent,
            &format!(
                "OperatorBinder \"{}\" {}",
                escape_text(spelling),
                origin_inline(origin)
            ),
        ),
        NormPattern::Product { elements, origin } => {
            line(
                output,
                indent,
                &format!("PatternProduct {}", origin_inline(origin)),
            );
            line(output, indent + 1, "elements:");
            if elements.is_empty() {
                line(output, indent + 2, "(empty)");
            }
            for element in elements {
                dump_pattern_elem(output, element, indent + 2);
            }
        }
        NormPattern::Unit { origin } => {
            line(output, indent, &format!("Unit {}", origin_inline(origin)))
        }
        NormPattern::HoleRef { name, origin } => line(
            output,
            indent,
            &format!(
                "HoleRef \"{}\" {}",
                escape_text(name),
                origin_inline(origin)
            ),
        ),
        NormPattern::Name { name, origin } => line(
            output,
            indent,
            &format!(
                "PatternName \"{}\" {}",
                escape_text(name),
                origin_inline(origin)
            ),
        ),
        NormPattern::Literal { text, origin } => line(
            output,
            indent,
            &format!(
                "PatternLiteral \"{}\" {}",
                escape_text(text),
                origin_inline(origin)
            ),
        ),
        NormPattern::Nav { components, origin } => {
            line(
                output,
                indent,
                &format!("PatternNav {}", origin_inline(origin)),
            );
            line(output, indent + 1, "components:");
            for component in components {
                dump_nav_component(output, component, indent + 2);
            }
        }
        NormPattern::Sequence { elements, origin } => {
            line(
                output,
                indent,
                &format!("PatternSequence {}", origin_inline(origin)),
            );
            line(output, indent + 1, "elements:");
            for element in elements {
                dump_pattern(output, element, indent + 2);
            }
        }
        NormPattern::Skeleton { skeleton, origin } => {
            line(
                output,
                indent,
                &format!("PatternSkeleton {}", origin_inline(origin)),
            );
            dump_skeleton(output, skeleton, indent + 1);
        }
        NormPattern::BindingSlot { slot, origin } => {
            line(
                output,
                indent,
                &format!("PatternBindingSlot {}", origin_inline(origin)),
            );
            dump_binding_slot(output, slot, indent + 1);
        }
        NormPattern::Error(error) => {
            line(output, indent, "Pattern Error");
            dump_norm_error(output, error, indent + 1);
        }
        NormPattern::Unsupported {
            raw_kind_summary,
            origin,
        } => line(
            output,
            indent,
            &format!(
                "PatternUnsupported \"{}\" {}",
                escape_text(raw_kind_summary),
                origin_inline(origin)
            ),
        ),
    }
}

fn dump_pattern_elem(output: &mut String, element: &NormPatternElem, indent: usize) {
    match element {
        NormPatternElem::Pattern(pattern) => dump_pattern(output, pattern, indent),
        NormPatternElem::BindingSlot(slot) => {
            line(output, indent, "BindingSlotElem");
            dump_binding_slot(output, slot, indent + 1);
        }
        NormPatternElem::Unit { origin } => {
            line(output, indent, &format!("Unit {}", origin_inline(origin)));
        }
    }
}

fn dump_skeleton(output: &mut String, skeleton: &NormSkeleton, indent: usize) {
    match skeleton {
        NormSkeleton::Segment { elements, origin } => {
            line(
                output,
                indent,
                &format!("SkeletonSegment {}", origin_inline(origin)),
            );
            for element in elements {
                dump_skeleton(output, element, indent + 1);
            }
        }
        NormSkeleton::Product { elements, origin } => {
            line(
                output,
                indent,
                &format!("SkeletonProduct {}", origin_inline(origin)),
            );
            for element in elements {
                match element {
                    NormSkeletonElem::Skeleton(skeleton) => {
                        dump_skeleton(output, skeleton, indent + 1)
                    }
                    NormSkeletonElem::Unit { origin } => {
                        line(
                            output,
                            indent + 1,
                            &format!("Unit {}", origin_inline(origin)),
                        );
                    }
                }
            }
        }
        NormSkeleton::Wildcard { origin } => {
            line(
                output,
                indent,
                &format!("SkeletonWildcard {}", origin_inline(origin)),
            );
        }
        NormSkeleton::Name { name, role, origin } => line(
            output,
            indent,
            &format!(
                "SkeletonName \"{}\" role={} {}",
                escape_text(name),
                canonical_role_label(*role),
                origin_inline(origin)
            ),
        ),
        NormSkeleton::Nav { names, origin } => line(
            output,
            indent,
            &format!(
                "SkeletonNav [{}] {}",
                names
                    .iter()
                    .map(|name| format!("\"{}\"", escape_text(name)))
                    .collect::<Vec<_>>()
                    .join(", "),
                origin_inline(origin)
            ),
        ),
        NormSkeleton::Literal { text, origin } => line(
            output,
            indent,
            &format!(
                "SkeletonLiteral \"{}\" {}",
                escape_text(text),
                origin_inline(origin)
            ),
        ),
        NormSkeleton::Error(error) => {
            line(output, indent, "Skeleton Error");
            dump_norm_error(output, error, indent + 1);
        }
    }
}

fn dump_with_clause(output: &mut String, with_clause: &NormWithClause, indent: usize) {
    line(
        output,
        indent,
        &format!(
            "WithClause explicit_empty={} {}",
            with_clause.explicit_empty,
            origin_inline(&with_clause.origin)
        ),
    );
    if !with_clause.names.is_empty() {
        line(output, indent + 1, "names:");
        for name in &with_clause.names {
            line(output, indent + 2, &format!("\"{}\"", escape_text(name)));
        }
    }
    if let Some(error) = &with_clause.error {
        line(output, indent + 1, "error:");
        dump_norm_error(output, error, indent + 2);
    }
}

fn dump_closure(output: &mut String, closure: &NormClosure, indent: usize) {
    line(
        output,
        indent,
        &format!(
            "Closure kind={} {}",
            closure_kind_label(&closure.kind),
            origin_inline(&closure.origin)
        ),
    );
    line(output, indent + 1, "head:");
    match &closure.head {
        Some(head) => dump_closure_head(output, head, indent + 2),
        None => line(output, indent + 2, "None"),
    }
    line(output, indent + 1, "body:");
    match &closure.body {
        NormClosureBody::Block(program) => dump_norm_program_body(output, program, indent + 2),
        NormClosureBody::Delete(del) => {
            line(output, indent + 2, "Delete");
            let mut msg_buf = String::new();
            dump_norm_expr(&mut msg_buf, &del.message, 0);
            // dump_norm_expr appends a trailing \n; trim it to avoid
            // double-newline when passed through line().
            let msg_text = msg_buf.trim_end_matches('\n');
            line(output, indent + 3, msg_text);
        }
    }
}

fn dump_closure_head(output: &mut String, head: &NormClosureHead, indent: usize) {
    line(
        output,
        indent,
        &format!("ClosureHead {}", origin_inline(&head.origin)),
    );
    line(output, indent + 1, "deduce:");
    if head.deduce.is_empty() {
        line(output, indent + 2, "None");
    } else {
        for hole in &head.deduce {
            dump_hole_decl(output, hole, indent + 2);
        }
    }
    if !head.captures.is_empty() {
        line(output, indent + 1, "captures:");
        for capture in &head.captures {
            dump_norm_expr(output, capture, indent + 2);
        }
    }
    line(output, indent + 1, "params:");
    if head.params.is_empty() {
        line(output, indent + 2, "None");
    } else {
        for param in &head.params {
            dump_pattern_elem(output, param, indent + 2);
        }
    }
    if let Some(annotation) = &head.fn_item_trait {
        line(output, indent + 1, "fn_item_trait:");
        dump_annotation(output, annotation, indent + 2);
    }
    if let Some(returns) = &head.returns {
        line(output, indent + 1, "returns:");
        dump_binding_slot(output, returns, indent + 2);
    }
    if !head.clauses.is_empty() {
        line(output, indent + 1, "clauses:");
        for clause in &head.clauses {
            dump_head_clause(output, clause, indent + 2);
        }
    }
}

fn dump_head_clause(output: &mut String, clause: &NormHeadClause, indent: usize) {
    match clause {
        NormHeadClause::Require { expr, origin } => {
            dump_named_clause(output, "Require", expr, origin, indent)
        }
        NormHeadClause::Pre { expr, origin } => {
            dump_named_clause(output, "Pre", expr, origin, indent)
        }
        NormHeadClause::Post { expr, origin } => {
            dump_named_clause(output, "Post", expr, origin, indent)
        }
        NormHeadClause::LifetimePre { expr, origin } => {
            dump_named_clause(output, "LifetimePre", expr, origin, indent)
        }
        NormHeadClause::LifetimePost { expr, origin } => {
            dump_named_clause(output, "LifetimePost", expr, origin, indent)
        }
        NormHeadClause::Error(error) => {
            line(output, indent, "HeadClause Error");
            dump_norm_error(output, error, indent + 1);
        }
    }
}

fn dump_named_clause(
    output: &mut String,
    name: &str,
    expr: &NormExpr,
    origin: &NormOrigin,
    indent: usize,
) {
    line(
        output,
        indent,
        &format!("{} {}", name, origin_inline(origin)),
    );
    dump_norm_expr(output, expr, indent + 1);
}

fn dump_norm_program_body(output: &mut String, program: &NormProgram, indent: usize) {
    line(
        output,
        indent,
        &format!("NormBody {}", origin_inline(&program.origin)),
    );
    line(output, indent + 1, "forms:");
    if program.forms.is_empty() {
        line(output, indent + 2, "(empty)");
    }
    for form in &program.forms {
        dump_norm_form(output, form, indent + 2);
    }
}

fn dump_alias_binder(output: &mut String, binder: &NormAliasBinder, indent: usize) {
    match binder {
        NormAliasBinder::Name { name, origin } => line(
            output,
            indent,
            &format!("Name \"{}\" {}", escape_text(name), origin_inline(origin)),
        ),
        NormAliasBinder::Operator { spelling, origin } => line(
            output,
            indent,
            &format!(
                "Operator \"{}\" {}",
                escape_text(spelling),
                origin_inline(origin)
            ),
        ),
        NormAliasBinder::Error(error) => {
            line(output, indent, "AliasBinder Error");
            dump_norm_error(output, error, indent + 1);
        }
    }
}

fn dump_entity_ref(output: &mut String, entity_ref: &NormEntityRef, indent: usize) {
    line(
        output,
        indent,
        &format!("EntityRef {}", origin_inline(&entity_ref.origin)),
    );
    line(output, indent + 1, "components:");
    for component in &entity_ref.components {
        dump_nav_component(output, component, indent + 2);
    }
}

fn dump_nav_component(output: &mut String, component: &NormNavComponent, indent: usize) {
    match component {
        NormNavComponent::Name { name, origin } => line(
            output,
            indent,
            &format!("Name \"{}\" {}", escape_text(name), origin_inline(origin)),
        ),
        NormNavComponent::Operator { spelling, origin } => line(
            output,
            indent,
            &format!(
                "Operator \"{}\" {}",
                escape_text(spelling),
                origin_inline(origin)
            ),
        ),
        NormNavComponent::Group { expr, origin } => {
            line(output, indent, &format!("Group {}", origin_inline(origin)));
            dump_norm_expr(output, expr, indent + 1);
        }
        NormNavComponent::Error(error) => {
            line(output, indent, "NavComponent Error");
            dump_norm_error(output, error, indent + 1);
        }
    }
}

fn dump_norm_error(output: &mut String, error: &NormError, indent: usize) {
    line(
        output,
        indent,
        &format!(
            "Error \"{}\" {}",
            escape_text(&error.message),
            origin_inline(&error.origin)
        ),
    );
}

fn origin_inline(origin: &NormOrigin) -> String {
    match origin {
        NormOrigin::Source(span) => format!("origin=Source{}", span_inline(*span)),
        NormOrigin::Generated { rule, span } => {
            format!(
                "origin=Generated({}){}",
                rule_label(*rule),
                span_inline(*span)
            )
        }
        NormOrigin::Derived {
            rule,
            span,
            summary,
        } => format!(
            "origin=Derived({}; {}){}",
            rule_label(*rule),
            escape_text(summary),
            span_inline(*span)
        ),
    }
}

fn span_inline(span: Span) -> String {
    format!(
        "@{}:{}[{}..{}]",
        span.line, span.column, span.byte_start, span.byte_end
    )
}

fn line(output: &mut String, indent: usize, text: &str) {
    for _ in 0..indent {
        output.push_str("  ");
    }
    output.push_str(text);
    output.push('\n');
}

fn rule_label(rule: NormRule) -> &'static str {
    match rule {
        NormRule::ProductLift => "ProductLift",
        NormRule::ProductMerge => "ProductMerge",
        NormRule::PipeFallback => "PipeFallback",
        NormRule::SecondLegalityRepair => "SecondLegalityRepair",
        NormRule::OperatorLowering => "OperatorLowering",
        NormRule::PrefixNegativeLowering => "PrefixNegativeLowering",
        NormRule::MemberLowering => "MemberLowering",
        NormRule::DoubleDotLowering => "DoubleDotLowering",
        NormRule::BracketCallLowering => "BracketCallLowering",
        NormRule::BranchNameExpansion => "BranchNameExpansion",
        NormRule::AliasPreserve => "AliasPreserve",
        NormRule::ClosureNormalize => "ClosureNormalize",
        NormRule::PatternNormalize => "PatternNormalize",
        NormRule::Unsupported => "Unsupported",
    }
}

fn raw_fixity_label(fixity: OperatorFixity) -> &'static str {
    match fixity {
        OperatorFixity::Prefix => "Prefix",
        OperatorFixity::Postfix => "Postfix",
        OperatorFixity::Binary => "Binary",
    }
}

fn norm_fixity_label(fixity: NormOperatorFixity) -> &'static str {
    match fixity {
        NormOperatorFixity::Prefix => "Prefix",
        NormOperatorFixity::Postfix => "Postfix",
        NormOperatorFixity::Binary => "Binary",
        NormOperatorFixity::BracketCall => "BracketCall",
    }
}

fn literal_kind_label(kind: NormLiteralKind) -> &'static str {
    match kind {
        NormLiteralKind::Int => "Int",
        NormLiteralKind::Float => "Float",
        NormLiteralKind::String => "String",
    }
}

fn closure_kind_label(kind: &NormClosureKind) -> String {
    match kind {
        NormClosureKind::InPlace => "InPlace".to_string(),
        NormClosureKind::Explicit => "Explicit".to_string(),
        NormClosureKind::Generated { rule } => format!("Generated({})", rule_label(*rule)),
    }
}

fn canonical_role_label(role: NormCanonicalNameRole) -> &'static str {
    match role {
        NormCanonicalNameRole::Hole => "Hole",
        NormCanonicalNameRole::NodeName => "NodeName",
        NormCanonicalNameRole::Unknown => "Unknown",
    }
}

fn escape_text(text: &str) -> String {
    let mut escaped = String::new();

    for ch in text.chars() {
        match ch {
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            _ => escaped.push(ch),
        }
    }

    escaped
}
