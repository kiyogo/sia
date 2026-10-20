# Windows Computer Control Subsystem
**Universal Autonomous AI Workspace — Phase 3**
**Subsystem:** Window Management, Input Simulation, and Desktop UI Automation

---

## 1. Architectural Role
The Computer Control Subsystem enables the AI agent to interact with native Windows applications (Notepad, File Explorer, Command Prompts, custom desktop applications, and settings dialogs) through structured UI Automation and input simulation.

It operates primarily through **Windows-MCP** augmented by native Rust supervisory routines.

---

## 2. Window & Application Management

### 2.1 Window Discovery & Switching
The agent can inspect open top-level windows:
- Retrieves window title, process name, window ID, and bounding rectangle `[x, y, width, height]`.
- Switches focus using `windows_mcp_App` in mode `"switch"`, bringing the target application to the foreground.
- Verifies that `GetForegroundWindow` matches the expected window title before issuing subsequent inputs.

### 2.2 Launching Applications
- Launches applications via `windows_mcp_App` in mode `"launch"`.
- Waits for the application process to spawn and render its primary top-level window.
- Handles standard Windows utilities like `notepad.exe`, `calc.exe`, and developer tools.

---

## 3. UI Automation & Accessibility Tree Inspection

Rather than blind screen scraping, the subsystem uses the Windows UI Automation (UIA) tree:
- **`Snapshot` Tool**: Walks the element hierarchy, extracting control types (`Button`, `Edit`, `MenuItem`, `ComboBox`), accessible names, values, and bounding coordinates.
- **Element Bounding**: Allows precise clicks directly on the center point of target elements or referencing elements by their unique UI ID.
- **Pruning**: Excludes invisible or disabled elements to save context tokens.

---

## 4. Input Simulation & Determinism

### 4.1 Mouse Actions
- **Click**: Dispatches Left, Right, or Double-clicks at coordinates `[x, y]` or element label.
- **Move & Drag**: Smooth cursor movements and deterministic drag-and-drop actions.
- **Scroll**: Vertical and horizontal mouse wheel scrolling with configurable wheel increments.

### 4.2 Keyboard Actions
- **Type**: Unicode text typing into focused input controls with optional prior field clearing (`clear=true`) and enter dispatch (`press_enter=true`).
- **Shortcuts**: Key combinations (`ctrl+s`, `ctrl+c`, `ctrl+v`, `alt+f4`, `win+r`).

---

## 5. Post-Condition Verification
Every computer action must verify its outcome:
- **Window Switch Verification**: Checks that active foreground window matches target.
- **File Save Verification**: Inspects target file on disk after save shortcut/dialog to ensure data is written.
- **Text Insertion Verification**: Re-reads field value or UI tree node.
