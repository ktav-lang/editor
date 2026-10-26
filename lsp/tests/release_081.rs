use ktav::{Error, ErrorKind};
use ktav_lsp::diagnostics::parse_for_diagnostics;
use tower_lsp::lsp_types::{DiagnosticSeverity, Position, Range};

#[test]
fn explicit_root_eof_has_a_diagnostic_from_opener_to_eof() {
    for (source, end) in [
        ("{", Position::new(0, 1)),
        ("[", Position::new(0, 1)),
        ("{\n", Position::new(1, 0)),
        ("[\r\n", Position::new(1, 0)),
        ("{\ra: 1\r", Position::new(2, 0)),
        ("{\ninner: [1]\n", Position::new(2, 0)),
    ] {
        let error = ktav::parse(source).expect_err("незакрытый корень");
        assert!(
            matches!(
                &error,
                Error::Structured(ErrorKind::UnclosedCompound { .. })
            ),
            "{source:?}: {error:?}"
        );
        let diagnostics = parse_for_diagnostics(source);
        assert_eq!(diagnostics.len(), 1, "{source:?}");
        assert_eq!(diagnostics[0].severity, Some(DiagnosticSeverity::ERROR));
        assert_eq!(
            diagnostics[0].range,
            Range::new(Position::new(0, 0), end),
            "{source:?}"
        );
    }
    for source in ["{\n}\n", "[\r\n]\r\n", "{\ra: 1\r}\r"] {
        assert!(parse_for_diagnostics(source).is_empty(), "{source:?}");
    }
}
