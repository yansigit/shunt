use std::fs;

fn read(path: &str) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("{path} is missing or unreadable: {error}"))
}

fn section(text: &str, heading: &str) -> String {
    let start = text
        .find(heading)
        .unwrap_or_else(|| panic!("required section {heading:?} is missing"));
    let tail = &text[start..];
    tail.find("\n## ")
        .map(|end| tail[..end].to_owned())
        .unwrap_or_else(|| tail.to_owned())
}

fn assert_zero_support_contract(text: &str, surface: &str) {
    for token in [
        "SHUNT_OPENCODE_GO_API_KEY",
        "https://opencode.ai/zen/go/v1",
        "opencode_go",
        "not supported",
        "empty admitted set",
        "x-opencode-session",
    ] {
        assert!(
            text.contains(token),
            "{surface} is missing required token {token:?}"
        );
    }
    for positive in [
        "OpenCode Go is supported",
        "OpenCode Go is live-verified",
        "supported OpenCode Go models",
    ] {
        assert!(
            !text.contains(positive),
            "{surface} contains forbidden positive claim {positive:?}"
        );
    }
}

#[test]
fn opencode_go_docs_readme() {
    let text = read("README.md");
    let section = section(&text, "OpenCode Go (evidence-gated, zero support)");
    assert_zero_support_contract(&section, "README.md OpenCode Go section");
    for candidate in [
        "glm-5.3-flash",
        "omen-alpha",
        "muse-spark-1.3-contributor",
        "deepseek-v4-flash",
        "pre-credential",
        "authoritative terminal",
    ] {
        assert!(
            section.contains(candidate),
            "README.md Go section missing {candidate:?}"
        );
    }
}

#[test]
fn opencode_go_docs_provider() {
    let text = read("site/src/content/docs/providers/opencode-go.md");
    assert_zero_support_contract(&text, "site provider page");
    for candidate in [
        "glm-5.3-flash",
        "omen-alpha",
        "muse-spark-1.3-contributor",
        "deepseek-v4-flash",
        "pre-credential",
        "authoritative terminal",
    ] {
        assert!(
            text.contains(candidate),
            "provider page missing {candidate:?}"
        );
    }
}

#[test]
fn opencode_go_docs_configuration() {
    let text = read("site/src/content/docs/reference/configuration.md");
    let section = section(&text, "### OpenCode Go (evidence-gated, zero support)");
    assert_zero_support_contract(&section, "configuration reference Go section");
    for candidate in [
        "glm-5.3-flash",
        "omen-alpha",
        "muse-spark-1.3-contributor",
        "deepseek-v4-flash",
        "pre-credential",
        "authoritative terminal",
    ] {
        assert!(
            section.contains(candidate),
            "configuration Go section missing {candidate:?}"
        );
    }
}

#[test]
fn opencode_go_docs_nav_i18n() {
    let text = read("site/src/lib/i18n.ts");
    assert!(
        text.contains("slug: \"providers/opencode-go\""),
        "i18n navigation is missing the Go provider link"
    );
    for label in [
        "OpenCode Go",
        "OpenCode Go (한국어)",
        "OpenCode Go (日本語)",
        "OpenCode Go (简体中文)",
    ] {
        assert!(
            text.contains(label),
            "i18n navigation is missing locale label {label:?}"
        );
    }
}

#[test]
fn opencode_go_docs_engineering_note() {
    let text = read("docs/opencode-go-evidence-gate.md");
    assert_zero_support_contract(&text, "engineering note");
    for decision in [
        "D-01", "D-02", "D-03", "D-04", "D-05", "D-06", "D-07", "D-08", "D-09", "D-10", "D-11",
    ] {
        assert!(
            text.contains(decision),
            "engineering note missing decision reference {decision}"
        );
    }
    assert!(
        text.contains("wiki/"),
        "engineering note must describe the generated wiki boundary"
    );
}
