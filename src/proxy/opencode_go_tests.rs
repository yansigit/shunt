use std::fs;

use crate::config::Config;

#[test]
fn opencode_go_config_acceptance() {
    let path = std::env::temp_dir().join(format!(
        "shunt-opencode-go-acceptance-{}-{}.toml",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock before epoch")
            .as_nanos()
    ));
    let synthetic = r#"
[[upstreams]]
name = "go"
provider = "opencode-go"
"#;
    fs::write(&path, synthetic).expect("write synthetic config");
    let result = Config::load(Some(&path));
    let _ = fs::remove_file(&path);
    assert!(result.is_ok(), "{result:?}");
}
