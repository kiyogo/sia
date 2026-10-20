use crate::traits::{
    DiagnosticMessage, DiagnosticReport, DiagnosticSeverity, LanguageToolingAdapter, ProjectInfo,
};
use async_trait::async_trait;
use regex::Regex;
use std::fs;
use std::path::Path;

pub struct MultiLanguageAdapter;

impl MultiLanguageAdapter {
    pub fn new() -> Self {
        Self
    }

    /// Read git branch if repo has a .git folder
    pub fn read_git_info(root: &Path) -> (bool, Option<String>) {
        let git_dir = root.join(".git");
        if !git_dir.exists() {
            return (false, None);
        }

        let head_path = git_dir.join("HEAD");
        if let Ok(content) = fs::read_to_string(&head_path) {
            let trimmed = content.trim();
            if let Some(branch) = trimmed.strip_prefix("ref: refs/heads/") {
                return (true, Some(branch.to_string()));
            } else if !trimmed.is_empty() {
                // Detached HEAD, show short hash
                let short_hash = if trimmed.len() > 7 {
                    &trimmed[..7]
                } else {
                    trimmed
                };
                return (true, Some(format!("HEAD ({short_hash})")));
            }
        }

        (true, Some("HEAD".to_string()))
    }
}

impl Default for MultiLanguageAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LanguageToolingAdapter for MultiLanguageAdapter {
    fn language_name(&self) -> &'static str {
        "multi"
    }

    fn detect_project(&self, root: &Path) -> Option<ProjectInfo> {
        let mut languages = Vec::new();
        let mut build_systems = Vec::new();
        let mut test_runners = Vec::new();
        let mut entry_points = Vec::new();

        // 1. Rust detection
        if root.join("Cargo.toml").exists() {
            languages.push("Rust".to_string());
            build_systems.push("cargo".to_string());
            test_runners.push("cargo test".to_string());
            for ep in &["src/main.rs", "src/lib.rs", "main.rs"] {
                let p = root.join(ep);
                if p.exists() {
                    entry_points.push(p);
                }
            }
        }

        // 2. TypeScript / JavaScript detection
        let has_package_json = root.join("package.json").exists();
        let has_tsconfig = root.join("tsconfig.json").exists();
        if has_package_json || has_tsconfig {
            if has_tsconfig {
                languages.push("TypeScript".to_string());
            } else {
                languages.push("JavaScript".to_string());
            }

            // Detect package manager
            if root.join("pnpm-lock.yaml").exists() {
                build_systems.push("pnpm".to_string());
                test_runners.push("pnpm test".to_string());
            } else if root.join("yarn.lock").exists() {
                build_systems.push("yarn".to_string());
                test_runners.push("yarn test".to_string());
            } else if root.join("bun.lockb").exists() || root.join("bun.lock").exists() {
                build_systems.push("bun".to_string());
                test_runners.push("bun test".to_string());
            } else {
                build_systems.push("npm".to_string());
                test_runners.push("npm test".to_string());
            }

            for ep in &[
                "src/index.ts",
                "src/main.tsx",
                "src/main.ts",
                "src/App.tsx",
                "src/index.js",
                "index.ts",
                "index.js",
            ] {
                let p = root.join(ep);
                if p.exists() {
                    entry_points.push(p);
                }
            }
        }

        // 3. Python detection
        if root.join("pyproject.toml").exists()
            || root.join("requirements.txt").exists()
            || root.join("setup.py").exists()
            || root.join("Pipfile").exists()
        {
            languages.push("Python".to_string());
            if root.join("uv.lock").exists() {
                build_systems.push("uv".to_string());
                test_runners.push("uv run pytest".to_string());
            } else if root.join("poetry.lock").exists() {
                build_systems.push("poetry".to_string());
                test_runners.push("poetry run pytest".to_string());
            } else {
                build_systems.push("pip".to_string());
                test_runners.push("pytest".to_string());
            }

            for ep in &["main.py", "app.py", "src/main.py", "__main__.py"] {
                let p = root.join(ep);
                if p.exists() {
                    entry_points.push(p);
                }
            }
        }

        // 4. C / C++ detection
        if root.join("CMakeLists.txt").exists() || root.join("Makefile").exists() {
            if root.join("CMakeLists.txt").exists() {
                build_systems.push("cmake".to_string());
                test_runners.push("ctest".to_string());
            } else {
                build_systems.push("make".to_string());
                test_runners.push("make test".to_string());
            }
            languages.push("C/C++".to_string());

            for ep in &["main.cpp", "src/main.cpp", "main.c", "src/main.c"] {
                let p = root.join(ep);
                if p.exists() {
                    entry_points.push(p);
                }
            }
        }

        let (is_git, git_branch) = Self::read_git_info(root);
        let primary_language = languages.first().cloned();

        if languages.is_empty() && !is_git {
            return None;
        }

        Some(ProjectInfo {
            root_path: root.to_path_buf(),
            primary_language,
            languages,
            build_systems,
            test_runners,
            entry_points,
            is_git,
            git_branch,
        })
    }

    fn build_command(&self, root: &Path) -> Option<(String, Vec<String>)> {
        if root.join("Cargo.toml").exists() {
            return Some(("cargo".to_string(), vec!["check".to_string()]));
        }
        if root.join("package.json").exists() {
            if root.join("pnpm-lock.yaml").exists() {
                return Some(("pnpm".to_string(), vec!["build".to_string()]));
            }
            return Some(("npm".to_string(), vec!["run".to_string(), "build".to_string()]));
        }
        if root.join("pyproject.toml").exists() || root.join("requirements.txt").exists() {
            return Some((
                "python".to_string(),
                vec!["-m".to_string(), "py_compile".to_string()],
            ));
        }
        if root.join("CMakeLists.txt").exists() {
            return Some((
                "cmake".to_string(),
                vec!["--build".to_string(), ".".to_string()],
            ));
        }
        None
    }

    fn test_command(&self, root: &Path, filter: Option<&str>) -> Option<(String, Vec<String>)> {
        if root.join("Cargo.toml").exists() {
            let mut args = vec!["test".to_string()];
            if let Some(f) = filter {
                args.push(f.to_string());
            }
            return Some(("cargo".to_string(), args));
        }
        if root.join("package.json").exists() {
            let (cmd, mut args) = if root.join("pnpm-lock.yaml").exists() {
                ("pnpm".to_string(), vec!["test".to_string()])
            } else {
                ("npm".to_string(), vec!["test".to_string()])
            };
            if let Some(f) = filter {
                args.push("--".to_string());
                args.push(f.to_string());
            }
            return Some((cmd, args));
        }
        if root.join("pyproject.toml").exists() || root.join("requirements.txt").exists() {
            let mut args = vec!["-m".to_string(), "pytest".to_string()];
            if let Some(f) = filter {
                args.push("-k".to_string());
                args.push(f.to_string());
            }
            return Some(("python".to_string(), args));
        }
        None
    }

    fn parse_diagnostics(&self, stdout: &str, stderr: &str) -> DiagnosticReport {
        let combined = format!("{stdout}\n{stderr}");
        let mut diagnostics = Vec::new();

        // 1. Rust JSON message parsing
        for line in combined.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('{') && trimmed.ends_with('}') {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed) {
                    if val.get("reason").and_then(|r| r.as_str()) == Some("compiler-message") {
                        if let Some(msg) = val.get("message") {
                            let level = msg.get("level").and_then(|l| l.as_str()).unwrap_or("error");
                            let severity = match level {
                                "warning" => DiagnosticSeverity::Warning,
                                "error" => DiagnosticSeverity::Error,
                                _ => DiagnosticSeverity::Info,
                            };
                            let message_text = msg.get("message").and_then(|m| m.as_str()).unwrap_or("").to_string();
                            let code = msg.get("code").and_then(|c| c.get("code")).and_then(|c| c.as_str()).map(|s| s.to_string());

                            let mut file = String::new();
                            let mut line_num = 1;
                            let mut col_num = 1;
                            let mut snippet = None;

                            if let Some(spans) = msg.get("spans").and_then(|s| s.as_array()) {
                                if let Some(primary) = spans.iter().find(|s| s.get("is_primary").and_then(|p| p.as_bool()).unwrap_or(false)).or_else(|| spans.first()) {
                                    file = primary.get("file_name").and_then(|f| f.as_str()).unwrap_or("").to_string();
                                    line_num = primary.get("line_start").and_then(|l| l.as_u64()).unwrap_or(1) as usize;
                                    col_num = primary.get("column_start").and_then(|c| c.as_u64()).unwrap_or(1) as usize;
                                    snippet = primary.get("text").and_then(|t| t.as_array()).map(|arr| {
                                        arr.iter().filter_map(|t| t.get("text").and_then(|s| s.as_str())).collect::<Vec<_>>().join("\n")
                                    });
                                }
                            }

                            if !message_text.is_empty() {
                                diagnostics.push(DiagnosticMessage {
                                    file,
                                    line: line_num,
                                    column: col_num,
                                    severity,
                                    message: message_text,
                                    code,
                                    context_snippet: snippet,
                                });
                            }
                        }
                    }
                }
            }
        }

        // 2. Rust human-readable pattern:
        // error[E0425]: cannot find value `foo` in this scope
        //   --> src\main.rs:10:5
        let rust_re = Regex::new(r"(?m)^(error|warning)(?:\[([A-Z0-9]+)\])?: (.+)\n\s+-->\s+([^:]+):(\d+):(\d+)").unwrap();
        for cap in rust_re.captures_iter(&combined) {
            let level = &cap[1];
            let code = cap.get(2).map(|c| c.as_str().to_string());
            let message = cap[3].to_string();
            let file = cap[4].to_string();
            let line_num: usize = cap[5].parse().unwrap_or(1);
            let col_num: usize = cap[6].parse().unwrap_or(1);

            let severity = if level == "warning" {
                DiagnosticSeverity::Warning
            } else {
                DiagnosticSeverity::Error
            };

            diagnostics.push(DiagnosticMessage {
                file,
                line: line_num,
                column: col_num,
                severity,
                message,
                code,
                context_snippet: None,
            });
        }

        // 3. TypeScript / ESLint pattern:
        // src/index.ts:15:3 - error TS2304: Cannot find name 'foo'.
        // src/index.ts(15,3): error TS2304: Cannot find name 'foo'.
        let ts_re = Regex::new(r"(?m)^([^\s:(]+)(?::(\d+):(\d+)|\((\d+),(\d+)\))\s*-\s*(error|warning)\s*([A-Za-z0-9]+)?:\s*(.+)$").unwrap();
        for cap in ts_re.captures_iter(&combined) {
            let file = cap[1].to_string();
            let line_num: usize = cap.get(2).or_else(|| cap.get(4)).and_then(|m| m.as_str().parse().ok()).unwrap_or(1);
            let col_num: usize = cap.get(3).or_else(|| cap.get(5)).and_then(|m| m.as_str().parse().ok()).unwrap_or(1);
            let level = &cap[6];
            let code = cap.get(7).map(|c| c.as_str().to_string());
            let message = cap[8].to_string();

            let severity = if level == "warning" {
                DiagnosticSeverity::Warning
            } else {
                DiagnosticSeverity::Error
            };

            diagnostics.push(DiagnosticMessage {
                file,
                line: line_num,
                column: col_num,
                severity,
                message,
                code,
                context_snippet: None,
            });
        }

        // 4. Python traceback / SyntaxError pattern:
        // File "app.py", line 12
        //   x =
        // SyntaxError: invalid syntax
        let py_re = Regex::new(r#"(?m)File "([^"]+)", line (\d+)(?:, in .*)?\n(?:\s+(.+)\n)?\s*([A-Za-z0-9_]+Error): (.+)"#).unwrap();
        for cap in py_re.captures_iter(&combined) {
            let file = cap[1].to_string();
            let line_num: usize = cap[2].parse().unwrap_or(1);
            let snippet = cap.get(3).map(|m| m.as_str().trim().to_string());
            let err_type = &cap[4];
            let err_desc = &cap[5];

            diagnostics.push(DiagnosticMessage {
                file,
                line: line_num,
                column: 1,
                severity: DiagnosticSeverity::Error,
                message: format!("{err_type}: {err_desc}"),
                code: Some(err_type.to_string()),
                context_snippet: snippet,
            });
        }

        // 5. Generic gcc/clang format: file:line:col: (error|warning): msg
        let generic_re = Regex::new(r"(?m)^([^:\s]+):(\d+):(\d+):\s*(error|warning):\s*(.+)$").unwrap();
        for cap in generic_re.captures_iter(&combined) {
            let file = cap[1].to_string();
            let line_num: usize = cap[2].parse().unwrap_or(1);
            let col_num: usize = cap[3].parse().unwrap_or(1);
            let level = &cap[4];
            let message = cap[5].to_string();

            let severity = if level == "warning" {
                DiagnosticSeverity::Warning
            } else {
                DiagnosticSeverity::Error
            };

            // Avoid adding duplicate if already added
            if !diagnostics.iter().any(|d| d.file == file && d.line == line_num && d.message == message) {
                diagnostics.push(DiagnosticMessage {
                    file,
                    line: line_num,
                    column: col_num,
                    severity,
                    message,
                    code: None,
                    context_snippet: None,
                });
            }
        }

        // Deduplication & Root Cause Identification
        let mut unique_diagnostics = Vec::new();
        for d in diagnostics {
            if !unique_diagnostics.iter().any(|u: &DiagnosticMessage| u.file == d.file && u.line == d.line && u.message == d.message) {
                unique_diagnostics.push(d);
            }
        }

        let total_errors = unique_diagnostics.iter().filter(|d| d.severity == DiagnosticSeverity::Error).count();
        let total_warnings = unique_diagnostics.iter().filter(|d| d.severity == DiagnosticSeverity::Warning).count();

        // The root cause is the first error encountered
        let root_cause = unique_diagnostics.iter().find(|d| d.severity == DiagnosticSeverity::Error).cloned();

        DiagnosticReport {
            diagnostics: unique_diagnostics,
            root_cause,
            total_errors,
            total_warnings,
            raw_output: combined,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_detect_rust_project() {
        let temp = tempdir().expect("tempdir");
        let root = temp.path();
        fs::write(root.join("Cargo.toml"), "[package]\nname=\"foo\"\nversion=\"0.1.0\"\n").expect("write");
        let src = root.join("src");
        fs::create_dir_all(&src).expect("mkdir");
        fs::write(src.join("main.rs"), "fn main() {}\n").expect("write main");

        let adapter = MultiLanguageAdapter::new();
        let info = adapter.detect_project(root).expect("detected");

        assert_eq!(info.primary_language.as_deref(), Some("Rust"));
        assert!(info.languages.contains(&"Rust".to_string()));
        assert!(info.build_systems.contains(&"cargo".to_string()));
        assert_eq!(info.entry_points.len(), 1);
    }

    #[test]
    fn test_parse_rust_error() {
        let stderr = r#"
error[E0425]: cannot find value `my_var` in this scope
  --> src/main.rs:14:9
   |
14 |         my_var += 1;
   |         ^^^^^^ not found in this scope
"#;
        let adapter = MultiLanguageAdapter::new();
        let report = adapter.parse_diagnostics("", stderr);

        assert_eq!(report.total_errors, 1);
        let root = report.root_cause.expect("root cause");
        assert_eq!(root.file, "src/main.rs");
        assert_eq!(root.line, 14);
        assert_eq!(root.column, 9);
        assert_eq!(root.code.as_deref(), Some("E0425"));
    }

    #[test]
    fn test_parse_ts_error() {
        let output = "src/components/App.tsx:42:10 - error TS2304: Cannot find name 'UnknownWidget'.\n";
        let adapter = MultiLanguageAdapter::new();
        let report = adapter.parse_diagnostics(output, "");

        assert_eq!(report.total_errors, 1);
        let root = report.root_cause.expect("root cause");
        assert_eq!(root.file, "src/components/App.tsx");
        assert_eq!(root.line, 42);
        assert_eq!(root.column, 10);
        assert_eq!(root.code.as_deref(), Some("TS2304"));
    }

    #[test]
    fn test_parse_python_error() {
        let output = r#"
Traceback (most recent call last):
  File "app.py", line 25, in <module>
    calculate_total()
ZeroDivisionError: division by zero
"#;
        let adapter = MultiLanguageAdapter::new();
        let report = adapter.parse_diagnostics(output, "");

        assert_eq!(report.total_errors, 1);
        let root = report.root_cause.expect("root cause");
        assert_eq!(root.file, "app.py");
        assert_eq!(root.line, 25);
        assert!(root.message.contains("ZeroDivisionError"));
    }
}
