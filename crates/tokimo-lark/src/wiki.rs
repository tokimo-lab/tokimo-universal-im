use async_trait::async_trait;
use serde::Deserialize;
use tokimo_core::{
    WikiService, ImResult, ImError,
    WikiSpace, WikiNode, Page,
    ListWikiSpacesRequest, ListWikiNodesRequest,
    CreateWikiNodeRequest, MoveWikiNodeRequest, SearchWikiRequest,
};
use crate::client::LarkClient;

#[derive(Deserialize)]
struct LarkResp<T> {
    code: Option<i64>,
    msg: Option<String>,
    data: Option<T>,
}

// ── Space types ──

#[derive(Deserialize)]
struct SpaceListData {
    #[serde(default)]
    items: Vec<LarkSpace>,
    page_token: Option<String>,
    has_more: Option<bool>,
}

#[derive(Deserialize)]
struct SpaceData {
    space: Option<LarkSpace>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct LarkSpace {
    space_id: Option<String>,
    name: Option<String>,
    description: Option<String>,
    visibility: Option<String>,
}

impl From<LarkSpace> for WikiSpace {
    fn from(s: LarkSpace) -> Self {
        WikiSpace {
            id: s.space_id.unwrap_or_default(),
            name: s.name.unwrap_or_default(),
            description: s.description,
            visibility: s.visibility,
        }
    }
}

// ── Node types ──

#[derive(Deserialize)]
struct NodeListData {
    #[serde(default)]
    items: Vec<LarkNode>,
    page_token: Option<String>,
    has_more: Option<bool>,
}

#[derive(Deserialize)]
struct NodeData {
    node: Option<LarkNode>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct LarkNode {
    node_token: Option<String>,
    space_id: Option<String>,
    parent_node_token: Option<String>,
    title: Option<String>,
    obj_type: Option<String>,
    has_child: Option<bool>,
    url: Option<String>,
    creator: Option<String>,
    create_time: Option<String>,
    edit_time: Option<String>,
}

fn ts_opt(s: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    let ts: i64 = s.parse().ok()?;
    chrono::DateTime::from_timestamp(ts, 0)
}

impl From<LarkNode> for WikiNode {
    fn from(n: LarkNode) -> Self {
        WikiNode {
            id: n.node_token.unwrap_or_default(),
            parent_id: n.parent_node_token,
            title: n.title.unwrap_or_default(),
            node_type: n.obj_type.unwrap_or_else(|| "doc".into()),
            has_child: n.has_child.unwrap_or(false),
            url: n.url,
            creator: n.creator,
            created_at: n.create_time.as_deref().and_then(ts_opt),
            updated_at: n.edit_time.as_deref().and_then(ts_opt),
        }
    }
}

// ── Search types ──

#[derive(Deserialize)]
struct SearchData {
    #[serde(default)]
    docs_entities: Vec<SearchItem>,
    has_more: Option<bool>,
    count: Option<u32>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct SearchItem {
    docs_token: Option<String>,
    title: Option<String>,
    docs_type: Option<String>,
    url: Option<String>,
    owner_id: Option<String>,
    create_time: Option<String>,
    edit_time: Option<String>,
}

#[async_trait]
impl WikiService for LarkClient {
    async fn list_spaces(&self, req: ListWikiSpacesRequest) -> ImResult<Page<WikiSpace>> {
        let mut path = format!(
            "/open-apis/wiki/v2/spaces?page_size={}",
            req.limit.unwrap_or(20),
        );
        if let Some(ref cursor) = req.cursor {
            path.push_str(&format!("&page_token={}", cursor));
        }
        let resp = self.get(&path).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<SpaceListData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let list = data.data.unwrap_or(SpaceListData { items: vec![], page_token: None, has_more: None });
        Ok(Page {
            items: list.items.into_iter().map(Into::into).collect(),
            has_more: list.has_more.unwrap_or(false),
            next_cursor: list.page_token,
        })
    }

    async fn get_space(&self, space_id: &str) -> ImResult<WikiSpace> {
        let path = format!("/open-apis/wiki/v2/spaces/{}", space_id);
        let resp = self.get(&path).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<SpaceData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let space = data.data.and_then(|d| d.space).ok_or_else(|| ImError::NotFound {
            resource: space_id.into(),
        })?;
        Ok(space.into())
    }

    async fn list_nodes(&self, req: ListWikiNodesRequest) -> ImResult<Page<WikiNode>> {
        let mut path = format!(
            "/open-apis/wiki/v2/spaces/{}/nodes?page_size={}",
            req.space_id,
            req.limit.unwrap_or(20),
        );
        if let Some(ref cursor) = req.cursor {
            path.push_str(&format!("&page_token={}", cursor));
        }
        if let Some(ref parent) = req.parent_node_id {
            path.push_str(&format!("&parent_node_token={}", parent));
        }
        let resp = self.get(&path).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<NodeListData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let list = data.data.unwrap_or(NodeListData { items: vec![], page_token: None, has_more: None });
        Ok(Page {
            items: list.items.into_iter().map(Into::into).collect(),
            has_more: list.has_more.unwrap_or(false),
            next_cursor: list.page_token,
        })
    }

    async fn get_node(&self, space_id: &str, node_id: &str) -> ImResult<WikiNode> {
        let path = format!("/open-apis/wiki/v2/spaces/{}/nodes/{}", space_id, node_id);
        let resp = self.get(&path).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<NodeData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let node = data.data.and_then(|d| d.node).ok_or_else(|| ImError::NotFound {
            resource: node_id.into(),
        })?;
        Ok(node.into())
    }

    async fn create_node(&self, req: CreateWikiNodeRequest) -> ImResult<WikiNode> {
        let body = serde_json::json!({
            "obj_type": req.node_type,
            "title": req.title,
            "parent_node_token": req.parent_node_id.unwrap_or_default(),
        });
        let path = format!("/open-apis/wiki/v2/spaces/{}/nodes", req.space_id);
        let resp = self.post(&path, &body).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<NodeData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let node = data.data.and_then(|d| d.node).ok_or_else(|| ImError::Internal("empty node".into()))?;
        Ok(node.into())
    }

    async fn move_node(&self, req: MoveWikiNodeRequest) -> ImResult<()> {
        let body = serde_json::json!({
            "target_parent_token": req.target_parent_id.unwrap_or_default(),
        });
        let path = format!(
            "/open-apis/wiki/v2/spaces/{}/nodes/{}/move",
            req.space_id, req.node_id,
        );
        let resp = self.post(&path, &body).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<serde_json::Value> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        Ok(())
    }

    async fn search(&self, req: SearchWikiRequest) -> ImResult<Page<WikiNode>> {
        let body = serde_json::json!({
            "search_key": req.query,
            "count": req.limit.unwrap_or(20),
            "offset": req.cursor.as_deref().and_then(|s| s.parse::<u32>().ok()).unwrap_or(0),
        });
        let resp = self.post("/open-apis/suite/docs-api/search/object", &body).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<SearchData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let search = data.data.unwrap_or(SearchData {
            docs_entities: vec![], has_more: None, count: None,
        });
        let items: Vec<WikiNode> = search.docs_entities.into_iter().map(|item| {
            WikiNode {
                id: item.docs_token.unwrap_or_default(),
                parent_id: None,
                title: item.title.unwrap_or_default(),
                node_type: item.docs_type.unwrap_or_else(|| "doc".into()),
                has_child: false,
                url: item.url,
                creator: item.owner_id,
                created_at: item.create_time.as_deref().and_then(ts_opt),
                updated_at: item.edit_time.as_deref().and_then(ts_opt),
            }
        }).collect();
        Ok(Page {
            has_more: search.has_more.unwrap_or(false),
            next_cursor: search.count.map(|c| c.to_string()),
            items,
        })
    }
}
