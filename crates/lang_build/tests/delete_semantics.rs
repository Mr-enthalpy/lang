use lang_build::{selected_callable_delete_diagnostic, DiagnosticSeverity, Provenance};
use lang_syntax::{NormDeleteBody, NormOrigin, Span};

#[test]
fn selected_delete_keeps_message_and_source_provenance() {
    let span = Span::new(3, 9, 2, 4);
    let body = NormDeleteBody {
        message: Some("\"cannot combine bare if\"".into()),
        origin: NormOrigin::Source(span),
    };
    let mut provenance = Provenance::new("selected callable");
    provenance.file = Some("body.lang".into());
    let diagnostic = selected_callable_delete_diagnostic(&body, provenance);
    assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
    assert_eq!(
        diagnostic.message,
        "selected delete: cannot combine bare if"
    );
    let origin = diagnostic.provenance.unwrap();
    assert_eq!(origin.span, Some(span));
    assert_eq!(
        origin.file.as_deref(),
        Some(std::path::Path::new("body.lang"))
    );
}

#[test]
fn selected_bare_delete_rejects_without_producing_material() {
    let body = NormDeleteBody {
        message: None,
        origin: NormOrigin::Source(Span::new(0, 0, 1, 1)),
    };
    let diagnostic = selected_callable_delete_diagnostic(&body, Provenance::new("callable"));
    assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
    assert_eq!(
        diagnostic.message,
        "selected delete: selected callable is deleted"
    );
}
