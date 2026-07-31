use lang_build::{
    CallableOwnerPlacement, CallableReceiverBindingSource, CallableReceiverTypeId,
    CanonicalValueAddr, ExtractionMemberVisibility, LocalCallableIdentity, LocalGenerationIdentity,
    LocalSymbolIdentity, MetaCallableIdentity, MetaInstanceKey, NamespaceLookupFailure,
    NamespaceNameView, NamespaceSymbolEntry, NamespaceVisibility, OwnerNamespaceGraph,
    OwnerNamespaceNodeId, OwnerQualificationError, PackageId, Provenance, SemanticOwnerGraph,
    SemanticOwnerQualification, SemanticSymbolIdentity, SemanticValueId,
};
use lang_syntax::{NormDecl, NormForm};

fn entry(
    symbol: u64,
    declaration_owner: lang_build::SemanticOwnerId,
    visibility: NamespaceVisibility,
    retained: bool,
    external: bool,
    extraction: ExtractionMemberVisibility,
) -> NamespaceSymbolEntry {
    NamespaceSymbolEntry {
        identity: SemanticSymbolIdentity {
            owner: declaration_owner,
            local: LocalSymbolIdentity(symbol),
        },
        declaration_owner,
        namespace_visibility: visibility,
        in_export_retention_closure: retained,
        has_external_candidate_view: external,
        extraction_visibility: extraction,
    }
}

fn canonical_meta_key(callable: MetaCallableIdentity, argument_addr: u64) -> MetaInstanceKey {
    MetaInstanceKey {
        callable,
        arguments: CanonicalValueAddr(argument_addr),
        provenance: Provenance::new(format!("canonical args@{argument_addr}")),
    }
}

#[test]
fn every_callable_has_parent_linked_owner_and_standalone_anonymous_type_without_inner_namespace() {
    let mut owners = SemanticOwnerGraph::new();
    let package = owners.package_root(PackageId(1), "app");
    let namespace = owners.namespace(package, "main");
    assert_eq!(
        namespace,
        owners.namespace(package, "main"),
        "the same parent/name constructor reuses one namespace owner"
    );
    let outer = owners.callable(
        namespace,
        LocalCallableIdentity(0),
        CallableOwnerPlacement::Ordinary,
    );
    let in_place = owners.callable(
        outer,
        LocalCallableIdentity(1),
        CallableOwnerPlacement::InPlace,
    );
    let deeper = owners.callable(
        in_place,
        LocalCallableIdentity(2),
        CallableOwnerPlacement::InPlace,
    );

    assert_eq!(owners.parent(in_place), Some(outer));
    assert_eq!(owners.parent(deeper), Some(in_place));
    assert_eq!(
        owners
            .anonymous_callable_type(in_place)
            .unwrap()
            .callable_owner,
        in_place,
        "in-place closures have an anonymous type available for standalone materialization"
    );
    assert_eq!(
        owners.printable_self_path(deeper),
        Some(vec!["Self", "Self", "Self"])
    );
    assert_eq!(
        owners
            .callable_owner_path(deeper)
            .unwrap()
            .into_iter()
            .collect::<Vec<_>>(),
        vec![deeper, in_place, outer],
        "source navigation prints the innermost Self first and outermost Self last"
    );
    assert_ne!(
        SemanticSymbolIdentity {
            owner: outer,
            local: LocalSymbolIdentity(9),
        },
        SemanticSymbolIdentity {
            owner: in_place,
            local: LocalSymbolIdentity(9),
        },
        "the same local spelling/key under distinct callable owners has distinct identity"
    );
    assert_ne!(
        owners.generated(outer, LocalGenerationIdentity(0)),
        owners.generated(in_place, LocalGenerationIdentity(0)),
        "generated helper identity is qualified by its generating owner"
    );
    assert_eq!(
        owners.generated(outer, LocalGenerationIdentity(0)),
        owners.generated(outer, LocalGenerationIdentity(0)),
        "the same semantic constructor key reuses a generated owner"
    );
    assert_eq!(
        owners.callable(
            outer,
            LocalCallableIdentity(1),
            CallableOwnerPlacement::InPlace,
        ),
        in_place,
        "the same parent/local callable key reuses one callable owner"
    );
}

#[test]
fn callable_owner_is_independent_from_associated_call_receiver_type() {
    let mut owners = SemanticOwnerGraph::new();
    let package = owners.package_root(PackageId(1), "app");
    let namespace = owners.namespace(package, "main");
    let callable = owners.callable(
        namespace,
        LocalCallableIdentity(0),
        CallableOwnerPlacement::Ordinary,
    );

    let standalone = owners
        .standalone_receiver_binding(callable)
        .expect("standalone receiver binding");
    assert_eq!(standalone.callable_owner, callable);
    assert_eq!(
        standalone.source,
        CallableReceiverBindingSource::StandaloneAnonymousDefault
    );
    assert!(matches!(
        standalone.receiver_type,
        CallableReceiverTypeId::Anonymous(_)
    ));

    let ref_t = SemanticSymbolIdentity {
        owner: namespace,
        local: LocalSymbolIdentity(77),
    };
    let associated = owners
        .associated_call_entry_receiver_binding(callable, ref_t)
        .expect("associated call-entry receiver");
    assert_eq!(associated.callable_owner, callable);
    assert_eq!(
        associated.source,
        CallableReceiverBindingSource::AssociatedCallEntry
    );
    assert_eq!(
        associated.receiver_type,
        CallableReceiverTypeId::Named(ref_t),
        "the same callable body owner may receive a named caller type rather than its anonymous standalone type"
    );
}

#[test]
fn semantic_owner_identity_is_qualified_by_its_graph() {
    let mut left = SemanticOwnerGraph::new();
    let mut right = SemanticOwnerGraph::new();
    assert_ne!(
        left.package_root(PackageId(1), "same"),
        right.package_root(PackageId(1), "same"),
        "local owner ordinal zero from two graphs must not compare equal"
    );
}

#[test]
fn frontend_pattern_root_identity_is_qualified_at_the_build_owner_boundary() {
    let parsed = lang_syntax::parse("let <A> x = value;");
    let normalized = lang_syntax::normalize_program(&parsed.program);
    let [NormForm::Let(NormDecl::Let { slot, .. })] = normalized.forms.as_slice() else {
        panic!("expected one normalized let");
    };
    let frontend = slot.deduce[0].id;
    let frontend_owner = frontend.pattern_root().owner;

    let mut owners = SemanticOwnerGraph::new();
    let semantic_owner = owners.package_root(PackageId(1), "app");
    let mut qualification = SemanticOwnerQualification::default();
    assert_eq!(
        qualification.qualify_hole(frontend),
        Err(OwnerQualificationError::UnmappedFrontendOwner(
            frontend_owner
        )),
        "a local hole cannot enter build-world identity before its exact owner is mapped"
    );
    qualification.bind(frontend_owner, semantic_owner).unwrap();
    let resolved = qualification.qualify_hole(frontend).unwrap();
    assert_eq!(resolved.root.owner, semantic_owner);
    assert_eq!(resolved.root.local_root, frontend.pattern_root().local_root);
    assert_eq!(resolved.local_binder, frontend.local_ordinal());

    let conflicting_owner = owners.namespace(semantic_owner, "wrong");
    assert!(matches!(
        qualification.bind(frontend_owner, conflicting_owner),
        Err(OwnerQualificationError::ConflictingMapping { .. })
    ));
}

#[test]
fn canonical_meta_invocations_share_the_callable_owner_graph_and_are_interned() {
    let mut owners = SemanticOwnerGraph::new();
    let package = owners.package_root(PackageId(1), "app");
    let namespace = owners.namespace(package, "meta");
    // Meta instance interning keys off the selected function object VALUE
    // identity, never the carrier Symbol hosting the overload cluster.
    let f = MetaCallableIdentity {
        selected_function_value: SemanticValueId(7),
        selected_call_entry: SemanticValueId(70),
    };
    let uint8 = canonical_meta_key(f, 8);
    let uint16 = canonical_meta_key(f, 16);

    let f_uint8 = owners.meta_instance(namespace, f, uint8.clone());
    assert_eq!(
        f_uint8,
        owners.meta_instance(namespace, f, uint8),
        "the same canonical invocation reuses one semantic owner"
    );
    assert_ne!(
        f_uint8,
        owners.meta_instance(namespace, f, uint16),
        "different canonical arguments create distinct owners"
    );

    let returned_meta = MetaCallableIdentity {
        selected_function_value: SemanticValueId(100),
        selected_call_entry: SemanticValueId(101),
    };
    let nested = owners.meta_instance(f_uint8, returned_meta, canonical_meta_key(returned_meta, 8));
    assert_eq!(owners.parent(nested), Some(f_uint8));
}

#[test]
fn non_export_symbols_are_visible_to_same_package_descendants_but_not_siblings() {
    let mut owners = SemanticOwnerGraph::new();
    let package_id = PackageId(1);
    let package = owners.package_root(package_id, "app");
    let namespace_owner = owners.namespace(package, "n");
    let descendant = owners.callable(
        namespace_owner,
        LocalCallableIdentity(0),
        CallableOwnerPlacement::Ordinary,
    );
    let sibling = owners.namespace(package, "sibling");

    let mut namespaces = OwnerNamespaceGraph::new();
    let root = namespaces.add_node(
        package,
        None,
        "app",
        Some(package_id),
        NamespaceVisibility::Public,
    );
    namespaces.add_symbol(
        root,
        "helper",
        entry(
            10,
            namespace_owner,
            NamespaceVisibility::Public,
            false,
            false,
            ExtractionMemberVisibility::Public,
        ),
    );
    namespaces.add_symbol(
        root,
        "helper",
        entry(
            11,
            namespace_owner,
            NamespaceVisibility::Public,
            false,
            false,
            ExtractionMemberVisibility::Public,
        ),
    );

    let found = namespaces
        .resolve_lexical_symbol(&owners, descendant, root, "helper")
        .expect("descendant owner sees ancestor non-export symbol");
    assert_eq!(found.view, NamespaceNameView::FullNameView);
    assert_eq!(found.candidate_identities[0].local, LocalSymbolIdentity(10));
    assert_eq!(
        found.candidate_identities.len(),
        2,
        "name resolution exposes the complete overload set and does not select a candidate"
    );
    assert_eq!(
        namespaces.resolve_lexical_symbol(&owners, sibling, root, "helper"),
        Err(NamespaceLookupFailure::Unresolved),
        "same package alone does not grant sibling lexical visibility"
    );
    assert!(
        namespaces
            .resolve_outer_to_inner(
                &owners,
                sibling,
                root,
                &["helper".to_string()],
            )
            .is_ok(),
        "explicit same-package navigation consumes FullNameView independently of lexical inheritance"
    );
}

#[test]
fn mount_crossing_switches_to_external_view_and_preserves_target_identity() {
    let mut owners = SemanticOwnerGraph::new();
    let app_package_id = PackageId(1);
    let dep_package_id = PackageId(2);
    let app = owners.package_root(app_package_id, "app");
    let dep = owners.package_root(dep_package_id, "dep");
    let app_query = owners.callable(
        app,
        LocalCallableIdentity(0),
        CallableOwnerPlacement::Ordinary,
    );
    let dep_query = owners.callable(
        dep,
        LocalCallableIdentity(0),
        CallableOwnerPlacement::Ordinary,
    );

    let mut namespaces = OwnerNamespaceGraph::new();
    let app_root = namespaces.add_node(
        app,
        None,
        "app",
        Some(app_package_id),
        NamespaceVisibility::Public,
    );
    let dep_root = namespaces.add_node(
        dep,
        None,
        "dep",
        Some(dep_package_id),
        NamespaceVisibility::Public,
    );
    namespaces.add_symbol(
        dep_root,
        "foo",
        entry(
            77,
            dep,
            NamespaceVisibility::Public,
            true,
            true,
            ExtractionMemberVisibility::Public,
        ),
    );
    namespaces.add_symbol(
        dep_root,
        "foo",
        entry(
            78,
            dep,
            NamespaceVisibility::Public,
            true,
            false,
            ExtractionMemberVisibility::Public,
        ),
    );
    namespaces.add_mount(
        app,
        app_root,
        "vendor_dep",
        dep_root,
        NamespaceVisibility::Public,
    );

    let direct = namespaces
        .resolve_outer_to_inner(&owners, dep_query, dep_root, &["foo".to_string()])
        .unwrap();
    let mounted = namespaces
        .resolve_outer_to_inner(
            &owners,
            app_query,
            app_root,
            &["vendor_dep".to_string(), "foo".to_string()],
        )
        .unwrap();
    assert_eq!(direct.candidate_identities.len(), 2);
    assert_eq!(mounted.candidate_identities.len(), 1);
    assert!(
        direct
            .candidate_identities
            .contains(&mounted.candidate_identities[0]),
        "mount/external projection preserves the target candidate identity rather than copying it"
    );
    assert_eq!(direct.view, NamespaceNameView::FullNameView);
    assert_eq!(mounted.view, NamespaceNameView::ExternalNameView);
    assert!(mounted.crossed_package_boundary);
}

#[test]
fn conflicting_package_names_remain_distinct_behind_separate_mount_paths() {
    let mut owners = SemanticOwnerGraph::new();
    let app = owners.package_root(PackageId(1), "app");
    let package_a = owners.package_root(PackageId(2), "package-a");
    let package_b = owners.package_root(PackageId(3), "package-b");
    let query = owners.callable(
        app,
        LocalCallableIdentity(0),
        CallableOwnerPlacement::Ordinary,
    );

    let mut graph = OwnerNamespaceGraph::new();
    let app_root = graph.add_node(
        app,
        None,
        "app",
        Some(PackageId(1)),
        NamespaceVisibility::Public,
    );
    let vendor = graph.add_node(
        app,
        Some(app_root),
        "vendor",
        None,
        NamespaceVisibility::Public,
    );
    let root_a = graph.add_node(
        package_a,
        None,
        "package-a",
        Some(PackageId(2)),
        NamespaceVisibility::Public,
    );
    let root_b = graph.add_node(
        package_b,
        None,
        "package-b",
        Some(PackageId(3)),
        NamespaceVisibility::Public,
    );
    graph.add_symbol(
        root_a,
        "foo",
        entry(
            1,
            package_a,
            NamespaceVisibility::Public,
            true,
            true,
            ExtractionMemberVisibility::Default,
        ),
    );
    graph.add_symbol(
        root_b,
        "foo",
        entry(
            1,
            package_b,
            NamespaceVisibility::Public,
            true,
            true,
            ExtractionMemberVisibility::Default,
        ),
    );
    graph.add_mount(app, vendor, "A", root_a, NamespaceVisibility::Public);
    graph.add_mount(app, vendor, "B", root_b, NamespaceVisibility::Public);

    let mounted_a = graph
        .resolve_inner_to_outer(
            &owners,
            query,
            app_root,
            &["foo".into(), "A".into(), "vendor".into()],
        )
        .unwrap();
    let mounted_b = graph
        .resolve_inner_to_outer(
            &owners,
            query,
            app_root,
            &["foo".into(), "B".into(), "vendor".into()],
        )
        .unwrap();
    assert_ne!(
        mounted_a.candidate_identities,
        mounted_b.candidate_identities
    );
    assert_eq!(mounted_a.view, NamespaceNameView::ExternalNameView);
    assert_eq!(mounted_b.view, NamespaceNameView::ExternalNameView);
}

#[test]
fn external_resolution_preserves_private_and_export_failure_reasons() {
    let mut owners = SemanticOwnerGraph::new();
    let app = owners.package_root(PackageId(1), "app");
    let dep = owners.package_root(PackageId(2), "dep");
    let query = owners.callable(
        app,
        LocalCallableIdentity(0),
        CallableOwnerPlacement::Ordinary,
    );
    let mut graph = OwnerNamespaceGraph::new();
    let app_root = graph.add_node(
        app,
        None,
        "app",
        Some(PackageId(1)),
        NamespaceVisibility::Public,
    );
    let dep_root = graph.add_node(
        dep,
        None,
        "dep",
        Some(PackageId(2)),
        NamespaceVisibility::Public,
    );
    graph.add_symbol(
        dep_root,
        "private_name",
        entry(
            1,
            dep,
            NamespaceVisibility::Private,
            true,
            true,
            ExtractionMemberVisibility::Private,
        ),
    );
    let private_child_owner = owners.namespace(dep, "private_child");
    let private_child = graph.add_node(
        private_child_owner,
        Some(dep_root),
        "private_child",
        None,
        NamespaceVisibility::Private,
    );
    graph.add_symbol(
        private_child,
        "public_grandchild",
        entry(
            4,
            private_child_owner,
            NamespaceVisibility::Public,
            true,
            true,
            ExtractionMemberVisibility::Public,
        ),
    );
    graph.add_symbol(
        dep_root,
        "not_retained",
        entry(
            2,
            dep,
            NamespaceVisibility::Public,
            false,
            false,
            ExtractionMemberVisibility::Public,
        ),
    );
    graph.add_symbol(
        dep_root,
        "no_candidate",
        entry(
            3,
            dep,
            NamespaceVisibility::Public,
            true,
            false,
            ExtractionMemberVisibility::Public,
        ),
    );
    graph.add_mount(app, app_root, "dep", dep_root, NamespaceVisibility::Public);

    let resolve = |name: &str| {
        graph.resolve_outer_to_inner(
            &owners,
            query,
            app_root,
            &["dep".to_string(), name.to_string()],
        )
    };
    assert_eq!(
        resolve("private_name"),
        Err(NamespaceLookupFailure::PrivatePath)
    );
    assert_eq!(
        resolve("not_retained"),
        Err(NamespaceLookupFailure::NotInExportRetentionDomain)
    );
    assert_eq!(
        resolve("no_candidate"),
        Err(NamespaceLookupFailure::NoExternallyEligibleCandidate)
    );
    assert_eq!(
        graph.resolve_outer_to_inner(
            &owners,
            query,
            app_root,
            &[
                "dep".to_string(),
                "private_child".to_string(),
                "public_grandchild".to_string(),
            ],
        ),
        Err(NamespaceLookupFailure::PrivatePath),
        "a public descendant remains blocked behind a private path component"
    );
}

#[test]
fn default_extraction_view_excludes_private_members_without_deleting_full_entries() {
    let mut owners = SemanticOwnerGraph::new();
    let package = owners.package_root(PackageId(1), "app");
    let mut graph = OwnerNamespaceGraph::new();
    let node = graph.add_node(
        package,
        None,
        "S",
        Some(PackageId(1)),
        NamespaceVisibility::Public,
    );
    graph.add_symbol(
        node,
        "visible",
        entry(
            1,
            package,
            NamespaceVisibility::Public,
            true,
            true,
            ExtractionMemberVisibility::Public,
        ),
    );
    graph.add_symbol(
        node,
        "default_visible",
        entry(
            3,
            package,
            NamespaceVisibility::Public,
            true,
            true,
            ExtractionMemberVisibility::Default,
        ),
    );
    graph.add_symbol(
        node,
        "secret",
        entry(
            2,
            package,
            NamespaceVisibility::Private,
            true,
            false,
            ExtractionMemberVisibility::Private,
        ),
    );

    assert_eq!(
        graph.node(node).unwrap().symbols.len(),
        3,
        "FullNameView retains both structural members"
    );
    let extraction = graph.default_extraction_view(node);
    assert_eq!(
        extraction.get("visible").unwrap()[0].local,
        LocalSymbolIdentity(1)
    );
    assert!(!extraction.contains_key("secret"));
    assert!(extraction.contains_key("default_visible"));
}

#[test]
fn missing_mount_target_is_a_typed_failure() {
    let mut owners = SemanticOwnerGraph::new();
    let app = owners.package_root(PackageId(1), "app");
    let query = owners.callable(
        app,
        LocalCallableIdentity(0),
        CallableOwnerPlacement::Ordinary,
    );
    let mut graph = OwnerNamespaceGraph::new();
    let root = graph.add_node(
        app,
        None,
        "app",
        Some(PackageId(1)),
        NamespaceVisibility::Public,
    );
    graph.add_mount(
        app,
        root,
        "missing",
        OwnerNamespaceNodeId(u64::MAX),
        NamespaceVisibility::Public,
    );
    assert_eq!(
        graph.resolve_outer_to_inner(
            &owners,
            query,
            root,
            &["missing".to_string(), "x".to_string()],
        ),
        Err(NamespaceLookupFailure::MountTargetMissing)
    );
}

#[test]
fn nearest_package_boundary_controls_namespace_domain() {
    let mut owners = SemanticOwnerGraph::new();
    let outer_owner = owners.package_root(PackageId(1), "outer");
    let nested_owner = owners.package_root(PackageId(2), "nested");
    let mut graph = OwnerNamespaceGraph::new();
    let outer = graph.add_node(
        outer_owner,
        None,
        "outer",
        Some(PackageId(1)),
        NamespaceVisibility::Public,
    );
    let nested = graph.add_node(
        nested_owner,
        Some(outer),
        "nested",
        Some(PackageId(2)),
        NamespaceVisibility::Public,
    );
    let inherited = graph.add_node(
        nested_owner,
        Some(nested),
        "child",
        None,
        NamespaceVisibility::Public,
    );
    assert_eq!(graph.package_of(outer), Some(PackageId(1)));
    assert_eq!(graph.package_of(nested), Some(PackageId(2)));
    assert_eq!(graph.package_of(inherited), Some(PackageId(2)));
}

#[test]
fn missing_package_boundary_is_a_typed_failure() {
    let mut owners = SemanticOwnerGraph::new();
    let app = owners.package_root(PackageId(1), "app");
    let query = owners.callable(
        app,
        LocalCallableIdentity(0),
        CallableOwnerPlacement::Ordinary,
    );
    let mut graph = OwnerNamespaceGraph::new();
    let root = graph.add_node(app, None, "unqualified", None, NamespaceVisibility::Public);
    assert_eq!(
        graph.resolve_inner_to_outer(&owners, query, root, &["x".to_string()]),
        Err(NamespaceLookupFailure::PackageBoundaryViolation)
    );
}

#[test]
fn namespace_boundary_must_agree_with_its_semantic_owner_package() {
    let mut owners = SemanticOwnerGraph::new();
    let app = owners.package_root(PackageId(1), "app");
    let query = owners.callable(
        app,
        LocalCallableIdentity(0),
        CallableOwnerPlacement::Ordinary,
    );
    let mut graph = OwnerNamespaceGraph::new();
    let inconsistent = graph.add_node(
        app,
        None,
        "wrong-package",
        Some(PackageId(2)),
        NamespaceVisibility::Public,
    );
    graph.add_symbol(
        inconsistent,
        "x",
        entry(
            1,
            app,
            NamespaceVisibility::Public,
            true,
            true,
            ExtractionMemberVisibility::Default,
        ),
    );
    assert_eq!(
        graph.resolve_inner_to_outer(&owners, query, inconsistent, &["x".to_string()]),
        Err(NamespaceLookupFailure::PackageBoundaryViolation)
    );
}
