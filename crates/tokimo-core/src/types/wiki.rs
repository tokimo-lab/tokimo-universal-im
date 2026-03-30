use serde::{Deserialize, Serialize};

/// A wiki space / knowledge base.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiSpace {
    /// Space ID.
    pub id: String,
    /// Space name.
    pub name: String,
    /// Description.
    pub description: Option<String>,
    /// Visibility level.
    pub visibility: Option<String>,
}

/// A wiki node (page/document within a wiki space).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiNode {
    /// Node token / ID.
    pub id: String,
    /// Parent node ID (None for root).
    pub parent_id: Option<String>,
    /// Title.
    pub title: String,
    /// Node type (e.g., "doc", "sheet", "folder").
    pub node_type: String,
    /// Whether this node has children.
    pub has_child: bool,
    /// URL to access.
    pub url: Option<String>,
    /// Creator user ID.
    pub creator: Option<String>,
    /// Creation time.
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Last edit time.
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Request to list wiki spaces.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListWikiSpacesRequest {
    pub cursor: Option<String>,
    pub limit: Option<u32>,
}

/// Request to list nodes in a wiki space.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListWikiNodesRequest {
    /// Space ID.
    pub space_id: String,
    /// Parent node token (None for root level).
    pub parent_node_id: Option<String>,
    pub cursor: Option<String>,
    pub limit: Option<u32>,
}

/// Request to create a wiki node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWikiNodeRequest {
    /// Space ID.
    pub space_id: String,
    /// Parent node token (None for root).
    pub parent_node_id: Option<String>,
    /// Node type.
    pub node_type: String,
    /// Title.
    pub title: String,
    /// Initial content (optional).
    pub content: Option<String>,
}

/// Request to move a wiki node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveWikiNodeRequest {
    /// Space ID.
    pub space_id: String,
    /// Node to move.
    pub node_id: String,
    /// Target parent node.
    pub target_parent_id: Option<String>,
}

/// Request to search wiki.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchWikiRequest {
    /// Query string.
    pub query: String,
    /// Space ID to search in (None for all).
    pub space_id: Option<String>,
    pub cursor: Option<String>,
    pub limit: Option<u32>,
}
