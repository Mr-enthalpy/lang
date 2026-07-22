use lang_build::{
    ArgProductShape, CallableCandidateKind, CallableFrameShape, CandidatePolicyPlanes,
    CanonicalArgProductShapeMaterial, CanonicalMetaInstanceKeySeed, CoreMetaFunction, ExecutionEnv,
    FlattenedProductInvariant, FlattenedProductObject, InvocationCallableRef,
    InvocationExecutionEnv, InvocationFrame, InvocationLookupEnv, MetaInvocationInput,
    ParameterShape, PolicyEnv, PreparedCallableCandidate, ProductAtom, Provenance,
    ReturnTargetShape, SelfPosition, SelfPositionSource, SelfSlotKind, SymbolId,
};

fn empty_arg_product_shape() -> ArgProductShape {
    ArgProductShape::from_flattened(FlattenedProductObject {
        atoms: Vec::new(),
        provenance: Provenance::new("empty explicit user product"),
        invariant: FlattenedProductInvariant {
            no_direct_product_atom_remains: true,
        },
    })
}

fn unit_arg_product_shape() -> ArgProductShape {
    ArgProductShape::from_flattened(FlattenedProductObject {
        atoms: vec![ProductAtom::Unit {
            provenance: Provenance::new("explicit unit user argument"),
        }],
        provenance: Provenance::new("single explicit user product"),
        invariant: FlattenedProductInvariant {
            no_direct_product_atom_remains: true,
        },
    })
}

fn candidate_with_arg_product(arg_product_shape: ArgProductShape) -> PreparedCallableCandidate {
    let canonical_args =
        CanonicalArgProductShapeMaterial::from_arg_product_shape(&arg_product_shape);
    PreparedCallableCandidate {
        callee_symbol_id: SymbolId(42),
        callee_name: "callable".to_string(),
        callee_primitive: Some(CoreMetaFunction::IdentityType),
        callable_kind: CallableCandidateKind::MetaFunction,
        arg_product_shape,
        parameter_shape: ParameterShape::deferred(Provenance::new("test parameter shape")),
        policy_planes: CandidatePolicyPlanes {
            lookup_env: PolicyEnv::OpenStatic,
            symbol_visibility_policy: lang_build::policy_metadata(lang_build::policy_set_meta()),
            demanded_execution: ExecutionEnv::OpenStatic,
            body_entry_policy: lang_build::policy_metadata(lang_build::policy_set_meta()),
            return_object_policy: lang_build::policy_metadata(lang_build::policy_set_meta()),
        },
        canonical_key_seed: CanonicalMetaInstanceKeySeed {
            callee_function_symbol_id: SymbolId(42),
            argument_product_shape_fingerprint_fragment: None,
            unit_positions: canonical_args.unit_positions.clone(),
            argument_arity: canonical_args.arity,
            argument_type_symbols: canonical_args.known_type_symbols.clone(),
            package_identity_fragment: None,
            mount_identity_fragment: None,
            build_config_fingerprint_fragment: None,
            policy_export_fingerprint_fragment: None,
            provenance: Provenance::new("test canonical key seed"),
            argument_product_shape_material: canonical_args,
        },
        provenance: Provenance::new("prepared callable candidate"),
    }
}

#[test]
fn self_slot_exists_for_zero_user_argument_callable() {
    let frame_shape = CallableFrameShape::new_with_self_slot(
        0,
        ReturnTargetShape::ImplicitNearest,
        Provenance::new("zero-user-arg callable"),
    );

    assert_eq!(frame_shape.self_slot.slot_index, 0);
    assert_eq!(frame_shape.explicit_parameter_shape.user_parameter_count, 0);
}

#[test]
fn callable_object_self_slot_kind_is_preserved_for_unit_callable_shape() {
    let frame_shape = CallableFrameShape::new_with_self_slot_kind(
        0,
        SelfSlotKind::CallableObject,
        ReturnTargetShape::ImplicitNearest,
        Provenance::new("callable-object unit shape"),
    );

    assert_eq!(frame_shape.self_slot.slot_index, 0);
    assert_eq!(frame_shape.self_slot.kind, SelfSlotKind::CallableObject);
    assert_eq!(frame_shape.explicit_parameter_shape.user_parameter_count, 0);
}

#[test]
fn self_is_not_counted_in_explicit_argument_product() {
    let explicit_user_product = unit_arg_product_shape();
    let original_user_arity = explicit_user_product.arity;

    let frame = InvocationFrame::new(
        InvocationCallableRef::Symbol(SymbolId(7)),
        SelfPosition::placeholder_from_callable_symbol(
            SymbolId(7),
            Provenance::new("resolved callable self"),
        ),
        explicit_user_product,
        InvocationLookupEnv::new(PolicyEnv::OpenStatic),
        InvocationExecutionEnv::new(ExecutionEnv::OpenStatic),
        Provenance::new("invocation frame"),
    )
    .expect("valid invocation frame");

    assert_eq!(frame.explicit_arg_product.arity, original_user_arity);
    assert_eq!(frame.explicit_arg_product.arity, 1);
    assert_eq!(frame.self_position.slot_index, 0);
}

#[test]
fn declaration_context_call_entry_placeholder_uses_same_frame_model() {
    // This is a frame-substrate test only. It models the empty explicit
    // product of a declaration-context `()` call entry such as
    // `let ()::ref::T = (self: T ref) => { ... }` without claiming that
    // source-level call-entry injection is implemented in this PR.
    let frame = InvocationFrame::new(
        InvocationCallableRef::Placeholder,
        SelfPosition::placeholder_from_call_entry(Provenance::new("call-entry self")),
        empty_arg_product_shape(),
        InvocationLookupEnv::new(PolicyEnv::OpenStatic),
        InvocationExecutionEnv::new(ExecutionEnv::OpenStatic),
        Provenance::new("unit callable placeholder frame"),
    )
    .expect("valid placeholder invocation frame");

    assert_eq!(frame.self_position.slot_index, 0);
    assert_eq!(
        frame.self_position.source,
        SelfPositionSource::PlaceholderFromCallEntry
    );
    assert_eq!(frame.explicit_arg_product.arity, 0);
}

#[test]
fn meta_invocation_helper_preserves_candidate_arg_shape() {
    let candidate = candidate_with_arg_product(unit_arg_product_shape());
    let input = MetaInvocationInput::new(candidate.clone(), Provenance::new("meta input"));

    let frame = input
        .placeholder_invocation_frame()
        .expect("placeholder invocation frame");

    assert_eq!(frame.explicit_arg_product, candidate.arg_product_shape);
    assert_eq!(frame.self_position.slot_index, 0);
    assert_eq!(
        frame.self_position.source,
        SelfPositionSource::PlaceholderFromCallableSymbol(candidate.callee_symbol_id)
    );
}

#[test]
fn invocation_frame_rejects_nonzero_self_position() {
    let result = InvocationFrame::new(
        InvocationCallableRef::Symbol(SymbolId(9)),
        SelfPosition {
            slot_index: 1,
            source: SelfPositionSource::PlaceholderFromCallableSymbol(SymbolId(9)),
            provenance: Provenance::new("invalid self position"),
        },
        empty_arg_product_shape(),
        InvocationLookupEnv::new(PolicyEnv::OpenStatic),
        InvocationExecutionEnv::new(ExecutionEnv::OpenStatic),
        Provenance::new("invalid invocation frame"),
    );

    let diagnostic = result.expect_err("nonzero self slot must be rejected");
    assert!(diagnostic.message.contains("slot 0"));
}

#[test]
fn invocation_frame_rejects_arg_shape_arity_atom_mismatch() {
    let mut mismatched_product = empty_arg_product_shape();
    mismatched_product.arity = 1;

    let result = InvocationFrame::new(
        InvocationCallableRef::Symbol(SymbolId(10)),
        SelfPosition::placeholder_from_callable_symbol(
            SymbolId(10),
            Provenance::new("resolved callable self"),
        ),
        mismatched_product,
        InvocationLookupEnv::new(PolicyEnv::OpenStatic),
        InvocationExecutionEnv::new(ExecutionEnv::OpenStatic),
        Provenance::new("mismatched invocation frame"),
    );

    let diagnostic = result.expect_err("arity/atom mismatch must be rejected");
    assert!(diagnostic.message.contains("arity"));
}
