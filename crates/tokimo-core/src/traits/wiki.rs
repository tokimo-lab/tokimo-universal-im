use async_trait::async_trait;
use crate::error::ImResult;
use crate::types::{
    WikiSpace, WikiNode, Page,
    ListWikiSpacesRequest, ListWikiNodesRequest,
    CreateWikiNodeRequest, MoveWikiNodeRequest, SearchWikiRequest,
};

/// Wiki / knowledge base management.
#[async_trait]
pub trait WikiService: Send + Sync {
    /// List wiki spaces.
    async fn list_spaces(&self, req: ListWikiSpacesRequest) -> ImResult<Page<WikiSpace>>;

    /// Get a wiki space by ID.
    async fn get_space(&self, space_id: &str) -> ImResult<WikiSpace>;

    /// List nodes (pages) in a wiki space.
    async fn list_nodes(&self, req: ListWikiNodesRequest) -> ImResult<Page<WikiNode>>;

    /// Get a single wiki node.
    async fn get_node(&self, space_id: &str, node_id: &str) -> ImResult<WikiNode>;

    /// Create a new wiki node (page/doc).
    async fn create_node(&self, req: CreateWikiNodeRequest) -> ImResult<WikiNode>;

    /// Move a wiki node.
    async fn move_node(&self, req: MoveWikiNodeRequest) -> ImResult<()>;

    /// Search wiki content.
    async fn search(&self, req: SearchWikiRequest) -> ImResult<Page<WikiNode>>;
}
