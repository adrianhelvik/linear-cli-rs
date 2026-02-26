use serde::Deserialize;

// Generic GraphQL response wrapper
#[derive(Debug, Deserialize)]
pub struct GraphQLResponse<T> {
    pub data: Option<T>,
    pub errors: Option<Vec<GraphQLError>>,
}

#[derive(Debug, Deserialize)]
pub struct GraphQLError {
    pub message: String,
}

// Connection types
#[derive(Debug, Clone, Deserialize)]
pub struct Connection<T> {
    pub nodes: Vec<T>,
    #[serde(rename = "pageInfo")]
    pub page_info: Option<PageInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PageInfo {
    #[serde(rename = "hasNextPage")]
    pub has_next_page: bool,
    #[serde(rename = "endCursor")]
    pub end_cursor: Option<String>,
}

// Domain types
#[derive(Debug, Clone, Deserialize)]
pub struct User {
    pub id: String,
    pub name: Option<String>,
    pub email: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    pub active: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Team {
    pub id: String,
    pub key: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkflowState {
    pub id: String,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub state_type: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Label {
    pub id: String,
    pub name: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Issue {
    pub id: String,
    pub identifier: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub priority: Option<i32>,
    pub estimate: Option<f64>,
    pub url: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<String>,
    pub state: Option<WorkflowState>,
    pub team: Option<Team>,
    pub assignee: Option<User>,
    pub labels: Option<Connection<Label>>,
    pub project: Option<Project>,
}

// Query response wrappers
#[derive(Debug, Deserialize)]
pub struct ViewerResponse {
    pub viewer: User,
}

#[derive(Debug, Deserialize)]
pub struct TeamsResponse {
    pub teams: Connection<Team>,
}

#[derive(Debug, Deserialize)]
pub struct IssuesResponse {
    pub issues: Connection<Issue>,
}

#[derive(Debug, Deserialize)]
pub struct IssueResponse {
    pub issue: Issue,
}

#[derive(Debug, Deserialize)]
pub struct IssueSearchResponse {
    #[serde(rename = "issueSearch")]
    pub issue_search: Connection<Issue>,
}

#[derive(Debug, Deserialize)]
pub struct WorkflowStatesResponse {
    #[serde(rename = "workflowStates")]
    pub workflow_states: Connection<WorkflowState>,
}

#[derive(Debug, Deserialize)]
pub struct UsersResponse {
    pub users: Connection<User>,
}

#[derive(Debug, Deserialize)]
pub struct LabelsResponse {
    #[serde(rename = "issueLabels")]
    pub issue_labels: Connection<Label>,
}

// Mutation responses
#[derive(Debug, Deserialize)]
pub struct MutationResult {
    pub success: bool,
    pub issue: Option<Issue>,
}

#[derive(Debug, Deserialize)]
pub struct IssueCreateResponse {
    #[serde(rename = "issueCreate")]
    pub issue_create: MutationResult,
}

#[derive(Debug, Deserialize)]
pub struct IssueUpdateResponse {
    #[serde(rename = "issueUpdate")]
    pub issue_update: MutationResult,
}
