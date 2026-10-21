use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AgentType {
    Supervisor,
    Coding,
    Browser,
    Computer,
    Research,
    Document,
    DataAnalysis,
    Email,
    Business,
    Custom(String),
}

impl std::fmt::Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentType::Supervisor => write!(f, "Supervisor"),
            AgentType::Coding => write!(f, "Coding"),
            AgentType::Browser => write!(f, "Browser"),
            AgentType::Computer => write!(f, "Computer"),
            AgentType::Research => write!(f, "Research"),
            AgentType::Document => write!(f, "Document"),
            AgentType::DataAnalysis => write!(f, "DataAnalysis"),
            AgentType::Email => write!(f, "Email"),
            AgentType::Business => write!(f, "Business"),
            AgentType::Custom(name) => write!(f, "Custom({})", name),
        }
    }
}

impl std::str::FromStr for AgentType {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Supervisor" => Ok(AgentType::Supervisor),
            "Coding" => Ok(AgentType::Coding),
            "Browser" => Ok(AgentType::Browser),
            "Computer" => Ok(AgentType::Computer),
            "Research" => Ok(AgentType::Research),
            "Document" => Ok(AgentType::Document),
            "DataAnalysis" => Ok(AgentType::DataAnalysis),
            "Email" => Ok(AgentType::Email),
            "Business" => Ok(AgentType::Business),
            other => Ok(AgentType::Custom(other.to_string())),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AgentCapability {
    TaskDecomposition,
    ResultVerification,
    CodeAnalysis,
    CodeEditing,
    GitOperations,
    BuildAndTest,
    BrowserNavigation,
    DomInspection,
    WebFormInteraction,
    DesktopWindowControl,
    DesktopInput,
    WebSearch,
    DocumentSummarization,
    DocumentParsing,
    DocumentGeneration,
    DataAnalysis,
    ChartGeneration,
    EmailReading,
    EmailDrafting,
    CalendarManagement,
    BusinessCoordination,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDefinition {
    pub id: String,
    pub agent_type: AgentType,
    pub display_name: String,
    pub capabilities: Vec<AgentCapability>,
    pub supported_tools: Vec<String>,
    pub required_permissions: Vec<String>,
    pub max_concurrency: usize,
    pub is_enabled: bool,
    pub model_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AgentRegistry {
    agents: HashMap<String, AgentDefinition>,
}

impl Default for AgentRegistry {
    fn default() -> Self {
        let mut reg = Self {
            agents: HashMap::new(),
        };
        reg.register_default_agents();
        reg
    }
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_default_agents(&mut self) {
        // 1. Supervisor Agent
        self.register_agent(AgentDefinition {
            id: "agent_supervisor".to_string(),
            agent_type: AgentType::Supervisor,
            display_name: "Supervisor AI Planner".to_string(),
            capabilities: vec![
                AgentCapability::TaskDecomposition,
                AgentCapability::ResultVerification,
                AgentCapability::DocumentSummarization,
            ],
            supported_tools: vec![
                "plan_decompose".to_string(),
                "plan_verify".to_string(),
                "plan_replan".to_string(),
            ],
            required_permissions: vec!["supervisor".to_string()],
            max_concurrency: 1,
            is_enabled: true,
            model_name: None,
        });

        // 2. Coding Agent (OpenCode Engine)
        self.register_agent(AgentDefinition {
            id: "agent_coding".to_string(),
            agent_type: AgentType::Coding,
            display_name: "OpenCode Autonomous Coder".to_string(),
            capabilities: vec![
                AgentCapability::CodeAnalysis,
                AgentCapability::CodeEditing,
                AgentCapability::GitOperations,
                AgentCapability::BuildAndTest,
            ],
            supported_tools: vec![
                "code_edit".to_string(),
                "code_patch".to_string(),
                "repo_map".to_string(),
                "code_search".to_string(),
                "file_outline".to_string(),
                "build_runner".to_string(),
                "test_runner".to_string(),
                "git_status".to_string(),
                "git_diff".to_string(),
                "git_commit".to_string(),
            ],
            required_permissions: vec!["workspace_write".to_string(), "terminal_exec".to_string()],
            max_concurrency: 2,
            is_enabled: true,
            model_name: None,
        });

        // 3. Browser Agent (Playwright MCP)
        self.register_agent(AgentDefinition {
            id: "agent_browser".to_string(),
            agent_type: AgentType::Browser,
            display_name: "Playwright Web Navigator".to_string(),
            capabilities: vec![
                AgentCapability::BrowserNavigation,
                AgentCapability::DomInspection,
                AgentCapability::WebFormInteraction,
            ],
            supported_tools: vec![
                "browser_navigate".to_string(),
                "browser_snapshot".to_string(),
                "browser_click".to_string(),
                "browser_fill".to_string(),
                "browser_download".to_string(),
                "browser_upload".to_string(),
                "browser_tabs".to_string(),
            ],
            required_permissions: vec!["browser_automation".to_string()],
            max_concurrency: 2,
            is_enabled: true,
            model_name: None,
        });

        // 4. Computer Agent (Windows-MCP / Win32)
        self.register_agent(AgentDefinition {
            id: "agent_computer".to_string(),
            agent_type: AgentType::Computer,
            display_name: "Windows Desktop Controller".to_string(),
            capabilities: vec![
                AgentCapability::DesktopWindowControl,
                AgentCapability::DesktopInput,
            ],
            supported_tools: vec![
                "desktop_list_windows".to_string(),
                "desktop_switch_window".to_string(),
                "desktop_capture".to_string(),
                "desktop_uia_tree".to_string(),
                "desktop_mouse_click".to_string(),
                "desktop_key_input".to_string(),
            ],
            required_permissions: vec!["computer_control".to_string()],
            max_concurrency: 1, // Single interactive desktop controller
            is_enabled: true,
            model_name: None,
        });

        // 5. Research Worker
        self.register_agent(AgentDefinition {
            id: "agent_research".to_string(),
            agent_type: AgentType::Research,
            display_name: "Research & Documentation Specialist".to_string(),
            capabilities: vec![
                AgentCapability::WebSearch,
                AgentCapability::DocumentSummarization,
                AgentCapability::CodeAnalysis,
            ],
            supported_tools: vec![
                "web_search".to_string(),
                "doc_summarize".to_string(),
                "code_search".to_string(),
            ],
            required_permissions: vec!["network_read".to_string()],
            max_concurrency: 3,
            is_enabled: true,
            model_name: None,
        });

        // 6. Document Specialist
        self.register_agent(AgentDefinition {
            id: "agent_document".to_string(),
            agent_type: AgentType::Document,
            display_name: "Universal Document Specialist".to_string(),
            capabilities: vec![
                AgentCapability::DocumentParsing,
                AgentCapability::DocumentGeneration,
                AgentCapability::DocumentSummarization,
            ],
            supported_tools: vec![
                "doc_parse".to_string(),
                "doc_generate_docx".to_string(),
                "doc_generate_pdf".to_string(),
                "doc_chunk".to_string(),
            ],
            required_permissions: vec!["workspace_write".to_string()],
            max_concurrency: 2,
            is_enabled: true,
            model_name: None,
        });

        // 7. Data Analysis Agent
        self.register_agent(AgentDefinition {
            id: "agent_data".to_string(),
            agent_type: AgentType::DataAnalysis,
            display_name: "Tabular Data Analyst".to_string(),
            capabilities: vec![
                AgentCapability::DataAnalysis,
                AgentCapability::ChartGeneration,
            ],
            supported_tools: vec![
                "data_summarize".to_string(),
                "data_group_by".to_string(),
                "data_outliers".to_string(),
                "chart_bar_svg".to_string(),
                "chart_line_svg".to_string(),
            ],
            required_permissions: vec!["workspace_read".to_string()],
            max_concurrency: 2,
            is_enabled: true,
            model_name: None,
        });

        // 8. Email & Calendar Agent
        self.register_agent(AgentDefinition {
            id: "agent_email".to_string(),
            agent_type: AgentType::Email,
            display_name: "Enterprise Email & Calendar Agent".to_string(),
            capabilities: vec![
                AgentCapability::EmailReading,
                AgentCapability::EmailDrafting,
                AgentCapability::CalendarManagement,
            ],
            supported_tools: vec![
                "email_search".to_string(),
                "email_read".to_string(),
                "email_draft".to_string(),
                "email_send_approval".to_string(),
                "calendar_events".to_string(),
                "calendar_conflicts".to_string(),
            ],
            required_permissions: vec!["external_side_effect".to_string()],
            max_concurrency: 1,
            is_enabled: true,
            model_name: None,
        });

        // 9. Business Workflow Coordinator
        self.register_agent(AgentDefinition {
            id: "agent_business".to_string(),
            agent_type: AgentType::Business,
            display_name: "Business Workflow Coordinator".to_string(),
            capabilities: vec![
                AgentCapability::BusinessCoordination,
                AgentCapability::TaskDecomposition,
                AgentCapability::ResultVerification,
            ],
            supported_tools: vec![
                "workflow_market_research".to_string(),
                "workflow_data_reporting".to_string(),
                "workflow_email_assist".to_string(),
                "workflow_hybrid".to_string(),
            ],
            required_permissions: vec!["supervisor".to_string()],
            max_concurrency: 1,
            is_enabled: true,
            model_name: None,
        });
    }

    pub fn register_agent(&mut self, def: AgentDefinition) {
        self.agents.insert(def.id.clone(), def);
    }

    pub fn get_agent(&self, id: &str) -> Option<&AgentDefinition> {
        self.agents.get(id)
    }

    pub fn find_agent_by_type(&self, agent_type: &AgentType) -> Option<&AgentDefinition> {
        self.agents.values().find(|a| &a.agent_type == agent_type && a.is_enabled)
    }

    pub fn find_agent_for_capability(&self, cap: &AgentCapability) -> Option<&AgentDefinition> {
        self.agents
            .values()
            .find(|a| a.is_enabled && a.capabilities.contains(cap))
    }

    pub fn list_agents(&self) -> Vec<AgentDefinition> {
        let mut list: Vec<_> = self.agents.values().cloned().collect();
        list.sort_by(|a, b| a.id.cmp(&b.id));
        list
    }

    pub fn set_enabled(&mut self, id: &str, enabled: bool) -> bool {
        if let Some(agent) = self.agents.get_mut(id) {
            agent.is_enabled = enabled;
            true
        } else {
            false
        }
    }

    pub fn set_concurrency(&mut self, id: &str, limit: usize) -> bool {
        if let Some(agent) = self.agents.get_mut(id) {
            agent.max_concurrency = limit;
            true
        } else {
            false
        }
    }
}
