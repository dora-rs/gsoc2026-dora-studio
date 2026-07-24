use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    pub error: String,
}

// --- Coordinator ---

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoordinatorStatus {
    pub connected: bool,
    pub version: String,
    pub running_dataflows: u32,
    pub active_nodes: u32,
    pub dataflows: Vec<CoordinatorDataflow>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoordinatorDataflow {
    pub id: String,
    pub name: String,
    pub status: String,
    pub nodes: u32,
}

// --- dviz ---

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DvizStatus {
    pub installed: bool,
    pub running: bool,
    pub binary_path: Option<String>,
    pub message: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DvizTopicsResponse {
    pub source: String,
    pub message: String,
    pub topics: Vec<DvizTopic>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DvizTopic {
    pub name: String,
    pub data_type: String,
    pub source: String,
    pub status: String,
    pub message_rate_hz: f32,
    pub last_seen: String,
    pub summary: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DvizDisplaysResponse {
    pub source: String,
    pub message: String,
    pub displays: Vec<DvizDisplay>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DvizDisplay {
    pub id: String,
    pub name: String,
    pub data_type: String,
    pub enabled: bool,
    pub source_topic: Option<String>,
    pub status: String,
    pub summary: String,
    pub color: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DvizSnapshotResponse {
    pub source: String,
    pub message: String,
    pub status: DvizStatus,
    pub summary: DvizSnapshotSummary,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DvizSnapshotSummary {
    pub topic_count: usize,
    pub ready_topic_count: usize,
    pub idle_topic_count: usize,
    pub display_count: usize,
    pub enabled_display_count: usize,
}

// --- robot profile ---

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RobotProfileResponse {
    pub source: String,
    pub message: String,
    pub profile: RobotProfile,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RobotProfile {
    pub id: String,
    pub name: String,
    pub family: String,
    pub summary: String,
    pub simulation_owner: String,
    pub viewport_role: String,
    pub modules: Vec<RobotModule>,
    pub workflows: Vec<RobotWorkflow>,
    pub visualization_displays: Vec<String>,
    pub planning_capabilities: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RobotModule {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub role: String,
    pub transport: String,
    pub frame: String,
    pub status: String,
    pub summary: String,
    pub required: bool,
    pub source_topics: Vec<String>,
    pub linked_displays: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RobotWorkflow {
    pub id: String,
    pub name: String,
    pub status: String,
    pub owner: String,
    pub summary: String,
}

// --- dora-moveit2 ---

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveitStatus {
    pub installed: bool,
    pub running: bool,
    pub message: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeState {
    pub status: String,
    pub pid: Option<u32>,
    pub last_message: String,
    pub dataflow_id: Option<String>,
    pub dataflow_path: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemStatus {
    pub coordinator: String,
    pub daemon: String,
    pub version: String,
    pub running_dataflows: u32,
    pub active_nodes: u32,
    pub error_count: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DataflowSummary {
    pub id: String,
    pub name: String,
    pub status: String,
    pub node_count: u32,
    pub edge_count: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DataflowDefinition {
    pub id: String,
    pub name: String,
    pub relative_path: String,
    pub source: String,
    pub node_count: u32,
    pub edge_count: u32,
    pub nodes: Vec<DataflowDefinitionNode>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DataflowDefinitionNode {
    pub id: String,
    pub path: Option<String>,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeMetrics {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub status: String,
    pub cpu: u32,
    pub memory: u32,
    pub restarts: u32,
    pub pending: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    pub time: String,
    pub timestamp: String,
    pub node: String,
    pub level: String,
    pub message: String,
    pub raw_message: String,
    pub source: String,
    pub source_file: Option<String>,
    pub source_line: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub status: String,
    pub x: u32,
    pub y: u32,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub cpu: u32,
    pub memory: u32,
    pub restarts: u32,
    pub pending: u32,
    pub note: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphEdge {
    pub id: String,
    pub from: String,
    pub to: String,
    pub label: String,
}

#[derive(Serialize)]
pub struct Diagnostic {
    pub severity: String,
    pub message: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DataflowGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub diagnostics: Vec<Diagnostic>,
}
