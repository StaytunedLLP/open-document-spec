

fn exit_code(diagnostics: &[Diagnostic]) -> ExitCode {
    if diagnostics
        .iter()
        .any(|d| matches!(d.severity, Severity::Error))
    {
        ExitCode::from(1)
    } else {
        ExitCode::from(0)
    }
}
