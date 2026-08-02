//! multi_spec detection/resolution + Agent Skills engine coverage.
use ods_core::{
    Detected, ExtraSpecs, SkillsInitOptions, detect_workspace, init_skill_package,
    lint_skill_package, parse_extra_spec_flags, parse_skill_package, resolve_engines,
    skill_package_roots, skills_enabled,
};
use std::fs;

fn tempdir() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}

#[test]
fn resolve_engines_matrix_and_messages() {
    let det = Detected {
        ods: false,
        okf: true,
        skills: true,
    };
    let err = resolve_engines(ExtraSpecs::default(), det, true).unwrap_err();
    let msg = err.message();
    assert!(msg.contains("--okf") || msg.contains("OKF"), "{msg}");
    assert!(msg.contains("--skills") || msg.contains("Skills"), "{msg}");

    let forbidden = parse_extra_spec_flags(["lint", "--ods"]).unwrap_err();
    assert!(forbidden.message().contains("--ods"));

    let no_skills = resolve_engines(
        ExtraSpecs {
            okf: false,
            skills: true,
        },
        Detected::default(),
        true,
    )
    .unwrap_err();
    assert!(no_skills.message().contains("Skills") || no_skills.message().contains("SKILL"));

    // require_present=false allows flags without markers
    let e = resolve_engines(
        ExtraSpecs {
            okf: true,
            skills: true,
        },
        Detected::default(),
        false,
    )
    .unwrap();
    assert!(e.okf && e.skills && !e.ods);
    assert!(e.any());
}

#[test]
fn skills_init_parse_lint_roundtrip() {
    let td = tempdir();
    let pkg = td.path().join("my-cool-skill");
    fs::create_dir_all(&pkg).unwrap();

    let report = init_skill_package(
        &pkg,
        SkillsInitOptions {
            name: Some("My Cool Skill!!".into()),
        },
    )
    .unwrap();
    assert!(!report.created.is_empty());
    assert!(pkg.join("SKILL.md").is_file());
    assert!(pkg.join("scripts").is_dir());
    assert!(pkg.join("references").is_dir());
    assert!(pkg.join("assets").is_dir());

    // second init skips existing
    let report2 = init_skill_package(&pkg, SkillsInitOptions::default()).unwrap();
    assert!(!report2.skipped.is_empty());

    let parsed = parse_skill_package(&pkg).unwrap();
    assert_eq!(parsed.dir_name, "my-cool-skill");
    // sanitized name
    assert!(
        parsed
            .frontmatter
            .name
            .as_deref()
            .unwrap_or("")
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
        "{:?}",
        parsed.frontmatter.name
    );

    // Force name to match dir for clean lint
    let body = "---\nname: my-cool-skill\ndescription: A valid skill description for tests.\nlicense: Apache-2.0\n---\n\n# Skill\n";
    fs::write(pkg.join("SKILL.md"), body).unwrap();
    let parsed = parse_skill_package(&pkg).unwrap();
    let diags = lint_skill_package(&parsed);
    assert!(
        diags
            .iter()
            .all(|d| d.severity != ods_core::Severity::Error),
        "{diags:?}"
    );
}

#[test]
fn skills_lint_edge_cases() {
    let td = tempdir();
    let pkg = td.path().join("bad");
    fs::create_dir_all(&pkg).unwrap();

    // missing name/description
    fs::write(pkg.join("SKILL.md"), "---\n---\n\n# X\n").unwrap();
    let p = parse_skill_package(&pkg).unwrap();
    let d = lint_skill_package(&p);
    assert!(d.iter().any(|x| x.message.contains("name")));
    assert!(d.iter().any(|x| x.message.contains("description")));

    // too long name / invalid chars / consecutive hyphens
    fs::write(
        pkg.join("SKILL.md"),
        format!(
            "---\nname: {}\ndescription: ok desc here for length.\n---\n\n# X\n",
            "a".repeat(65)
        ),
    )
    .unwrap();
    let p = parse_skill_package(&pkg).unwrap();
    let d = lint_skill_package(&p);
    assert!(
        d.iter()
            .any(|x| x.message.contains("64") || x.message.contains("lowercase"))
    );

    fs::write(
        pkg.join("SKILL.md"),
        "---\nname: bad--name\ndescription: ok desc here for length.\n---\n\n# X\n",
    )
    .unwrap();
    let p = parse_skill_package(&pkg).unwrap();
    let d = lint_skill_package(&p);
    assert!(d.iter().any(|x| x.message.contains("hyphen")));

    // long description + compatibility + body warning
    let long_desc = "x".repeat(1025);
    let long_compat = "y".repeat(501);
    let long_body = (0..520)
        .map(|i| format!("line {i}"))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(
        pkg.join("SKILL.md"),
        format!(
            "---\nname: bad\ndescription: {long_desc}\ncompatibility: {long_compat}\n---\n\n{long_body}\n"
        ),
    )
    .unwrap();
    let p = parse_skill_package(&pkg).unwrap();
    let d = lint_skill_package(&p);
    assert!(d.iter().any(|x| x.message.contains("1024")));
    assert!(d.iter().any(|x| x.message.contains("500")));
    assert!(d.iter().any(|x| x.message.contains("500")
        || x.message.contains("progressive")
        || x.message.contains("lines")));
}

#[test]
fn skills_parse_folded_description_and_unknown_keys() {
    let td = tempdir();
    let pkg = td.path().join("demo");
    fs::create_dir_all(&pkg).unwrap();
    fs::write(
        pkg.join("SKILL.md"),
        "---\n\
         name: demo\n\
         description: >\n\
           Line one of description\n\
           Line two continues\n\
         license: 'MIT'\n\
         allowed-tools: \"Bash Read\"\n\
         metadata:\n\
           author: org\n\
           version: \"2\"\n\
         custom-key: value\n\
         ---\n\
         \n\
         # Demo\n",
    )
    .unwrap();
    let p = parse_skill_package(&pkg).unwrap();
    assert_eq!(p.frontmatter.name.as_deref(), Some("demo"));
    // Folded description / optional fields: ensure parse completes and name sticks.
    assert!(p.frontmatter.license.as_deref() == Some("MIT") || p.frontmatter.license.is_some());
    assert!(
        p.frontmatter.metadata.contains_key("author")
            || p.frontmatter.unknown.contains_key("custom-key")
            || p.frontmatter.description.is_some()
    );
}

#[test]
fn detect_hybrid_workspace_and_skill_roots() {
    let td = tempdir();
    let root = td.path();
    fs::write(
        root.join("index.md"),
        "---\nprofile: index\nods: 0.1\n---\n\n# R\n",
    )
    .unwrap();
    let sk = root.join("skills").join("demo");
    fs::create_dir_all(&sk).unwrap();
    fs::write(
        sk.join("SKILL.md"),
        "---\nname: demo\ndescription: Nested skill package for detect.\n---\n\n# Demo\n",
    )
    .unwrap();

    assert!(skills_enabled(root));
    let d = detect_workspace(root);
    assert!(d.ods && d.skills);
    let roots = skill_package_roots(root);
    assert_eq!(roots.len(), 1);

    let e = resolve_engines(
        ExtraSpecs {
            okf: false,
            skills: true,
        },
        d,
        true,
    )
    .unwrap();
    assert!(e.ods && e.skills);
}

#[test]
fn init_skill_default_name_from_dirname() {
    let td = tempdir();
    let pkg = td.path().join("pdf-extract");
    fs::create_dir_all(&pkg).unwrap();
    let report = init_skill_package(&pkg, SkillsInitOptions { name: None }).unwrap();
    assert!(report.created.iter().any(|p| p.ends_with("SKILL.md")));
    let text = fs::read_to_string(pkg.join("SKILL.md")).unwrap();
    assert!(text.contains("name: pdf-extract"), "{text}");
}
