use crate::{
    callable_body::BuiltinBodyInput,
    candidate_preparation::{
        prepare_callable_candidate_with_declared_planes, CandidatePrepIncompleteReason,
        CandidatePrepResult, CandidatePreparationContext, ParameterShape,
    },
    model::{BuiltinCallableImpl, Diagnostic, Provenance},
    policy_pair::ObservationHorizon,
    product_shape::ArgProductShape,
    semantic_name_index::BuildError,
    semantic_world::{OrdinaryCallEntry, OrdinaryCallableImplementation},
};

/// Distinguish decided argument diagnostics from an incomplete applicability relation.
#[derive(Debug)]
pub(crate) enum BuiltinPreparationFailure {
    Diagnostic(BuildError),
    /// Applicability is unknown; the candidate family cannot reach maxima.
    Incomplete(Diagnostic),
}

impl From<BuildError> for BuiltinPreparationFailure {
    fn from(error: BuildError) -> Self {
        Self::Diagnostic(error)
    }
}

/// Prepare builtin argument relations from the real call entry.
/// Its implementation selects argument handling; its declared Policy planes
/// remain the only Policy authority. No graph callable payload is read.
pub(crate) fn prepare_resolved_builtin_call(
    entry: &OrdinaryCallEntry,
    classified_shape: ArgProductShape,
    horizon: ObservationHorizon,
    provenance: Provenance,
) -> Result<BuiltinBodyInput, BuiltinPreparationFailure> {
    let OrdinaryCallableImplementation::Builtin(primitive) = entry.implementation else {
        unreachable!("builtin preparation requires a builtin call entry");
    };
    let primitive_name = match primitive {
        BuiltinCallableImpl::Struct => "struct",
        BuiltinCallableImpl::Assert => "assert",
        BuiltinCallableImpl::Verify(_) => "verify",
        BuiltinCallableImpl::IdentityType => "IdentityType",
    };
    let parameter_shape = match primitive {
        BuiltinCallableImpl::IdentityType | BuiltinCallableImpl::Struct => {
            ParameterShape::type_parameter_signature(Provenance::new(format!(
                "{primitive_name} : type -> type signature"
            )))
        }
        BuiltinCallableImpl::Assert | BuiltinCallableImpl::Verify(_) => {
            return Err(BuiltinPreparationFailure::Incomplete(Diagnostic::hard_error(
                format!("builtin `{primitive_name}` applicability relation consumer is not connected"),
                Some(provenance),
            )));
        }
    };

    // All Policy planes are the real call entry's already declared facts.
    // Implementation identity supplies argument handling, never Policy.
    let candidate = match prepare_callable_candidate_with_declared_planes(
        entry.backing_declaration,
        &entry.declaration_name,
        entry.callable_view.clone(),
        entry.body_entry_view.clone(),
        entry.complete_result_view.clone(),
        classified_shape,
        parameter_shape,
        CandidatePreparationContext {
            horizon,
            provenance: provenance.clone(),
        },
    ) {
        CandidatePrepResult::Applicable(candidate) => *candidate,
        CandidatePrepResult::Incomplete { reason, .. } => {
            let message = match reason {
                CandidatePrepIncompleteReason::BodyEntryObservationHidden => {
                    "body-entry observation is not visible at the demanded horizon"
                }
                CandidatePrepIncompleteReason::ParameterShapeCompatibilityIncomplete => {
                    "candidate preparation is incomplete because parameter shape compatibility is not established"
                }
            };
            return Err(BuiltinPreparationFailure::Incomplete(
                Diagnostic::hard_error(message, Some(provenance)),
            ));
        }
        CandidatePrepResult::Diagnostic(diagnostic) => {
            return Err(BuildError::single(diagnostic).into());
        }
    };
    Ok(BuiltinBodyInput::new(candidate, primitive, provenance))
}

#[cfg(test)]
mod preparation_policy_tests {
    use super::*;
    use crate::{declared_policy_view, PolicyMode, Stage, SymbolId};

    #[test]
    fn changing_builtin_leaf_does_not_reconstruct_declared_policy_planes() {
        let base = crate::CompilationWorld::from_manifest(&crate::BuildManifest::new(
            "app",
            vec!["app".into()],
        ))
        .unwrap();
        let mut world = base.semantic_world().clone();
        let callable = declared_policy_view(Stage::Compile, PolicyMode::Const);
        let body = declared_policy_view(Stage::Seal, PolicyMode::Mut);
        let result = declared_policy_view(Stage::Runtime, PolicyMode::Const);
        let installed = world
            .register_core_callable(
                base.package_root_node(),
                "declared",
                SymbolId(900_003),
                BuiltinCallableImpl::IdentityType,
                None,
                crate::DeclaredResultClass::CompleteType,
                callable.clone(),
                body.clone(),
                result.clone(),
                None,
                Provenance::new("declared planes"),
            )
            .unwrap();
        let crate::SemanticValuePayload::CallEntry(entry) =
            &world.value(installed.call_entry).unwrap().payload
        else {
            panic!("call entry");
        };
        for (implementation, source) in [
            (
                BuiltinCallableImpl::IdentityType,
                "let result = uint8 declared;",
            ),
            (BuiltinCallableImpl::Struct, "let result = uint8 declared;"),
        ] {
            let mut entry = entry.clone();
            entry.implementation = OrdinaryCallableImplementation::Builtin(implementation);
            let parsed = lang_syntax::parse(source);
            assert!(parsed.diagnostics.is_empty());
            let program = lang_syntax::normalize_program(&parsed.program);
            let lang_syntax::NormForm::Let(lang_syntax::NormDecl::Let { slot, .. }) =
                &program.forms[0]
            else {
                panic!("initializer");
            };
            let site =
                crate::extract_single_call_site(slot.initializer.as_deref().unwrap()).unwrap();
            let prepared = prepare_resolved_builtin_call(
                &entry,
                crate::classify_type_arguments_env_with_report(
                    &site.to_arg_product_shape(),
                    &crate::SemanticTypeEnv::new(&world),
                    &base.package_context(),
                )
                .classified_shape,
                ObservationHorizon::SealStatic,
                Provenance::new("entry observation"),
            )
            .unwrap();
            assert_eq!(
                prepared.candidate.callee_symbol_id,
                entry.backing_declaration
            );
            assert_eq!(prepared.candidate.callee_name, entry.declaration_name);
            assert_eq!(prepared.candidate.policy_planes.callable_view, callable);
            assert_eq!(prepared.candidate.policy_planes.body_entry_policy, body);
            assert_eq!(
                prepared.candidate.policy_planes.return_object_policy,
                result
            );
        }
    }
}
