use std::fs;

fn assert_guide(locale: &str, zero: &str, empty: &str, before: &str, positive: &str) {
    let prefix = if locale.is_empty() {
        String::new()
    } else {
        format!("{locale}/")
    };
    let path = format!("site/src/content/docs/{prefix}guides/providers.mdx");
    let text = read(&path);
    let go = section(&text, "## OpenCode Go");
    for token in [
        "opencode-go",
        "opencode_go",
        "SHUNT_OPENCODE_GO_API_KEY",
        "https://opencode.ai/zen/go/v1",
        zero,
        empty,
        before,
    ] {
        assert!(go.contains(token), "{path}: missing {token:?}");
    }
    let link = format!("](/{prefix}providers/opencode-go/)");
    assert!(go.contains(&link), "{path}: missing locale provider link");
    assert!(
        !go.contains(positive),
        "{path}: affirmative Go support claim"
    );
}

#[test]
fn opencode_go_docs_guides_en() {
    assert_guide(
        "",
        "not supported",
        "empty admitted set",
        "before credential lookup",
        "OpenCode Go is supported",
    );
}

#[test]
fn opencode_go_docs_guides_ko() {
    assert_guide(
        "ko",
        "현재 지원되지 않습니다",
        "허용 집합은 비어",
        "자격 증명 조회",
        "OpenCode Go는 지원됩니다",
    );
}

#[test]
fn opencode_go_docs_guides_ja() {
    assert_guide(
        "ja",
        "現在サポートされていません",
        "許可集合は空",
        "資格情報の参照",
        "OpenCode Go はサポートされています",
    );
}

#[test]
fn opencode_go_docs_guides_zh_cn() {
    assert_guide(
        "zh-cn",
        "目前不受支持",
        "准入集合为空",
        "凭据查找",
        "OpenCode Go 受支持",
    );
}

fn read(path: &str) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("{path} is missing or unreadable: {error}"))
}

fn section(text: &str, heading: &str) -> String {
    let start = text
        .find(heading)
        .unwrap_or_else(|| panic!("required section {heading:?} is missing"));
    let line_start = text[..start].rfind('\n').map_or(0, |index| index + 1);
    let level = text[line_start..]
        .chars()
        .take_while(|character| *character == '#')
        .count();
    assert!(
        level > 0,
        "required section {heading:?} is not a Markdown heading"
    );
    let tail = &text[start..];
    let mut offset = tail.find('\n').map_or(tail.len(), |index| index + 1);
    while offset < tail.len() {
        let line_end = tail[offset..]
            .find('\n')
            .map_or(tail.len(), |index| offset + index);
        let line = &tail[offset..line_end];
        let candidate_level = line
            .chars()
            .take_while(|character| *character == '#')
            .count();
        if candidate_level > 0 && candidate_level <= level {
            return tail[..offset - 1].to_owned();
        }
        offset = line_end.saturating_add(1);
    }
    tail.to_owned()
}

#[test]
fn opencode_go_docs_section_scope() {
    let fixture = "### OpenCode Go\nbody\n#### nested\nnested body\n### Sibling\nsibling body";
    let extracted = section(fixture, "### OpenCode Go");
    assert!(extracted.contains("body"));
    assert!(extracted.contains("nested body"));
    assert!(!extracted.contains("Sibling"));
    assert!(!extracted.contains("sibling body"));
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

fn assert_locale_contract(
    text: &str,
    surface: &str,
    zero_support: &str,
    pre_credential: &str,
    strict_terminal: &str,
    forbidden_positive: &str,
) {
    for token in [
        "SHUNT_OPENCODE_GO_API_KEY",
        "https://opencode.ai/zen/go/v1",
        "opencode_go",
        "x-opencode-session",
    ] {
        assert!(
            text.contains(token),
            "{surface} is missing required token {token:?}"
        );
    }
    assert!(
        text.contains(zero_support),
        "{surface} is missing localized zero-support wording {zero_support:?}"
    );
    assert!(
        text.contains(pre_credential),
        "{surface} is missing localized pre-credential wording {pre_credential:?}"
    );
    assert!(
        text.contains(strict_terminal),
        "{surface} is missing localized terminal wording {strict_terminal:?}"
    );
    assert!(
        !text.contains(forbidden_positive),
        "{surface} contains forbidden localized positive claim {forbidden_positive:?}"
    );
    for candidate in [
        "glm-5.3-flash",
        "omen-alpha",
        "muse-spark-1.3-contributor",
        "deepseek-v4-flash",
    ] {
        assert!(
            text.contains(candidate),
            "{surface} is missing candidate {candidate:?}"
        );
    }
    assert!(
        !text.contains("](#future-promotion-contract)"),
        "{surface} invents an English locale fragment anchor"
    );
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

#[test]
fn opencode_go_docs_locale_readme_ko() {
    let text = read("README.ko.md");
    let scoped = section(&text, "## OpenCode Go (증거 게이트, 지원 없음)");
    assert_locale_contract(
        &scoped,
        "README.ko.md OpenCode Go section",
        "OpenCode Go는 현재 지원되지 않습니다",
        "자격 증명 전 게이트",
        "엄격한 권위 있는 터미널",
        "OpenCode Go는 지원됩니다",
    );
}

#[test]
fn opencode_go_docs_locale_readme_ja() {
    let text = read("README.ja.md");
    let scoped = section(&text, "## OpenCode Go（証拠ゲート、サポートなし）");
    assert_locale_contract(
        &scoped,
        "README.ja.md OpenCode Go section",
        "OpenCode Go は現在サポートされていません",
        "資格情報取得前ゲート",
        "厳格な権威ある終端",
        "OpenCode Go はサポートされています",
    );
}

#[test]
fn opencode_go_docs_locale_readme_zh_cn() {
    let text = read("README.zh-CN.md");
    let scoped = section(&text, "## OpenCode Go（证据门控，不支持）");
    assert_locale_contract(
        &scoped,
        "README.zh-CN.md OpenCode Go section",
        "OpenCode Go 目前不受支持",
        "凭据前门控",
        "严格的权威终止",
        "OpenCode Go 受支持",
    );
}

#[test]
fn opencode_go_docs_locale_provider_ko() {
    let text = read("site/src/content/docs/ko/providers/opencode-go.md");
    assert_locale_contract(
        &text,
        "ko OpenCode Go provider page",
        "OpenCode Go는 현재 지원되지 않습니다",
        "자격 증명 전 게이트",
        "엄격한 권위 있는 터미널",
        "OpenCode Go는 지원됩니다",
    );
}

#[test]
fn opencode_go_docs_locale_provider_ja() {
    let text = read("site/src/content/docs/ja/providers/opencode-go.md");
    assert_locale_contract(
        &text,
        "ja OpenCode Go provider page",
        "OpenCode Go は現在サポートされていません",
        "資格情報取得前ゲート",
        "厳格な権威ある終端",
        "OpenCode Go はサポートされています",
    );
}

#[test]
fn opencode_go_docs_locale_provider_zh_cn() {
    let text = read("site/src/content/docs/zh-cn/providers/opencode-go.md");
    assert_locale_contract(
        &text,
        "zh-cn OpenCode Go provider page",
        "OpenCode Go 目前不受支持",
        "凭据前门控",
        "严格的权威终止",
        "OpenCode Go 受支持",
    );
}

#[test]
fn opencode_go_docs_locale_configuration_ko() {
    let text = read("site/src/content/docs/ko/reference/configuration.md");
    let scoped = section(&text, "### OpenCode Go (증거 게이트, 지원 없음)");
    assert_locale_contract(
        &scoped,
        "ko configuration OpenCode Go section",
        "OpenCode Go는 현재 지원되지 않습니다",
        "자격 증명 전 게이트",
        "엄격한 권위 있는 터미널",
        "OpenCode Go는 지원됩니다",
    );
}

#[test]
fn opencode_go_docs_locale_configuration_ja() {
    let text = read("site/src/content/docs/ja/reference/configuration.md");
    let scoped = section(&text, "### OpenCode Go（証拠ゲート、サポートなし）");
    assert_locale_contract(
        &scoped,
        "ja configuration OpenCode Go section",
        "OpenCode Go は現在サポートされていません",
        "資格情報取得前ゲート",
        "厳格な権威ある終端",
        "OpenCode Go はサポートされています",
    );
}

#[test]
fn opencode_go_docs_locale_configuration_zh_cn() {
    let text = read("site/src/content/docs/zh-cn/reference/configuration.md");
    let scoped = section(&text, "### OpenCode Go（证据门控，不支持）");
    assert_locale_contract(
        &scoped,
        "zh-cn configuration OpenCode Go section",
        "OpenCode Go 目前不受支持",
        "凭据前门控",
        "严格的权威终止",
        "OpenCode Go 受支持",
    );
}
