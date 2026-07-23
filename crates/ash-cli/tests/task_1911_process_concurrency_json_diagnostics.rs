use assert_cmd::Command;
use serde_json::Value;
use tempfile::tempdir;

#[test]
fn ash_check_json_reports_structured_process_channel_row_boundary_diagnostic() {
    for family in [
        "requires",
        "ensures",
        "invariant",
        "law",
        "proof",
        "contract",
    ] {
        let dir = tempdir().expect("tempdir");
        let path = dir
            .path()
            .join(format!("invalid-process-channel-{family}-row.ash"));
        std::fs::write(
            &path,
            format!(
                r#"
        fn invalid(job: Int) -> Int
        where
            row {{
                process spawn,
                channel jobs,
                {family}_fact
            }}
        {{
            job
        }}

        fn main() {{ 0 }}
        "#,
            ),
        )
        .expect("write invalid process/channel fixture");

        let output = Command::cargo_bin("ash")
            .expect("ash binary exists")
            .args(["check", "--format", "json"])
            .arg(&path)
            .output()
            .expect("run ash check json");
        assert!(
            !output.status.success(),
            "{family} predicate-style row boundary should fail\nstdout={}\nstderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        let json: Value =
            serde_json::from_slice(&output.stdout).expect("ash check should emit JSON diagnostics");
        let diagnostics = json["diagnostics"]
            .as_array()
            .expect("diagnostics should be an array");
        assert!(!diagnostics.is_empty(), "family={family} json={json}");

        let diagnostic = &diagnostics[0];
        assert_eq!(diagnostic["severity"].as_str(), Some("error"));
        assert_eq!(diagnostic["code"].as_str(), Some("E181"), "json={json}");
        assert!(
            diagnostic["message"].as_str().is_some_and(|message| {
                message.contains("unsupported")
                    && message.contains("row")
                    && message.contains(family)
            }),
            "json={json}"
        );
        assert!(diagnostic["location"]["file"].as_str().is_some());
        assert!(diagnostic["location"]["line"].as_u64().is_some());
        assert!(diagnostic["location"]["column"].as_u64().is_some());
    }
}
