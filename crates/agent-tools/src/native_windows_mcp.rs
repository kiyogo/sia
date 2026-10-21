use crate::types::{Tool, ToolDefinition, ToolError, ToolExecutionResult, ToolResult};
use agent_platform::{
    hidden_tokio_command, ComputerControlAdapter, WindowsComputerControlAdapter,
};
use agent_security::{PermissionTier, SharedSecurityEngine};
use async_trait::async_trait;
use serde_json::json;
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[cfg(windows)]
#[allow(dead_code)]
mod win32 {
    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub struct POINT {
        pub x: i32,
        pub y: i32,
    }

    #[link(name = "user32")]
    extern "system" {
        pub fn SetCursorPos(x: i32, y: i32) -> i32;
        pub fn GetCursorPos(lpPoint: *mut POINT) -> i32;
        pub fn mouse_event(dwFlags: u32, dx: u32, dy: u32, dwData: u32, dwExtraInfo: usize);
        pub fn keybd_event(bVk: u8, bScan: u8, dwFlags: u32, dwExtraInfo: usize);
        pub fn GetSystemMetrics(nIndex: i32) -> i32;
    }

    #[link(name = "shell32")]
    extern "system" {
        pub fn ShellExecuteW(
            hwnd: isize,
            lpOperation: *const u16,
            lpFile: *const u16,
            lpParameters: *const u16,
            lpDirectory: *const u16,
            nShowCmd: i32,
        ) -> isize;
    }

    pub const MOUSEEVENTF_LEFTDOWN: u32 = 0x0002;
    pub const MOUSEEVENTF_LEFTUP: u32 = 0x0004;
    pub const MOUSEEVENTF_RIGHTDOWN: u32 = 0x0008;
    pub const MOUSEEVENTF_RIGHTUP: u32 = 0x0010;
    pub const MOUSEEVENTF_MIDDLEDOWN: u32 = 0x0020;
    pub const MOUSEEVENTF_MIDDLEUP: u32 = 0x0040;
    pub const MOUSEEVENTF_WHEEL: u32 = 0x0800;
    pub const MOUSEEVENTF_HWHEEL: u32 = 0x01000;

    pub const KEYEVENTF_KEYUP: u32 = 0x0002;
    pub const KEYEVENTF_UNICODE: u32 = 0x0004;

    pub const VK_BACK: u8 = 0x08;
    pub const VK_TAB: u8 = 0x09;
    pub const VK_RETURN: u8 = 0x0D;
    pub const VK_SHIFT: u8 = 0x10;
    pub const VK_CONTROL: u8 = 0x11;
    pub const VK_MENU: u8 = 0x12; // Alt
    pub const VK_ESCAPE: u8 = 0x1B;
    pub const VK_SPACE: u8 = 0x20;
    pub const VK_PRIOR: u8 = 0x21; // Page Up
    pub const VK_NEXT: u8 = 0x22;  // Page Down
    pub const VK_END: u8 = 0x23;
    pub const VK_HOME: u8 = 0x24;
    pub const VK_LEFT: u8 = 0x25;
    pub const VK_UP: u8 = 0x26;
    pub const VK_RIGHT: u8 = 0x27;
    pub const VK_DOWN: u8 = 0x28;
    pub const VK_DELETE: u8 = 0x2E;
    pub const VK_LWIN: u8 = 0x5B;

    pub const SM_CXSCREEN: i32 = 0;
    pub const SM_CYSCREEN: i32 = 1;
}

/// Helper to parse coordinates from varied JSON representations: [x, y], "[x, y]", "x, y", or {"x": x, "y": y}
pub fn parse_coordinates(value: &serde_json::Value) -> Result<(i32, i32), String> {
    if let Some(arr) = value.as_array() {
        if arr.len() >= 2 {
            let x = arr[0].as_i64().ok_or("loc[0] must be an integer")? as i32;
            let y = arr[1].as_i64().ok_or("loc[1] must be an integer")? as i32;
            return Ok((x, y));
        }
    } else if let Some(s) = value.as_str() {
        let trimmed = s.trim().trim_start_matches('[').trim_end_matches(']');
        let parts: Vec<&str> = trimmed.split(',').map(|p| p.trim()).collect();
        if parts.len() >= 2 {
            let x = parts[0].parse::<i32>().map_err(|e| format!("Invalid x coordinate: {e}"))?;
            let y = parts[1].parse::<i32>().map_err(|e| format!("Invalid y coordinate: {e}"))?;
            return Ok((x, y));
        }
    } else if let Some(obj) = value.as_object() {
        if let (Some(x_val), Some(y_val)) = (obj.get("x"), obj.get("y")) {
            let x = x_val.as_i64().ok_or("x must be an integer")? as i32;
            let y = y_val.as_i64().ok_or("y must be an integer")? as i32;
            return Ok((x, y));
        }
    }
    Err("Invalid coordinates. Expected [x, y] format.".to_string())
}

/// Smart application resolution across all standard Windows installation paths
pub fn resolve_windows_app(name: &str) -> Option<PathBuf> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return None;
    }

    // 1. Direct path check
    let direct = PathBuf::from(trimmed);
    if direct.is_file() {
        return Some(direct);
    }

    let lower = trimmed.to_lowercase();
    let lower_stem = lower
        .trim_start_matches("app ")
        .trim_start_matches("ứng dụng ")
        .trim_start_matches("ung dung ")
        .trim_start_matches("phần mềm ")
        .trim_start_matches("phan mem ")
        .trim_end_matches(".exe")
        .trim_end_matches(".lnk")
        .trim();

    // 2. Standard Windows system built-in tools
    match lower_stem {
        "notepad" => return Some(PathBuf::from("notepad.exe")),
        "calc" | "calculator" => return Some(PathBuf::from("calc.exe")),
        "mspaint" | "paint" => return Some(PathBuf::from("mspaint.exe")),
        "cmd" => return Some(PathBuf::from("cmd.exe")),
        "powershell" => return Some(PathBuf::from("powershell.exe")),
        "explorer" => return Some(PathBuf::from("explorer.exe")),
        "taskmgr" => return Some(PathBuf::from("taskmgr.exe")),
        _ => {}
    }

    // 3. Search common program and shortcut locations (Local AppData, Start Menu, Desktop, Program Files)
    let mut search_dirs = Vec::new();
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        search_dirs.push(PathBuf::from(&local_app_data).join("Programs"));
    }
    if let Ok(app_data) = std::env::var("APPDATA") {
        search_dirs.push(PathBuf::from(&app_data).join(r"Microsoft\Windows\Start Menu\Programs"));
    }
    search_dirs.push(PathBuf::from(r"C:\ProgramData\Microsoft\Windows\Start Menu\Programs"));
    if let Ok(user_profile) = std::env::var("USERPROFILE") {
        search_dirs.push(PathBuf::from(&user_profile).join("Desktop"));
    }
    search_dirs.push(PathBuf::from(r"C:\Users\Public\Desktop"));
    search_dirs.push(PathBuf::from(r"C:\Program Files"));
    search_dirs.push(PathBuf::from(r"C:\Program Files (x86)"));

    let mut exact_matches = Vec::new();
    let mut prefix_matches = Vec::new();
    let mut substring_matches = Vec::new();

    for dir in search_dirs {
        if !dir.exists() {
            continue;
        }
        for entry in walkdir::WalkDir::new(&dir).max_depth(5).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
                let f_lower = file_name.to_lowercase();
                if f_lower.ends_with(".exe") || f_lower.ends_with(".lnk") {
                    // Exclude uninstallers, updaters, crash handlers, setups, or helpers
                    if f_lower.contains("unins")
                        || f_lower.contains("setup")
                        || f_lower.contains("update")
                        || f_lower.contains("helper")
                        || f_lower.contains("crash")
                        || f_lower.contains("elevate")
                    {
                        continue;
                    }

                    let stem = f_lower.trim_end_matches(".exe").trim_end_matches(".lnk");
                    if stem == lower_stem {
                        exact_matches.push(path.to_path_buf());
                    } else if stem.starts_with(lower_stem) {
                        prefix_matches.push(path.to_path_buf());
                    } else if stem.contains(lower_stem) {
                        substring_matches.push(path.to_path_buf());
                    }
                }
            }
        }
    }

    if let Some(exact) = exact_matches.into_iter().next() {
        return Some(exact);
    }
    if let Some(prefix) = prefix_matches.into_iter().next() {
        return Some(prefix);
    }
    if let Some(substring) = substring_matches.into_iter().next() {
        return Some(substring);
    }

    None
}

// ---------------------------------------------------------------------------
// 1. DesktopClickTool (Click / desktop_click)
// ---------------------------------------------------------------------------
pub struct DesktopClickTool {
    security: SharedSecurityEngine,
}

impl DesktopClickTool {
    pub fn new(security: SharedSecurityEngine) -> Self {
        Self { security }
    }
}

#[async_trait]
impl Tool for DesktopClickTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "desktop_click".to_string(),
            description: "[Windows-MCP] Performs mouse click or hover at specified coordinates [x, y]. Supports button: 'left', 'right', 'middle'. Supports clicks: 0=hover, 1=single click, 2=double click.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "loc": {
                        "type": "array",
                        "items": { "type": "integer" },
                        "description": "Target screen coordinates [x, y]"
                    },
                    "button": {
                        "type": "string",
                        "enum": ["left", "right", "middle"],
                        "default": "left",
                        "description": "Mouse button to click"
                    },
                    "clicks": {
                        "type": "integer",
                        "default": 1,
                        "description": "Number of clicks: 0=hover, 1=single click, 2=double click"
                    }
                },
                "required": ["loc"]
            }),
        }
    }

    async fn execute(&self, call_id: &str, params: serde_json::Value) -> ToolResult<ToolExecutionResult> {
        let start = Instant::now();
        self.security
            .check_permission(
                PermissionTier::ShellExecution,
                "Desktop automation mouse click",
                None,
                params.clone(),
            )
            .await?;

        let loc_val = params.get("loc").ok_or_else(|| ToolError::InvalidParameters("loc parameter is required".to_string()))?;
        let (x, y) = parse_coordinates(loc_val).map_err(ToolError::InvalidParameters)?;
        let button = params.get("button").and_then(|v| v.as_str()).unwrap_or("left");
        let clicks = params.get("clicks").and_then(|v| v.as_i64()).unwrap_or(1) as i32;

        #[cfg(windows)]
        {
            unsafe {
                win32::SetCursorPos(x, y);
            }

            if clicks > 0 {
                let (down_flag, up_flag) = match button {
                    "right" => (win32::MOUSEEVENTF_RIGHTDOWN, win32::MOUSEEVENTF_RIGHTUP),
                    "middle" => (win32::MOUSEEVENTF_MIDDLEDOWN, win32::MOUSEEVENTF_MIDDLEUP),
                    _ => (win32::MOUSEEVENTF_LEFTDOWN, win32::MOUSEEVENTF_LEFTUP),
                };

                for i in 0..clicks {
                    if i > 0 {
                        tokio::time::sleep(Duration::from_millis(60)).await;
                    }
                    unsafe {
                        win32::mouse_event(down_flag, 0, 0, 0, 0);
                        win32::mouse_event(up_flag, 0, 0, 0, 0);
                    }
                }
            }
        }

        let action_name = match clicks {
            0 => "Hovered",
            1 => "Single clicked",
            2 => "Double clicked",
            _ => "Clicked",
        };

        let msg = format!("{action_name} {button} button at ({x}, {y}).");
        Ok(ToolExecutionResult {
            tool_call_id: call_id.to_string(),
            tool_name: "desktop_click".to_string(),
            stdout: msg,
            stderr: String::new(),
            exit_code: Some(0),
            duration_ms: start.elapsed().as_millis() as u64,
            is_error: false,
            truncated: false,
        })
    }
}

// ---------------------------------------------------------------------------
// 2. DesktopTypeTool (Type / desktop_type)
// ---------------------------------------------------------------------------
pub struct DesktopTypeTool {
    security: SharedSecurityEngine,
}

impl DesktopTypeTool {
    pub fn new(security: SharedSecurityEngine) -> Self {
        Self { security }
    }
}

#[async_trait]
impl Tool for DesktopTypeTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "desktop_type".to_string(),
            description: "[Windows-MCP] Types text at optional coordinates [x, y] or current focus. Set clear=true to clear existing field contents. Set press_enter=true to send Enter after typing.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "Text to type"
                    },
                    "loc": {
                        "type": "array",
                        "items": { "type": "integer" },
                        "description": "Optional screen coordinates [x, y] to click and focus before typing"
                    },
                    "clear": {
                        "type": "boolean",
                        "default": false,
                        "description": "Clear field before typing (Ctrl+A then Backspace)"
                    },
                    "press_enter": {
                        "type": "boolean",
                        "default": false,
                        "description": "Press Enter key after typing"
                    }
                },
                "required": ["text"]
            }),
        }
    }

    async fn execute(&self, call_id: &str, params: serde_json::Value) -> ToolResult<ToolExecutionResult> {
        let start = Instant::now();
        self.security
            .check_permission(
                PermissionTier::ShellExecution,
                "Desktop automation keyboard typing",
                None,
                params.clone(),
            )
            .await?;

        let text = params.get("text").and_then(|v| v.as_str()).unwrap_or("");
        let clear = params.get("clear").and_then(|v| v.as_bool()).unwrap_or(false);
        let press_enter = params.get("press_enter").and_then(|v| v.as_bool()).unwrap_or(false);

        #[cfg(windows)]
        {
            // 1. If loc provided, click to focus
            if let Some(loc_val) = params.get("loc") {
                if let Ok((x, y)) = parse_coordinates(loc_val) {
                    unsafe {
                        win32::SetCursorPos(x, y);
                        win32::mouse_event(win32::MOUSEEVENTF_LEFTDOWN, 0, 0, 0, 0);
                        win32::mouse_event(win32::MOUSEEVENTF_LEFTUP, 0, 0, 0, 0);
                    }
                    tokio::time::sleep(Duration::from_millis(80)).await;
                }
            }

            // 2. If clear requested, select all and backspace
            if clear {
                unsafe {
                    // Ctrl + A
                    win32::keybd_event(win32::VK_CONTROL, 0, 0, 0);
                    win32::keybd_event(b'A', 0, 0, 0);
                    win32::keybd_event(b'A', 0, win32::KEYEVENTF_KEYUP, 0);
                    win32::keybd_event(win32::VK_CONTROL, 0, win32::KEYEVENTF_KEYUP, 0);

                    // Backspace
                    win32::keybd_event(win32::VK_BACK, 0, 0, 0);
                    win32::keybd_event(win32::VK_BACK, 0, win32::KEYEVENTF_KEYUP, 0);
                }
                tokio::time::sleep(Duration::from_millis(50)).await;
            }

            // 3. Type text via Unicode key events
            for c in text.encode_utf16() {
                unsafe {
                    win32::keybd_event(0, (c & 0xFF) as u8, win32::KEYEVENTF_UNICODE, 0);
                    win32::keybd_event(0, (c & 0xFF) as u8, win32::KEYEVENTF_UNICODE | win32::KEYEVENTF_KEYUP, 0);
                }
            }

            // 4. Press Enter if requested
            if press_enter {
                tokio::time::sleep(Duration::from_millis(50)).await;
                unsafe {
                    win32::keybd_event(win32::VK_RETURN, 0, 0, 0);
                    win32::keybd_event(win32::VK_RETURN, 0, win32::KEYEVENTF_KEYUP, 0);
                }
            }
        }

        let summary = format!(
            "Typed '{}' (clear={}, press_enter={})",
            if text.len() > 60 { format!("{}...", &text[..60]) } else { text.to_string() },
            clear,
            press_enter
        );

        Ok(ToolExecutionResult {
            tool_call_id: call_id.to_string(),
            tool_name: "desktop_type".to_string(),
            stdout: summary,
            stderr: String::new(),
            exit_code: Some(0),
            duration_ms: start.elapsed().as_millis() as u64,
            is_error: false,
            truncated: false,
        })
    }
}

// ---------------------------------------------------------------------------
// 3. DesktopScrollTool (Scroll / desktop_scroll)
// ---------------------------------------------------------------------------
pub struct DesktopScrollTool {
    security: SharedSecurityEngine,
}

impl DesktopScrollTool {
    pub fn new(security: SharedSecurityEngine) -> Self {
        Self { security }
    }
}

#[async_trait]
impl Tool for DesktopScrollTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "desktop_scroll".to_string(),
            description: "[Windows-MCP] Scrolls at coordinates [x, y] or current cursor location. Direction: 'up', 'down', 'left', 'right'. wheel_times controls scroll amount.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "loc": {
                        "type": "array",
                        "items": { "type": "integer" },
                        "description": "Optional screen coordinates [x, y]"
                    },
                    "type": {
                        "type": "string",
                        "enum": ["vertical", "horizontal"],
                        "default": "vertical"
                    },
                    "direction": {
                        "type": "string",
                        "enum": ["up", "down", "left", "right"],
                        "default": "down"
                    },
                    "wheel_times": {
                        "type": "integer",
                        "default": 1,
                        "description": "Number of scroll clicks"
                    }
                }
            }),
        }
    }

    async fn execute(&self, call_id: &str, params: serde_json::Value) -> ToolResult<ToolExecutionResult> {
        let start = Instant::now();
        self.security
            .check_permission(
                PermissionTier::ShellExecution,
                "Desktop automation mouse scroll",
                None,
                params.clone(),
            )
            .await?;

        let direction = params.get("direction").and_then(|v| v.as_str()).unwrap_or("down");
        let scroll_type = params.get("type").and_then(|v| v.as_str()).unwrap_or("vertical");
        let wheel_times = params.get("wheel_times").and_then(|v| v.as_i64()).unwrap_or(1) as i32;

        #[cfg(windows)]
        {
            if let Some(loc_val) = params.get("loc") {
                if let Ok((x, y)) = parse_coordinates(loc_val) {
                    unsafe {
                        win32::SetCursorPos(x, y);
                    }
                }
            }

            let delta = if direction == "up" || direction == "right" {
                (120 * wheel_times) as u32
            } else {
                ((-120 * wheel_times) as i32) as u32
            };

            let flag = if scroll_type == "horizontal" {
                win32::MOUSEEVENTF_HWHEEL
            } else {
                win32::MOUSEEVENTF_WHEEL
            };

            unsafe {
                win32::mouse_event(flag, 0, 0, delta, 0);
            }
        }

        let msg = format!("Scrolled {scroll_type} {direction} by {wheel_times} wheel units.");
        Ok(ToolExecutionResult {
            tool_call_id: call_id.to_string(),
            tool_name: "desktop_scroll".to_string(),
            stdout: msg,
            stderr: String::new(),
            exit_code: Some(0),
            duration_ms: start.elapsed().as_millis() as u64,
            is_error: false,
            truncated: false,
        })
    }
}

// ---------------------------------------------------------------------------
// 4. DesktopMoveTool (Move / desktop_move)
// ---------------------------------------------------------------------------
pub struct DesktopMoveTool {
    security: SharedSecurityEngine,
}

impl DesktopMoveTool {
    pub fn new(security: SharedSecurityEngine) -> Self {
        Self { security }
    }
}

#[async_trait]
impl Tool for DesktopMoveTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "desktop_move".to_string(),
            description: "[Windows-MCP] Moves mouse cursor to coordinates [x, y]. Set drag=true to perform mouse drag from start position to target.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "loc": {
                        "type": "array",
                        "items": { "type": "integer" },
                        "description": "Target screen coordinates [x, y]"
                    },
                    "drag": {
                        "type": "boolean",
                        "default": false,
                        "description": "Whether to drag from current/from_loc to target"
                    },
                    "from_loc": {
                        "type": "array",
                        "items": { "type": "integer" },
                        "description": "Optional starting coordinates [x, y] for drag"
                    }
                },
                "required": ["loc"]
            }),
        }
    }

    async fn execute(&self, call_id: &str, params: serde_json::Value) -> ToolResult<ToolExecutionResult> {
        let start = Instant::now();
        self.security
            .check_permission(
                PermissionTier::ShellExecution,
                "Desktop automation mouse movement",
                None,
                params.clone(),
            )
            .await?;

        let loc_val = params.get("loc").ok_or_else(|| ToolError::InvalidParameters("loc parameter is required".to_string()))?;
        let (tx, ty) = parse_coordinates(loc_val).map_err(ToolError::InvalidParameters)?;
        let drag = params.get("drag").and_then(|v| v.as_bool()).unwrap_or(false);

        #[cfg(windows)]
        {
            if drag {
                if let Some(from_val) = params.get("from_loc") {
                    if let Ok((fx, fy)) = parse_coordinates(from_val) {
                        unsafe {
                            win32::SetCursorPos(fx, fy);
                        }
                    }
                }
                unsafe {
                    win32::mouse_event(win32::MOUSEEVENTF_LEFTDOWN, 0, 0, 0, 0);
                }
                tokio::time::sleep(Duration::from_millis(50)).await;
                unsafe {
                    win32::SetCursorPos(tx, ty);
                    win32::mouse_event(win32::MOUSEEVENTF_LEFTUP, 0, 0, 0, 0);
                }
            } else {
                unsafe {
                    win32::SetCursorPos(tx, ty);
                }
            }
        }

        let msg = if drag {
            format!("Dragged mouse to ({tx}, {ty}).")
        } else {
            format!("Moved mouse to ({tx}, {ty}).")
        };

        Ok(ToolExecutionResult {
            tool_call_id: call_id.to_string(),
            tool_name: "desktop_move".to_string(),
            stdout: msg,
            stderr: String::new(),
            exit_code: Some(0),
            duration_ms: start.elapsed().as_millis() as u64,
            is_error: false,
            truncated: false,
        })
    }
}

// ---------------------------------------------------------------------------
// 5. DesktopShortcutTool (Shortcut / desktop_shortcut)
// ---------------------------------------------------------------------------
pub struct DesktopShortcutTool {
    security: SharedSecurityEngine,
}

impl DesktopShortcutTool {
    pub fn new(security: SharedSecurityEngine) -> Self {
        Self { security }
    }
}

#[async_trait]
impl Tool for DesktopShortcutTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "desktop_shortcut".to_string(),
            description: "[Windows-MCP] Executes keyboard shortcuts separated by +. Examples: 'ctrl+c', 'ctrl+v', 'alt+tab', 'win+r', 'ctrl+shift+esc', 'enter', 'escape'.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "shortcut": {
                        "type": "string",
                        "description": "Keyboard shortcut (e.g. 'ctrl+c', 'win+r')"
                    }
                },
                "required": ["shortcut"]
            }),
        }
    }

    async fn execute(&self, call_id: &str, params: serde_json::Value) -> ToolResult<ToolExecutionResult> {
        let start = Instant::now();
        self.security
            .check_permission(
                PermissionTier::ShellExecution,
                "Desktop automation keyboard shortcut",
                None,
                params.clone(),
            )
            .await?;

        let shortcut = params.get("shortcut").and_then(|v| v.as_str()).unwrap_or("").to_lowercase();
        if shortcut.is_empty() {
            return Err(ToolError::InvalidParameters("shortcut string is required".to_string()));
        }

        #[cfg(windows)]
        {
            let parts: Vec<&str> = shortcut.split('+').map(|p| p.trim()).collect();
            let mut pressed_keys: Vec<u8> = Vec::new();

            for part in parts {
                let vk = match part {
                    "ctrl" | "control" => win32::VK_CONTROL,
                    "alt" => win32::VK_MENU,
                    "shift" => win32::VK_SHIFT,
                    "win" | "windows" => win32::VK_LWIN,
                    "enter" | "return" => win32::VK_RETURN,
                    "esc" | "escape" => win32::VK_ESCAPE,
                    "tab" => win32::VK_TAB,
                    "backspace" => win32::VK_BACK,
                    "delete" | "del" => win32::VK_DELETE,
                    "space" => win32::VK_SPACE,
                    "up" => win32::VK_UP,
                    "down" => win32::VK_DOWN,
                    "left" => win32::VK_LEFT,
                    "right" => win32::VK_RIGHT,
                    single if single.len() == 1 => {
                        let c = single.chars().next().unwrap().to_ascii_uppercase();
                        c as u8
                    }
                    _ => 0,
                };

                if vk != 0 {
                    unsafe {
                        win32::keybd_event(vk, 0, 0, 0);
                    }
                    pressed_keys.push(vk);
                }
            }

            tokio::time::sleep(Duration::from_millis(50)).await;

            // Release in reverse order
            for &vk in pressed_keys.iter().rev() {
                unsafe {
                    win32::keybd_event(vk, 0, win32::KEYEVENTF_KEYUP, 0);
                }
            }
        }

        let msg = format!("Pressed shortcut: {shortcut}");
        Ok(ToolExecutionResult {
            tool_call_id: call_id.to_string(),
            tool_name: "desktop_shortcut".to_string(),
            stdout: msg,
            stderr: String::new(),
            exit_code: Some(0),
            duration_ms: start.elapsed().as_millis() as u64,
            is_error: false,
            truncated: false,
        })
    }
}

// ---------------------------------------------------------------------------
// 6. DesktopAppTool (App / desktop_app / Launch)
// ---------------------------------------------------------------------------
pub struct DesktopAppTool {
    security: SharedSecurityEngine,
}

impl DesktopAppTool {
    pub fn new(security: SharedSecurityEngine) -> Self {
        Self { security }
    }
}

#[async_trait]
impl Tool for DesktopAppTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "desktop_app".to_string(),
            description: "[Windows-MCP] Launch or switch applications and windows. Modes: 'launch' (opens app by name or path), 'switch' (focuses window by title), 'launch_executable' (runs executable directly with arguments).".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "mode": {
                        "type": "string",
                        "enum": ["launch", "switch", "resize", "launch_executable"],
                        "default": "launch"
                    },
                    "name": {
                        "type": "string",
                        "description": "Application name or window title"
                    },
                    "executable": {
                        "type": "string",
                        "description": "Exact executable path for launch_executable"
                    },
                    "args": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Arguments for launch_executable"
                    },
                    "cwd": {
                        "type": "string",
                        "description": "Working directory"
                    }
                }
            }),
        }
    }

    async fn execute(&self, call_id: &str, params: serde_json::Value) -> ToolResult<ToolExecutionResult> {
        let start = Instant::now();
        self.security
            .check_permission(
                PermissionTier::ShellExecution,
                "Desktop application launch / window management",
                None,
                params.clone(),
            )
            .await?;

        let mode = params.get("mode").and_then(|v| v.as_str()).unwrap_or("launch");
        let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");

        match mode {
            "switch" => {
                let adapter = WindowsComputerControlAdapter::new();
                let switched = adapter
                    .switch_window(name)
                    .await
                    .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

                let msg = if switched {
                    format!("Switched to window '{name}'.")
                } else {
                    format!("Window '{name}' not found or could not be activated.")
                };

                Ok(ToolExecutionResult {
                    tool_call_id: call_id.to_string(),
                    tool_name: "desktop_app".to_string(),
                    stdout: msg,
                    stderr: String::new(),
                    exit_code: Some(if switched { 0 } else { 1 }),
                    duration_ms: start.elapsed().as_millis() as u64,
                    is_error: false,
                    truncated: false,
                })
            }
            "launch_executable" => {
                let exe = params.get("executable").and_then(|v| v.as_str()).unwrap_or(name);
                if exe.is_empty() {
                    return Err(ToolError::InvalidParameters("executable path is required".to_string()));
                }

                let mut cmd = hidden_tokio_command(exe);
                if let Some(args_arr) = params.get("args").and_then(|v| v.as_array()) {
                    for arg in args_arr {
                        if let Some(s) = arg.as_str() {
                            cmd.arg(s);
                        }
                    }
                }
                if let Some(cwd) = params.get("cwd").and_then(|v| v.as_str()) {
                    cmd.current_dir(cwd);
                }

                let child = cmd
                    .spawn()
                    .map_err(|e| ToolError::ExecutionFailed(format!("Failed to spawn {exe}: {e}")))?;

                let pid = child.id().unwrap_or(0);
                Ok(ToolExecutionResult {
                    tool_call_id: call_id.to_string(),
                    tool_name: "desktop_app".to_string(),
                    stdout: format!("Launched executable '{exe}' (PID: {pid})."),
                    stderr: String::new(),
                    exit_code: Some(0),
                    duration_ms: start.elapsed().as_millis() as u64,
                    is_error: false,
                    truncated: false,
                })
            }
            _ => {
                // "launch" mode
                let app_to_launch = if name.is_empty() {
                    params.get("executable").and_then(|v| v.as_str()).unwrap_or("")
                } else {
                    name
                };

                if app_to_launch.is_empty() {
                    return Err(ToolError::InvalidParameters("Application name or executable path is required".to_string()));
                }

                // Smartly resolve application path across Windows
                let target_path = if let Some(resolved) = resolve_windows_app(app_to_launch) {
                    resolved.to_string_lossy().to_string()
                } else {
                    app_to_launch.to_string()
                };

                // Native Windows ShellExecute: launches .exe, shortcuts (.lnk), or apps directly in foreground
                #[cfg(windows)]
                {
                    use std::ffi::OsStr;
                    use std::os::windows::ffi::OsStrExt;

                    let wide_file: Vec<u16> = OsStr::new(&target_path)
                        .encode_wide()
                        .chain(std::iter::once(0))
                        .collect();
                    let wide_op: Vec<u16> = OsStr::new("open")
                        .encode_wide()
                        .chain(std::iter::once(0))
                        .collect();

                    let parent_buf = PathBuf::from(&target_path);
                    let working_dir: Option<Vec<u16>> = parent_buf.parent().and_then(|p| {
                        if p.exists() {
                            Some(p.as_os_str().encode_wide().chain(std::iter::once(0)).collect())
                        } else {
                            None
                        }
                    });
                    let dir_ptr = working_dir.as_ref().map(|v| v.as_ptr()).unwrap_or(std::ptr::null());

                    let ret = unsafe {
                        win32::ShellExecuteW(
                            0,
                            wide_op.as_ptr(),
                            wide_file.as_ptr(),
                            std::ptr::null(),
                            dir_ptr,
                            1, // SW_SHOWNORMAL
                        )
                    };

                    if ret > 32 {
                        return Ok(ToolExecutionResult {
                            tool_call_id: call_id.to_string(),
                            tool_name: "desktop_app".to_string(),
                            stdout: format!("Successfully launched '{app_to_launch}' (Path: {target_path})."),
                            stderr: String::new(),
                            exit_code: Some(0),
                            duration_ms: start.elapsed().as_millis() as u64,
                            is_error: false,
                            truncated: false,
                        });
                    }
                }

                // Fallback to PowerShell Start-Process with WorkingDirectory for shortcuts (.lnk), UWP apps, or system commands
                let working_dir_arg = if let Some(parent) = PathBuf::from(&target_path).parent() {
                    if parent.exists() {
                        format!(" -WorkingDirectory '{}'", parent.to_string_lossy().replace('\'', "''"))
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };
                let script = format!("Start-Process '{}'{working_dir_arg}", target_path.replace('\'', "''"));
                let output = hidden_tokio_command("powershell.exe")
                    .args(["-NoProfile", "-NonInteractive", "-Command", &script])
                    .output()
                    .await
                    .map_err(|e| ToolError::ExecutionFailed(format!("Failed to launch '{app_to_launch}': {e}")))?;

                if output.status.success() {
                    Ok(ToolExecutionResult {
                        tool_call_id: call_id.to_string(),
                        tool_name: "desktop_app".to_string(),
                        stdout: format!("Successfully launched '{app_to_launch}' (Path: {target_path})."),
                        stderr: String::new(),
                        exit_code: Some(0),
                        duration_ms: start.elapsed().as_millis() as u64,
                        is_error: false,
                        truncated: false,
                    })
                } else {
                    let err = String::from_utf8_lossy(&output.stderr).to_string();
                    Err(ToolError::ExecutionFailed(format!("Launch failed: {err}")))
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 7. DesktopClipboardTool (Clipboard / desktop_clipboard)
// ---------------------------------------------------------------------------
pub struct DesktopClipboardTool {
    security: SharedSecurityEngine,
}

impl DesktopClipboardTool {
    pub fn new(security: SharedSecurityEngine) -> Self {
        Self { security }
    }
}

#[async_trait]
impl Tool for DesktopClipboardTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "desktop_clipboard".to_string(),
            description: "[Windows-MCP] Read or write the Windows clipboard. Use mode='get' to read clipboard text, mode='set' to write text.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "mode": {
                        "type": "string",
                        "enum": ["get", "set"],
                        "description": "Operation mode: 'get' or 'set'"
                    },
                    "text": {
                        "type": "string",
                        "description": "Text to set (required if mode is 'set')"
                    }
                },
                "required": ["mode"]
            }),
        }
    }

    async fn execute(&self, call_id: &str, params: serde_json::Value) -> ToolResult<ToolExecutionResult> {
        let start = Instant::now();
        let mode = params.get("mode").and_then(|v| v.as_str()).unwrap_or("get");

        if mode == "set" {
            self.security
                .check_permission(
                    PermissionTier::ShellExecution,
                    "Set Windows clipboard content",
                    None,
                    params.clone(),
                )
                .await?;

            let text = params.get("text").and_then(|v| v.as_str()).ok_or_else(|| {
                ToolError::InvalidParameters("text parameter is required for set mode".to_string())
            })?;

            let escaped = text.replace('\'', "''");
            let script = format!("Set-Clipboard -Value '{}'", escaped);
            let output = hidden_tokio_command("powershell.exe")
                .args(["-NoProfile", "-NonInteractive", "-Command", &script])
                .output()
                .await
                .map_err(|e| ToolError::ExecutionFailed(format!("Failed to set clipboard: {e}")))?;

            if output.status.success() {
                let preview = if text.len() > 60 { format!("{}...", &text[..60]) } else { text.to_string() };
                Ok(ToolExecutionResult {
                    tool_call_id: call_id.to_string(),
                    tool_name: "desktop_clipboard".to_string(),
                    stdout: format!("Clipboard set to: {preview}"),
                    stderr: String::new(),
                    exit_code: Some(0),
                    duration_ms: start.elapsed().as_millis() as u64,
                    is_error: false,
                    truncated: false,
                })
            } else {
                Err(ToolError::ExecutionFailed(String::from_utf8_lossy(&output.stderr).to_string()))
            }
        } else {
            // mode == "get"
            self.security
                .check_permission(
                    PermissionTier::ReadOnly,
                    "Read Windows clipboard content",
                    None,
                    params.clone(),
                )
                .await?;

            let output = hidden_tokio_command("powershell.exe")
                .args(["-NoProfile", "-NonInteractive", "-Command", "Get-Clipboard"])
                .output()
                .await
                .map_err(|e| ToolError::ExecutionFailed(format!("Failed to get clipboard: {e}")))?;

            let content = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(ToolExecutionResult {
                tool_call_id: call_id.to_string(),
                tool_name: "desktop_clipboard".to_string(),
                stdout: if content.is_empty() { "Clipboard is empty.".to_string() } else { content },
                stderr: String::new(),
                exit_code: Some(0),
                duration_ms: start.elapsed().as_millis() as u64,
                is_error: false,
                truncated: false,
            })
        }
    }
}

// ---------------------------------------------------------------------------
// 8. DesktopSnapshotTool (Snapshot / State / desktop_snapshot)
// ---------------------------------------------------------------------------
pub struct DesktopSnapshotTool {
    security: SharedSecurityEngine,
}

impl DesktopSnapshotTool {
    pub fn new(security: SharedSecurityEngine) -> Self {
        Self { security }
    }
}

#[async_trait]
impl Tool for DesktopSnapshotTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "desktop_snapshot".to_string(),
            description: "[Windows-MCP] Inspect current desktop state: screen resolution, active window, open window list, desktop lock status, and cursor coordinates.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "use_vision": {
                        "type": "boolean",
                        "default": false,
                        "description": "Whether to also capture a screenshot"
                    }
                }
            }),
        }
    }

    async fn execute(&self, call_id: &str, params: serde_json::Value) -> ToolResult<ToolExecutionResult> {
        let start = Instant::now();
        self.security
            .check_permission(
                PermissionTier::ReadOnly,
                "Inspect desktop state and open windows",
                None,
                params.clone(),
            )
            .await?;

        let adapter = WindowsComputerControlAdapter::new();
        let mut state = adapter
            .get_desktop_state()
            .await
            .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        #[cfg(windows)]
        {
            let mut pt = win32::POINT { x: 0, y: 0 };
            unsafe {
                win32::GetCursorPos(&mut pt);
            }
            let screen_w = unsafe { win32::GetSystemMetrics(win32::SM_CXSCREEN) };
            let screen_h = unsafe { win32::GetSystemMetrics(win32::SM_CYSCREEN) };

            if let Some(obj) = state.as_object_mut() {
                obj.insert("cursor_position".to_string(), json!([pt.x, pt.y]));
                obj.insert("screen_resolution".to_string(), json!({"width": screen_w, "height": screen_h}));
            }
        }

        let formatted = serde_json::to_string_pretty(&state).unwrap_or_default();
        Ok(ToolExecutionResult {
            tool_call_id: call_id.to_string(),
            tool_name: "desktop_snapshot".to_string(),
            stdout: formatted,
            stderr: String::new(),
            exit_code: Some(0),
            duration_ms: start.elapsed().as_millis() as u64,
            is_error: false,
            truncated: false,
        })
    }
}

// ---------------------------------------------------------------------------
// 9. DesktopScreenshotTool (Screenshot / desktop_screenshot)
// ---------------------------------------------------------------------------
pub struct DesktopScreenshotTool {
    security: SharedSecurityEngine,
}

impl DesktopScreenshotTool {
    pub fn new(security: SharedSecurityEngine) -> Self {
        Self { security }
    }
}

#[async_trait]
impl Tool for DesktopScreenshotTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "desktop_screenshot".to_string(),
            description: "[Windows-MCP] Captures a screenshot of the primary display, saving to disk and returning image metadata.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "save_path": {
                        "type": "string",
                        "description": "Optional file path where PNG screenshot should be saved"
                    }
                }
            }),
        }
    }

    async fn execute(&self, call_id: &str, params: serde_json::Value) -> ToolResult<ToolExecutionResult> {
        let start = Instant::now();
        self.security
            .check_permission(
                PermissionTier::ReadOnly,
                "Capture desktop screenshot",
                None,
                params.clone(),
            )
            .await?;

        let adapter = WindowsComputerControlAdapter::new();
        let raw_bytes = adapter
            .take_screenshot()
            .await
            .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        // Determine destination file path
        let save_path = if let Some(p) = params.get("save_path").and_then(|v| v.as_str()) {
            PathBuf::from(p)
        } else {
            let temp_dir = std::env::temp_dir();
            let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
            let filename = format!("screenshot_{ts}.png");
            temp_dir.join(filename)
        };

        // If raw_bytes is base64 encoded string from powershell, decode it
        let b64_str = String::from_utf8_lossy(&raw_bytes).trim().to_string();
        let final_bytes = if let Ok(decoded) = standard_base64_decode(&b64_str) {
            decoded
        } else {
            raw_bytes
        };

        std::fs::write(&save_path, &final_bytes)
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to write screenshot file: {e}")))?;

        let size_kb = final_bytes.len() / 1024;
        let msg = format!(
            "Captured desktop screenshot: {} ({} KB)",
            save_path.display(),
            size_kb
        );

        Ok(ToolExecutionResult {
            tool_call_id: call_id.to_string(),
            tool_name: "desktop_screenshot".to_string(),
            stdout: msg,
            stderr: String::new(),
            exit_code: Some(0),
            duration_ms: start.elapsed().as_millis() as u64,
            is_error: false,
            truncated: false,
        })
    }
}

// ---------------------------------------------------------------------------
// 10. DesktopSwitchWindowTool (SwitchWindow / desktop_switch_window)
// ---------------------------------------------------------------------------
pub struct DesktopSwitchWindowTool {
    security: SharedSecurityEngine,
}

impl DesktopSwitchWindowTool {
    pub fn new(security: SharedSecurityEngine) -> Self {
        Self { security }
    }
}

#[async_trait]
impl Tool for DesktopSwitchWindowTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "desktop_switch_window".to_string(),
            description: "[Windows-MCP] Brings a specific window into focus by matching its title.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "title": {
                        "type": "string",
                        "description": "Window title to activate"
                    }
                },
                "required": ["title"]
            }),
        }
    }

    async fn execute(&self, call_id: &str, params: serde_json::Value) -> ToolResult<ToolExecutionResult> {
        let start = Instant::now();
        self.security
            .check_permission(
                PermissionTier::ShellExecution,
                "Switch active desktop window",
                None,
                params.clone(),
            )
            .await?;

        let title = params.get("title").and_then(|v| v.as_str()).ok_or_else(|| {
            ToolError::InvalidParameters("title parameter is required".to_string())
        })?;

        let adapter = WindowsComputerControlAdapter::new();
        let switched = adapter
            .switch_window(title)
            .await
            .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        let msg = if switched {
            format!("Activated window '{title}'.")
        } else {
            format!("Window '{title}' could not be activated.")
        };

        Ok(ToolExecutionResult {
            tool_call_id: call_id.to_string(),
            tool_name: "desktop_switch_window".to_string(),
            stdout: msg,
            stderr: String::new(),
            exit_code: Some(if switched { 0 } else { 1 }),
            duration_ms: start.elapsed().as_millis() as u64,
            is_error: false,
            truncated: false,
        })
    }
}

fn standard_base64_decode(input: &str) -> Result<Vec<u8>, String> {
    // Basic standard base64 decoding helper without external heavy crates
    const B64_TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut clean: Vec<u8> = input.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
    while clean.len() % 4 != 0 {
        clean.push(b'=');
    }

    let mut out = Vec::new();
    for chunk in clean.chunks(4) {
        if chunk.len() < 4 { break; }
        let mut buf = 0u32;
        let mut pad_count = 0;
        for (i, &b) in chunk.iter().enumerate() {
            if b == b'=' {
                pad_count += 1;
            } else if let Some(idx) = B64_TABLE.iter().position(|&c| c == b) {
                buf |= (idx as u32) << (18 - i * 6);
            } else {
                return Err("Invalid base64 character".to_string());
            }
        }
        out.push(((buf >> 16) & 0xFF) as u8);
        if pad_count < 2 {
            out.push(((buf >> 8) & 0xFF) as u8);
        }
        if pad_count == 0 {
            out.push((buf & 0xFF) as u8);
        }
    }
    Ok(out)
}
