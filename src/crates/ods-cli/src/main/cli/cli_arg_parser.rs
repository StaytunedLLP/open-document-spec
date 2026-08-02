#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ServeMode {
    Auto,
    Watch,
    Poll,
}

#[derive(Clone, Debug)]
struct ServeOptions {
    root: PathBuf,
    mode: ServeMode,
    memory_report: bool,
    poll_secs: u64,
}

fn serve_options_from_args(args: &[String]) -> Result<ServeOptions, CliError> {
    let mut root = None;
    let mut mode = env::var("ODS_SERVE_MODE")
        .ok()
        .map(|value| parse_serve_mode(&value))
        .transpose()?
        .unwrap_or(ServeMode::Auto);
    let mut memory_report = false;
    let mut poll_secs = env::var("ODS_POLL_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10);
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--root" => {
                let v = args
                    .get(i + 1)
                    .ok_or_else(|| usage("serve --root requires a path"))?;
                root = Some(PathBuf::from(v));
                i += 2;
            }
            "--mode" => {
                let v = args
                    .get(i + 1)
                    .ok_or_else(|| usage("serve --mode requires auto, watch, or poll"))?;
                mode = parse_serve_mode(v)?;
                i += 2;
            }
            "--memory-report" => {
                memory_report = true;
                i += 1;
            }
            "--poll-secs" => {
                let v = args
                    .get(i + 1)
                    .ok_or_else(|| usage("serve --poll-secs requires seconds"))?;
                poll_secs = v
                    .parse()
                    .map_err(|_| usage("serve --poll-secs requires a positive integer"))?;
                i += 2;
            }
            other if !other.starts_with('-') => {
                root = Some(PathBuf::from(other));
                i += 1;
            }
            _ => i += 1,
        }
    }
    let path = root.unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    Ok(ServeOptions {
        root: resolve_root_path(path),
        mode,
        memory_report,
        poll_secs: poll_secs.max(1),
    })
}

fn parse_serve_mode(value: &str) -> Result<ServeMode, CliError> {
    match value {
        "auto" => Ok(ServeMode::Auto),
        "watch" => Ok(ServeMode::Watch),
        "poll" => Ok(ServeMode::Poll),
        other => Err(usage(format!(
            "invalid serve --mode {other} (use auto, watch, or poll)"
        ))),
    }
}

fn resolved_serve_mode(mode: ServeMode) -> ServeMode {
    match mode {
        ServeMode::Auto if env::var("ODS_LOW_MEMORY").ok().as_deref() == Some("1") => {
            ServeMode::Poll
        }
        ServeMode::Auto => ServeMode::Watch,
        other => other,
    }
}

fn parse_export_args(args: &[String]) -> Result<(PathBuf, PathBuf, OutputFormat, String), CliError> {
    let mut out = None;
    let mut path = None;
    let mut format = OutputFormat::Text;
    let mut spec = "ods:0.1".to_string();

    let mut i = 2;
    // Skip optional "graph" subcommand token if present (e.g. ods export graph)
    if i < args.len() && (args[i] == "graph" || args[i] == "all") {
        i += 1;
    }

    while i < args.len() {
        match args[i].as_str() {
            "--out" => {
                let v = args
                    .get(i + 1)
                    .ok_or_else(|| usage("export --out requires a path"))?;
                out = Some(PathBuf::from(v));
                i += 2;
            }
            other if other.starts_with("--out=") => {
                out = Some(PathBuf::from(&other["--out=".len()..]));
                i += 1;
            }
            "--format" => {
                let v = args
                    .get(i + 1)
                    .ok_or_else(|| usage("export --format requires text, json, or md"))?;
                format = match v.as_str() {
                    "text" => OutputFormat::Text,
                    "json" => OutputFormat::Json,
                    "md" | "markdown" => OutputFormat::Text, // md triggers markdown file or text output
                    other => return Err(usage(format!("invalid export --format {other} (use text, json, or md)"))),
                };
                i += 2;
            }
            "--spec" => {
                let v = args
                    .get(i + 1)
                    .ok_or_else(|| usage("export --spec requires ods or okf"))?;
                spec = match v.to_lowercase().as_str() {
                    "okf" | "okf:0.2" => "okf:0.2".to_string(),
                    _ => "ods:0.1".to_string(),
                };
                i += 2;
            }
            "--okf" => {
                spec = "okf:0.2".to_string();
                i += 1;
            }
            other if !other.starts_with('-') => {
                path = Some(PathBuf::from(other));
                i += 1;
            }
            _ => i += 1,
        }
    }
    let root = resolve_root_path(
        path.unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from("."))),
    );
    let out = out.unwrap_or_else(|| root.join("graph.md"));
    Ok((root, out, format, spec))
}

/// Parsed `ods share` arguments: `(workspace root, scope, out, include_org, include_private)`.
///
/// `scope` defaults to the discovered workspace root when `[path]` is omitted;
/// when given, it limits which documents are published without changing
/// where the workspace is loaded from (ancestor `share` cascades above
/// `scope` still apply).
fn parse_share_args(
    args: &[String],
) -> Result<(PathBuf, PathBuf, PathBuf, bool, bool), CliError> {
    let mut out = None;
    let mut path = None;
    let mut include_org = false;
    let mut include_private = false;
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--out" => {
                let v = args
                    .get(i + 1)
                    .ok_or_else(|| usage("share --out requires a path"))?;
                out = Some(PathBuf::from(v));
                i += 2;
            }
            other if other.starts_with("--out=") => {
                out = Some(PathBuf::from(&other["--out=".len()..]));
                i += 1;
            }
            "--include-org" => {
                include_org = true;
                i += 1;
            }
            "--include-private" => {
                include_private = true;
                i += 1;
            }
            other if !other.starts_with('-') => {
                path = Some(PathBuf::from(other));
                i += 1;
            }
            _ => i += 1,
        }
    }
    let root = resolve_root_path(
        path.clone()
            .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from("."))),
    );
    let scope = path.unwrap_or_else(|| root.clone());
    let out = out.ok_or_else(|| usage("share --out <dir> is required"))?;
    Ok((root, scope, out, include_org, include_private))
}

#[derive(Clone, Copy)]
enum OutputFormat {
    Text,
    Json,
    Sarif,
}

fn parse_common_flags(
    args: &[String],
    start: usize,
) -> Result<(PathBuf, LintLevel, OutputFormat), CliError> {
    let mut level = LintLevel::Level3;
    let mut format = OutputFormat::Text;
    let mut path = None;

    let mut i = start;
    while i < args.len() {
        match args[i].as_str() {
            "--mode" | "--level" => {
                let value = args
                    .get(i + 1)
                    .ok_or_else(|| usage("missing value for --mode / --level"))?;
                level = match value.to_lowercase().as_str() {
                    "standard" | "1" => LintLevel::Standard,
                    "strict" | "3" => LintLevel::Strict,
                    other => return Err(usage(format!("invalid compliance mode '{other}' (use standard or strict)"))),
                };
                i += 2;
            }
            "--format" => {
                let value = args
                    .get(i + 1)
                    .ok_or_else(|| usage("missing value for --format"))?;
                format = match value.as_str() {
                    "text" => OutputFormat::Text,
                    "json" => OutputFormat::Json,
                    "sarif" => OutputFormat::Sarif,
                    other => {
                        return Err(usage(format!(
                            "invalid --format {other} (use text, json, or sarif)"
                        )));
                    }
                };
                i += 2;
            }
            "--check"
            | "--write"
            | "--write-report"
            | "--all"
            | "--adopt"
            | "--status"
            | "--canonical-refs"
            | "--include-private"
            | "--keep-frontmatter"
            | "--remove-indexes"
            | "--remove-root-index"
            | "--full"
            | "--indexes"
            | "--strip-indexes"
            | "--profiles"
            | "--strip-profiles"
            | "--migrate" => {
                i += 1;
            }
            "--refs" => {
                i += 2;
            }
            "--tag" | "--prompt" | "--llm" | "--snapshot" | "--path" => {
                // value consumed by find/bench; skip so path parsing still works
                i += 2;
            }
            flag if flag.starts_with('-') => {
                return Err(usage(format!("unknown flag: {flag}")));
            }
            other => {
                if path.is_none() {
                    path = Some(PathBuf::from(other));
                }
                i += 1;
            }
        }
    }

    Ok((
        resolve_root_path(
            path.unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from("."))),
        ),
        level,
        format,
    ))
}

/// Positional args after `start`, skipping flags and their values.
fn positional_args(args: &[String], start: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut i = start;
    while i < args.len() {
        match args[i].as_str() {
            "--level" | "--format" | "--version" => i += 2,
            "--check" | "--write" | "--force" | "--canonical-refs" | "--include-private" => i += 1,
            "--refs" => i += 2,
            flag if flag.starts_with('-') => i += 1,
            other => {
                out.push(other.to_string());
                i += 1;
            }
        }
    }
    out
}
