use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ToolTier {
    Tier1DirectApi = 1,
    Tier2FilesystemDb = 2,
    Tier3ShellCliCoding = 3,
    Tier4BrowserPlaywright = 4,
    Tier5WindowsUiAutomation = 5,
    Tier6VisionScreenshot = 6,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStrategy {
    pub tier: ToolTier,
    pub tier_name: String,
    pub recommended_tools: Vec<String>,
    pub rationale: String,
}

pub struct ToolStrategyRouter;

impl ToolStrategyRouter {
    pub fn new() -> Self {
        Self
    }

    /// Evaluates user goal and returns the most token-efficient, deterministic strategy tier
    pub fn route(&self, goal: &str) -> TaskStrategy {
        let lower = goal.to_lowercase();

        // 1. Check for explicit Visual/Screenshot fallback request
        if (lower.contains("screenshot") || lower.contains("chụp màn hình") || lower.contains("xem màn hình"))
            && !lower.contains("navigate") && !lower.contains("web")
        {
            return TaskStrategy {
                tier: ToolTier::Tier6VisionScreenshot,
                tier_name: "Tier 6: Vision & Screenshot Fallback".to_string(),
                recommended_tools: vec!["windows_mcp_screenshot".to_string(), "browser_take_screenshot".to_string()],
                rationale: "Direct visual inspection requested. Using downsampled screen capture.".to_string(),
            };
        }

        // 2. Check for Web / Browser tasks
        if lower.contains("http://")
            || lower.contains("https://")
            || lower.contains("browser")
            || lower.contains("trình duyệt")
            || lower.contains("website")
            || lower.contains("web page")
            || lower.contains("navigate to")
            || lower.contains("playwright")
            || lower.contains("dom")
            || lower.contains("url")
            || lower.contains("biểu mẫu web")
            || lower.contains("web form")
        {
            return TaskStrategy {
                tier: ToolTier::Tier4BrowserPlaywright,
                tier_name: "Tier 4: Browser Automation (Playwright MCP)".to_string(),
                recommended_tools: vec![
                    "playwright_browser_navigate".to_string(),
                    "playwright_browser_snapshot".to_string(),
                    "playwright_browser_click".to_string(),
                    "playwright_browser_fill_form".to_string(),
                    "playwright_browser_tabs".to_string(),
                ],
                rationale: "Web/Browser interaction detected. Preferring DOM accessibility snapshots over pixel screenshots.".to_string(),
            };
        }

        // 3. Check for Desktop UI App tasks (Notepad, Calculator, Windows Settings, Window switching)
        if lower.contains("notepad")
            || lower.contains("calc")
            || lower.contains("calculator")
            || lower.contains("cửa sổ")
            || lower.contains("active window")
            || lower.contains("switch window")
            || lower.contains("desktop app")
            || lower.contains("ứng dụng desktop")
            || lower.contains("windows-mcp")
            || lower.contains("click nút")
            || lower.contains("bấm nút")
        {
            return TaskStrategy {
                tier: ToolTier::Tier5WindowsUiAutomation,
                tier_name: "Tier 5: Windows UI Automation (Windows-MCP)".to_string(),
                recommended_tools: vec![
                    "windows_mcp_App".to_string(),
                    "windows_mcp_Snapshot".to_string(),
                    "windows_mcp_Click".to_string(),
                    "windows_mcp_Type".to_string(),
                    "windows_mcp_Shortcut".to_string(),
                ],
                rationale: "Desktop application automation detected. Utilizing UI Automation accessibility tree.".to_string(),
            };
        }

        // 4. Check for Build / Test / Git / Coding tasks
        if lower.contains("build")
            || lower.contains("test")
            || lower.contains("cargo")
            || lower.contains("npm")
            || lower.contains("git")
            || lower.contains("commit")
            || lower.contains("compile")
            || lower.contains("diagnostics")
            || lower.contains("linter")
            || lower.contains("patch")
        {
            return TaskStrategy {
                tier: ToolTier::Tier3ShellCliCoding,
                tier_name: "Tier 3: Terminal & Autonomous Coding Agent".to_string(),
                recommended_tools: vec![
                    "run_build".to_string(),
                    "run_test".to_string(),
                    "git_status".to_string(),
                    "git_diff".to_string(),
                    "apply_patch".to_string(),
                    "terminal_execute".to_string(),
                ],
                rationale: "Software development or build operation detected. Direct CLI/coding tools provide machine-readable error codes.".to_string(),
            };
        }

        // 5. Check for File tasks
        if lower.contains("read file")
            || lower.contains("write file")
            || lower.contains("đọc file")
            || lower.contains("ghi file")
            || lower.contains("tạo file")
            || lower.contains("list directory")
            || lower.contains("danh sách file")
            || lower.contains("search file")
            || lower.contains("tìm kiếm file")
        {
            return TaskStrategy {
                tier: ToolTier::Tier2FilesystemDb,
                tier_name: "Tier 2: Filesystem & Database Tools".to_string(),
                recommended_tools: vec![
                    "file_read".to_string(),
                    "file_write".to_string(),
                    "file_list".to_string(),
                    "file_search".to_string(),
                ],
                rationale: "File operation detected. Operating directly via native filesystem tools (fastest & safest).".to_string(),
            };
        }

        // Default: Tier 3 Shell/CLI
        TaskStrategy {
            tier: ToolTier::Tier3ShellCliCoding,
            tier_name: "Tier 3: General Terminal / CLI Tooling".to_string(),
            recommended_tools: vec!["terminal_execute".to_string()],
            rationale: "Default general-purpose execution strategy via shell commands.".to_string(),
        }
    }
}

impl Default for ToolStrategyRouter {
    fn default() -> Self {
        Self::new()
    }
}
