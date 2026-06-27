use lang_syntax::{NormExpr, NormForm, NormNavComponent, NormOrigin, NormProductElem, NormProgram};

use crate::{
    graph::{BuildError, NamespaceGraphSnapshot, ResolveExpectation, ResolverContext},
    model::{
        Diagnostic, FieldProjection, NamespaceNodeId, NamespaceNodeKind, PolicyFlag, Provenance,
        SymbolKind, SymbolObject, SymbolPayload,
    },
};

const VERIFY_ENTRY: &str = "verify";
const VERIFY_ERROR_PREFIX: &str = "source verification error:";

pub fn evaluate_source_verifications(
    snapshot: &NamespaceGraphSnapshot,
    namespace: NamespaceNodeId,
    program: &NormProgram,
    context: &ResolverContext,
) -> Result<Vec<Diagnostic>, BuildError> {
    let _ = namespace;

    let mut diagnostics = Vec::new();
    for form in &program.forms {
        let NormForm::Expr(expr) = form else {
            continue;
        };
        let Some(invocation) = VerificationInvocation::from_expr(expr) else {
            continue;
        };
        if let Err(diagnostic) = invocation.evaluate(snapshot, context) {
            diagnostics.push(diagnostic);
        }
    }
    Ok(diagnostics)
}

#[derive(Clone, Debug)]
struct VerificationInvocation {
    operation: String,
    args: Vec<VerificationArg>,
    origin: NormOrigin,
}

impl VerificationInvocation {
    fn from_expr(expr: &NormExpr) -> Option<Self> {
        let mut terms = Vec::new();
        flatten_call_chain(expr, &mut terms);
        if terms.len() < 2 {
            return None;
        }
        if !matches!(terms.first(), Some(VerificationArg::Name(name, _)) if name == VERIFY_ENTRY) {
            return None;
        }

        let (operation, origin) = match terms.get(1)? {
            VerificationArg::Name(name, origin) => (name.clone(), origin.clone()),
            _ => {
                return Some(Self {
                    operation: "<invalid>".to_string(),
                    args: terms.into_iter().skip(1).collect(),
                    origin: expr_origin(expr).clone(),
                })
            }
        };
        Some(Self {
            operation,
            args: terms.into_iter().skip(2).collect(),
            origin,
        })
    }

    fn evaluate(
        &self,
        snapshot: &NamespaceGraphSnapshot,
        context: &ResolverContext,
    ) -> Result<(), Diagnostic> {
        match self.operation.as_str() {
            "exists" => self.expect_exists(snapshot, context, true),
            "not_exists" => self.expect_exists(snapshot, context, false),
            "resolves_as" => self.expect_kind(snapshot, context),
            "not_resolves" => self.expect_not_resolves(snapshot, context),
            "kind" => self.expect_kind(snapshot, context),
            "namespace_kind" => self.expect_namespace_kind(snapshot, context),
            "field_names" => self.expect_field_names(snapshot, context),
            "has_field" => self.expect_has_field(snapshot, context),
            "field_projection" => self.expect_field_projection(snapshot, context),
            "field_owner" => self.expect_field_owner(snapshot, context),
            "field_type" => self.expect_field_type(snapshot, context),
            "policy" => self.expect_policy(snapshot, context, PolicyCheck::Present),
            "not_policy" => self.expect_policy(snapshot, context, PolicyCheck::Absent),
            "body_entry_policy" => {
                self.expect_callable_policy(snapshot, context, CallablePolicyPlane::BodyEntry, true)
            }
            "not_body_entry_policy" => self.expect_callable_policy(
                snapshot,
                context,
                CallablePolicyPlane::BodyEntry,
                false,
            ),
            "return_policy" => {
                self.expect_callable_policy(snapshot, context, CallablePolicyPlane::Return, true)
            }
            "not_return_policy" => {
                self.expect_callable_policy(snapshot, context, CallablePolicyPlane::Return, false)
            }
            _ => Err(self.error(format!(
                "unknown verification operation `{}`",
                self.operation
            ))),
        }
    }

    fn expect_exists(
        &self,
        snapshot: &NamespaceGraphSnapshot,
        context: &ResolverContext,
        should_exist: bool,
    ) -> Result<(), Diagnostic> {
        self.expect_arity(1)?;
        let path = self.arg_path(0)?;
        let exists = resolve_any_role(snapshot, context, &path).is_ok();
        match (should_exist, exists) {
            (true, true) | (false, false) => Ok(()),
            (true, false) => Err(self.error(format!(
                "expected `{}` to exist",
                path.source_order_display()
            ))),
            (false, true) => Err(self.error(format!(
                "expected `{}` not to exist",
                path.source_order_display()
            ))),
        }
    }

    fn expect_not_resolves(
        &self,
        snapshot: &NamespaceGraphSnapshot,
        context: &ResolverContext,
    ) -> Result<(), Diagnostic> {
        self.expect_arity(1)?;
        let path = self.arg_path(0)?;
        match snapshot.capability().resolve_with_expectation(
            &path.components,
            context,
            ResolveExpectation::AnyUnique,
        ) {
            Ok(_) => Err(self.error(format!(
                "expected `{}` not to resolve",
                path.source_order_display()
            ))),
            Err(_) => Ok(()),
        }
    }

    fn expect_kind(
        &self,
        snapshot: &NamespaceGraphSnapshot,
        context: &ResolverContext,
    ) -> Result<(), Diagnostic> {
        self.expect_arity(2)?;
        let path = self.arg_path(0)?;
        let expected = self.arg_symbol_kind(1)?;
        let symbol = resolve_expected_kind(snapshot, context, &path, expected).map_err(|_| {
            self.error(format!(
                "expected `{}` to resolve as {}",
                path.source_order_display(),
                symbol_kind_label(expected)
            ))
        })?;
        if symbol.kind == expected {
            Ok(())
        } else {
            Err(self.error(format!(
                "expected `{}` to resolve as {}, got {}",
                path.source_order_display(),
                symbol_kind_label(expected),
                symbol_kind_label(symbol.kind)
            )))
        }
    }

    fn expect_namespace_kind(
        &self,
        snapshot: &NamespaceGraphSnapshot,
        context: &ResolverContext,
    ) -> Result<(), Diagnostic> {
        self.expect_arity(2)?;
        let path = self.arg_path(0)?;
        let expected = self.arg_namespace_kind(1)?;
        let symbol = snapshot
            .capability()
            .resolve_with_expectation(
                &path.components,
                context,
                ResolveExpectation::NamespaceSubspace,
            )
            .map_err(|_| {
                self.error(format!(
                    "expected `{}` to resolve as namespace",
                    path.source_order_display()
                ))
            })?;
        let actual = symbol.node_kind.ok_or_else(|| {
            self.error(format!(
                "expected `{}` to carry a namespace node kind",
                path.source_order_display()
            ))
        })?;
        if actual == expected {
            Ok(())
        } else {
            Err(self.error(format!(
                "expected `{}` namespace kind {}, got {}",
                path.source_order_display(),
                namespace_kind_label(expected),
                namespace_kind_label(actual)
            )))
        }
    }

    fn expect_field_names(
        &self,
        snapshot: &NamespaceGraphSnapshot,
        context: &ResolverContext,
    ) -> Result<(), Diagnostic> {
        self.expect_min_arity(1)?;
        let path = self.arg_path(0)?;
        let type_object = self.resolve_type_payload(snapshot, context, &path)?;
        let expected = self
            .args
            .iter()
            .skip(1)
            .map(VerificationArg::as_name)
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| self.error("field_names expects name arguments"))?;
        if type_object.field_names == expected {
            Ok(())
        } else {
            Err(self.error(format!(
                "expected `{}` fields [{}], got [{}]",
                path.source_order_display(),
                expected.join(", "),
                type_object.field_names.join(", ")
            )))
        }
    }

    fn expect_has_field(
        &self,
        snapshot: &NamespaceGraphSnapshot,
        context: &ResolverContext,
    ) -> Result<(), Diagnostic> {
        self.expect_arity(2)?;
        let path = self.arg_path(0)?;
        let field_name = self.arg_name(1)?;
        let type_object = self.resolve_type_payload(snapshot, context, &path)?;
        if type_object
            .field_names
            .iter()
            .any(|name| name == &field_name)
        {
            Ok(())
        } else {
            Err(self.error(format!(
                "expected `{}` to have field `{field_name}`",
                path.source_order_display()
            )))
        }
    }

    fn expect_field_projection(
        &self,
        snapshot: &NamespaceGraphSnapshot,
        context: &ResolverContext,
    ) -> Result<(), Diagnostic> {
        self.expect_arity(2)?;
        let path = self.arg_path(0)?;
        let expected = self.arg_field_projection(1)?;
        let field = self.resolve_field_payload(snapshot, context, &path)?;
        if field.projection == expected {
            Ok(())
        } else {
            Err(self.error(format!(
                "expected `{}` projection {}, got {}",
                path.source_order_display(),
                field_projection_label(expected),
                field_projection_label(field.projection)
            )))
        }
    }

    fn expect_field_owner(
        &self,
        snapshot: &NamespaceGraphSnapshot,
        context: &ResolverContext,
    ) -> Result<(), Diagnostic> {
        self.expect_arity(2)?;
        let field_path = self.arg_path(0)?;
        let owner_path = self.arg_path(1)?;
        let field = self.resolve_field_payload(snapshot, context, &field_path)?;
        let owner = resolve_expected_kind(snapshot, context, &owner_path, SymbolKind::Type)
            .map_err(|_| {
                self.error(format!(
                    "expected `{}` to resolve as type",
                    owner_path.source_order_display()
                ))
            })?;
        if field.owner_type_symbol_id == owner.id {
            Ok(())
        } else {
            Err(self.error(format!(
                "expected `{}` owner `{}`",
                field_path.source_order_display(),
                owner_path.source_order_display()
            )))
        }
    }

    fn expect_field_type(
        &self,
        snapshot: &NamespaceGraphSnapshot,
        context: &ResolverContext,
    ) -> Result<(), Diagnostic> {
        self.expect_arity(2)?;
        let field_path = self.arg_path(0)?;
        let type_path = self.arg_path(1)?;
        let field = self.resolve_field_payload(snapshot, context, &field_path)?;
        let field_type = resolve_expected_kind(snapshot, context, &type_path, SymbolKind::Type)
            .map_err(|_| {
                self.error(format!(
                    "expected `{}` to resolve as type",
                    type_path.source_order_display()
                ))
            })?;
        if field.field_type_symbol_id == field_type.id {
            Ok(())
        } else {
            Err(self.error(format!(
                "expected `{}` field type `{}`",
                field_path.source_order_display(),
                type_path.source_order_display()
            )))
        }
    }

    fn expect_policy(
        &self,
        snapshot: &NamespaceGraphSnapshot,
        context: &ResolverContext,
        check: PolicyCheck,
    ) -> Result<(), Diagnostic> {
        self.expect_arity(2)?;
        let path = self.arg_path(0)?;
        let flag = self.arg_policy_flag(1)?;
        let symbol = resolve_any_role(snapshot, context, &path).map_err(|_| {
            self.error(format!(
                "expected `{}` to resolve for policy verification",
                path.source_order_display()
            ))
        })?;
        let contains = symbol.policy_metadata.policy_set.contains(flag);
        match (check, contains) {
            (PolicyCheck::Present, true) | (PolicyCheck::Absent, false) => Ok(()),
            (PolicyCheck::Present, false) => Err(self.error(format!(
                "expected `{}` policy {}",
                path.source_order_display(),
                policy_flag_label(flag)
            ))),
            (PolicyCheck::Absent, true) => Err(self.error(format!(
                "expected `{}` not to have policy {}",
                path.source_order_display(),
                policy_flag_label(flag)
            ))),
        }
    }

    fn expect_callable_policy(
        &self,
        snapshot: &NamespaceGraphSnapshot,
        context: &ResolverContext,
        plane: CallablePolicyPlane,
        should_contain: bool,
    ) -> Result<(), Diagnostic> {
        self.expect_arity(2)?;
        let path = self.arg_path(0)?;
        let flag = self.arg_policy_flag(1)?;
        let symbol = resolve_callable_symbol(snapshot, context, &path).map_err(|_| {
            self.error(format!(
                "expected `{}` to resolve as callable",
                path.source_order_display()
            ))
        })?;
        let policy = match (&symbol.payload, plane) {
            (SymbolPayload::FieldFunction(field), CallablePolicyPlane::BodyEntry) => {
                &field.callable_policy.body_entry_policy
            }
            (SymbolPayload::FieldFunction(field), CallablePolicyPlane::Return) => {
                &field.callable_policy.return_object_policy
            }
            (SymbolPayload::MetaFunction(meta_function), CallablePolicyPlane::BodyEntry) => {
                &meta_function.body_entry_policy
            }
            (SymbolPayload::MetaFunction(meta_function), CallablePolicyPlane::Return) => {
                &meta_function.return_object_policy
            }
            _ => {
                return Err(self.error(format!(
                    "expected `{}` to carry callable policy metadata",
                    path.source_order_display()
                )));
            }
        };
        let contains = policy.policy_set.contains(flag);
        match (should_contain, contains) {
            (true, true) | (false, false) => Ok(()),
            (true, false) => Err(self.error(format!(
                "expected `{}` {} policy {}",
                path.source_order_display(),
                plane.label(),
                policy_flag_label(flag)
            ))),
            (false, true) => Err(self.error(format!(
                "expected `{}` not to have {} policy {}",
                path.source_order_display(),
                plane.label(),
                policy_flag_label(flag)
            ))),
        }
    }

    fn resolve_type_payload(
        &self,
        snapshot: &NamespaceGraphSnapshot,
        context: &ResolverContext,
        path: &SourcePath,
    ) -> Result<crate::model::TypeObject, Diagnostic> {
        let symbol =
            resolve_expected_kind(snapshot, context, path, SymbolKind::Type).map_err(|_| {
                self.error(format!(
                    "expected `{}` to resolve as type",
                    path.source_order_display()
                ))
            })?;
        match symbol.payload {
            SymbolPayload::Type(type_object) => Ok(type_object),
            _ => Err(self.error(format!(
                "expected `{}` to carry a type payload",
                path.source_order_display()
            ))),
        }
    }

    fn resolve_field_payload(
        &self,
        snapshot: &NamespaceGraphSnapshot,
        context: &ResolverContext,
        path: &SourcePath,
    ) -> Result<crate::model::FieldObject, Diagnostic> {
        let symbol = resolve_expected_kind(snapshot, context, path, SymbolKind::FieldFunction)
            .map_err(|_| {
                self.error(format!(
                    "expected `{}` to resolve as field_function",
                    path.source_order_display()
                ))
            })?;
        match symbol.payload {
            SymbolPayload::FieldFunction(field) => Ok(field),
            _ => Err(self.error(format!(
                "expected `{}` to carry a field-function payload",
                path.source_order_display()
            ))),
        }
    }

    fn expect_arity(&self, expected: usize) -> Result<(), Diagnostic> {
        if self.args.len() == expected {
            Ok(())
        } else {
            Err(self.error(format!(
                "`verify {}` expects {expected} argument(s), got {}",
                self.operation,
                self.args.len()
            )))
        }
    }

    fn expect_min_arity(&self, expected: usize) -> Result<(), Diagnostic> {
        if self.args.len() >= expected {
            Ok(())
        } else {
            Err(self.error(format!(
                "`verify {}` expects at least {expected} argument(s), got {}",
                self.operation,
                self.args.len()
            )))
        }
    }

    fn arg_path(&self, index: usize) -> Result<SourcePath, Diagnostic> {
        self.args
            .get(index)
            .and_then(VerificationArg::as_path)
            .ok_or_else(|| {
                self.error(format!(
                    "`verify {}` argument {} must be a name/path",
                    self.operation,
                    index + 1
                ))
            })
    }

    fn arg_name(&self, index: usize) -> Result<String, Diagnostic> {
        self.args
            .get(index)
            .and_then(VerificationArg::as_name)
            .ok_or_else(|| {
                self.error(format!(
                    "`verify {}` argument {} must be a name",
                    self.operation,
                    index + 1
                ))
            })
    }

    fn arg_symbol_kind(&self, index: usize) -> Result<SymbolKind, Diagnostic> {
        let name = self.arg_name(index)?;
        parse_symbol_kind(&name).ok_or_else(|| self.error(format!("unknown symbol kind `{name}`")))
    }

    fn arg_namespace_kind(&self, index: usize) -> Result<NamespaceNodeKind, Diagnostic> {
        let name = self.arg_name(index)?;
        parse_namespace_kind(&name)
            .ok_or_else(|| self.error(format!("unknown namespace kind `{name}`")))
    }

    fn arg_policy_flag(&self, index: usize) -> Result<PolicyFlag, Diagnostic> {
        let name = self.arg_name(index)?;
        parse_policy_flag(&name).ok_or_else(|| self.error(format!("unknown policy flag `{name}`")))
    }

    fn arg_field_projection(&self, index: usize) -> Result<FieldProjection, Diagnostic> {
        let name = self.arg_name(index)?;
        parse_field_projection(&name)
            .ok_or_else(|| self.error(format!("unknown field projection `{name}`")))
    }

    fn error(&self, message: impl Into<String>) -> Diagnostic {
        Diagnostic::hard_error(
            format!("{VERIFY_ERROR_PREFIX} {}", message.into()),
            Some(Provenance::from_norm_origin(
                format!("verify {}", self.operation),
                &self.origin,
            )),
        )
    }
}

#[derive(Clone, Debug)]
enum VerificationArg {
    Name(String, NormOrigin),
    Path(SourcePath),
    Unsupported,
}

impl VerificationArg {
    fn from_expr(expr: &NormExpr) -> Self {
        match expr {
            NormExpr::Name { text, origin } => Self::Name(text.clone(), origin.clone()),
            NormExpr::Nav { components, origin } => {
                let _ = origin;
                let path = components_to_path(components);
                match path {
                    Some(path) => Self::Path(path),
                    None => Self::Unsupported,
                }
            }
            _ => Self::Unsupported,
        }
    }

    fn as_name(&self) -> Option<String> {
        match self {
            Self::Name(name, _) => Some(name.clone()),
            _ => None,
        }
    }

    fn as_path(&self) -> Option<SourcePath> {
        match self {
            Self::Name(name, _) => Some(SourcePath {
                components: vec![name.clone()],
            }),
            Self::Path(path) => Some(path.clone()),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
struct SourcePath {
    components: Vec<String>,
}

impl SourcePath {
    fn source_order_display(&self) -> String {
        self.components.join("::")
    }
}

#[derive(Clone, Copy, Debug)]
enum PolicyCheck {
    Present,
    Absent,
}

#[derive(Clone, Copy, Debug)]
enum CallablePolicyPlane {
    BodyEntry,
    Return,
}

impl CallablePolicyPlane {
    fn label(self) -> &'static str {
        match self {
            Self::BodyEntry => "body-entry",
            Self::Return => "return",
        }
    }
}

fn flatten_call_chain(expr: &NormExpr, terms: &mut Vec<VerificationArg>) {
    match expr {
        NormExpr::Call { source, target, .. } if source.elements.len() == 1 => {
            if let Some(NormProductElem::Expr(source_expr)) = source.elements.first() {
                flatten_call_chain(source_expr, terms);
                terms.push(VerificationArg::from_expr(target));
                return;
            }
            terms.push(VerificationArg::from_expr(expr));
        }
        _ => terms.push(VerificationArg::from_expr(expr)),
    }
}

fn components_to_path(components: &[NormNavComponent]) -> Option<SourcePath> {
    let mut path = Vec::with_capacity(components.len());
    for component in components {
        match component {
            NormNavComponent::Name { name, .. } => path.push(name.clone()),
            _ => return None,
        }
    }
    Some(SourcePath { components: path })
}

fn resolve_any_role(
    snapshot: &NamespaceGraphSnapshot,
    context: &ResolverContext,
    path: &SourcePath,
) -> Result<SymbolObject, Diagnostic> {
    let capability = snapshot.capability();
    match capability.resolve_with_expectation(&path.components, context, ResolveExpectation::Object)
    {
        Ok(symbol) => Ok(symbol),
        Err(object_error) => capability
            .resolve_with_expectation(
                &path.components,
                context,
                ResolveExpectation::NamespaceSubspace,
            )
            .map_err(|_| object_error),
    }
}

fn resolve_expected_kind(
    snapshot: &NamespaceGraphSnapshot,
    context: &ResolverContext,
    path: &SourcePath,
    kind: SymbolKind,
) -> Result<SymbolObject, Diagnostic> {
    let expectation = match kind {
        SymbolKind::Namespace => ResolveExpectation::NamespaceSubspace,
        SymbolKind::Type => ResolveExpectation::TypeObject,
        SymbolKind::MetaFunction => ResolveExpectation::MetaFunction,
        SymbolKind::FieldFunction => ResolveExpectation::FieldFunction,
        SymbolKind::Alias | SymbolKind::Placeholder => ResolveExpectation::Object,
    };
    let symbol =
        snapshot
            .capability()
            .resolve_with_expectation(&path.components, context, expectation)?;
    if symbol.kind == kind {
        Ok(symbol)
    } else {
        Err(Diagnostic::hard_error(
            "resolved symbol has unexpected kind",
            Some(symbol.provenance),
        ))
    }
}

fn resolve_callable_symbol(
    snapshot: &NamespaceGraphSnapshot,
    context: &ResolverContext,
    path: &SourcePath,
) -> Result<SymbolObject, Diagnostic> {
    snapshot
        .capability()
        .resolve_with_expectation(&path.components, context, ResolveExpectation::FieldFunction)
        .or_else(|_| {
            snapshot.capability().resolve_with_expectation(
                &path.components,
                context,
                ResolveExpectation::MetaFunction,
            )
        })
}

fn parse_symbol_kind(name: &str) -> Option<SymbolKind> {
    match name {
        "namespace" => Some(SymbolKind::Namespace),
        "type" => Some(SymbolKind::Type),
        "meta_function" => Some(SymbolKind::MetaFunction),
        "field_function" => Some(SymbolKind::FieldFunction),
        "alias" => Some(SymbolKind::Alias),
        "placeholder" => Some(SymbolKind::Placeholder),
        _ => None,
    }
}

fn symbol_kind_label(kind: SymbolKind) -> &'static str {
    match kind {
        SymbolKind::Namespace => "namespace",
        SymbolKind::Type => "type",
        SymbolKind::MetaFunction => "meta_function",
        SymbolKind::FieldFunction => "field_function",
        SymbolKind::Alias => "alias",
        SymbolKind::Placeholder => "placeholder",
    }
}

fn parse_namespace_kind(name: &str) -> Option<NamespaceNodeKind> {
    match name {
        "physical" => Some(NamespaceNodeKind::Physical),
        "declared" => Some(NamespaceNodeKind::Declared),
        "virtual" => Some(NamespaceNodeKind::Virtual),
        _ => None,
    }
}

fn namespace_kind_label(kind: NamespaceNodeKind) -> &'static str {
    match kind {
        NamespaceNodeKind::Physical => "physical",
        NamespaceNodeKind::Declared => "declared",
        NamespaceNodeKind::Virtual => "virtual",
    }
}

fn parse_policy_flag(name: &str) -> Option<PolicyFlag> {
    match name {
        "export" => Some(PolicyFlag::Export),
        "meta" => Some(PolicyFlag::Meta),
        "runtime" => Some(PolicyFlag::Runtime),
        _ => None,
    }
}

fn policy_flag_label(flag: PolicyFlag) -> &'static str {
    match flag {
        PolicyFlag::Export => "export",
        PolicyFlag::Meta => "meta",
        PolicyFlag::Runtime => "runtime",
    }
}

fn parse_field_projection(name: &str) -> Option<FieldProjection> {
    match name {
        "value" => Some(FieldProjection::Value),
        "ref" => Some(FieldProjection::Ref),
        "share" => Some(FieldProjection::Share),
        _ => None,
    }
}

fn field_projection_label(projection: FieldProjection) -> &'static str {
    match projection {
        FieldProjection::Value => "value",
        FieldProjection::Ref => "ref",
        FieldProjection::Share => "share",
    }
}

fn expr_origin(expr: &NormExpr) -> &NormOrigin {
    match expr {
        NormExpr::Call { origin, .. }
        | NormExpr::Name { origin, .. }
        | NormExpr::Literal { origin, .. }
        | NormExpr::Nav { origin, .. }
        | NormExpr::OperatorTarget { origin, .. }
        | NormExpr::Unsupported { origin, .. } => origin,
        NormExpr::Product(product) => &product.origin,
        NormExpr::Closure(closure) => &closure.origin,
        NormExpr::Error(error) => &error.origin,
    }
}
