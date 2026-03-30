use async_trait::async_trait;
use serde::Deserialize;
use tokimo_core::{
    DataTableService, ImResult, ImError,
    DataBase, DataTable, DataField, DataRecord, Page,
    CreateBaseRequest, ListBasesRequest, CreateTableRequest,
    CreateFieldRequest, ListRecordsRequest, WriteRecordsRequest,
};
use crate::client::LarkClient;

#[derive(Deserialize)]
struct LarkResp<T> {
    code: Option<i64>,
    msg: Option<String>,
    data: Option<T>,
}

// ── Base types ──

#[derive(Deserialize)]
struct CreateBaseData {
    app: Option<LarkBase>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct LarkBase {
    app_token: Option<String>,
    name: Option<String>,
    url: Option<String>,
}

#[derive(Deserialize)]
struct ListBasesData {
    #[serde(default)]
    items: Vec<LarkBase>,
    page_token: Option<String>,
    has_more: Option<bool>,
}

#[derive(Deserialize)]
struct GetBaseData {
    app: Option<LarkBase>,
}

impl From<LarkBase> for DataBase {
    fn from(b: LarkBase) -> Self {
        DataBase {
            id: b.app_token.unwrap_or_default(),
            name: b.name.unwrap_or_default(),
            url: b.url,
            tables: vec![],
        }
    }
}

// ── Table types ──

#[derive(Deserialize)]
struct CreateTableData {
    table_id: Option<String>,
}

#[derive(Deserialize)]
struct ListTablesData {
    #[serde(default)]
    items: Vec<LarkTable>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct LarkTable {
    table_id: Option<String>,
    name: Option<String>,
    revision: Option<i64>,
}

impl From<LarkTable> for DataTable {
    fn from(t: LarkTable) -> Self {
        DataTable {
            id: t.table_id.unwrap_or_default(),
            name: t.name.unwrap_or_default(),
            revision: t.revision,
        }
    }
}

// ── Field types ──

#[derive(Deserialize)]
struct FieldData {
    field: Option<LarkField>,
}

#[derive(Deserialize)]
struct ListFieldsData {
    #[serde(default)]
    items: Vec<LarkField>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct LarkField {
    field_id: Option<String>,
    field_name: Option<String>,
    #[serde(rename = "type")]
    field_type: Option<i64>,
    property: Option<serde_json::Value>,
}

fn field_type_name(t: i64) -> String {
    match t {
        1 => "text".into(),
        2 => "number".into(),
        3 => "select".into(),
        4 => "multi_select".into(),
        5 => "date".into(),
        7 => "checkbox".into(),
        11 => "person".into(),
        13 => "phone".into(),
        15 => "url".into(),
        17 => "attachment".into(),
        18 => "link".into(),
        20 => "formula".into(),
        22 => "created_time".into(),
        23 => "modified_time".into(),
        _ => format!("type_{}", t),
    }
}

impl From<LarkField> for DataField {
    fn from(f: LarkField) -> Self {
        DataField {
            id: f.field_id.unwrap_or_default(),
            name: f.field_name.unwrap_or_default(),
            field_type: f.field_type.map(field_type_name).unwrap_or_default(),
            property: f.property.unwrap_or(serde_json::Value::Null),
        }
    }
}

// ── Record types ──

#[derive(Deserialize)]
#[allow(dead_code)]
struct ListRecordsData {
    #[serde(default)]
    items: Vec<LarkRecord>,
    page_token: Option<String>,
    has_more: Option<bool>,
    total: Option<u32>,
}

#[derive(Deserialize)]
struct BatchRecordData {
    #[serde(default)]
    records: Vec<LarkRecord>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct LarkRecord {
    record_id: Option<String>,
    fields: Option<serde_json::Value>,
    created_time: Option<i64>,
}

impl From<LarkRecord> for DataRecord {
    fn from(r: LarkRecord) -> Self {
        let created_at = r.created_time
            .and_then(|ms| chrono::DateTime::from_timestamp_millis(ms));
        DataRecord {
            id: r.record_id.unwrap_or_default(),
            fields: r.fields.unwrap_or(serde_json::Value::Null),
            created_at,
        }
    }
}

#[async_trait]
impl DataTableService for LarkClient {
    // ── Base / App operations ──

    async fn create_base(&self, req: CreateBaseRequest) -> ImResult<DataBase> {
        let body = serde_json::json!({
            "name": req.name,
            "folder_token": req.folder_id.unwrap_or_default(),
        });
        let resp = self.post("/open-apis/bitable/v1/apps", &body).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<CreateBaseData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let base = data.data.and_then(|d| d.app).ok_or_else(|| ImError::Internal("empty response".into()))?;
        Ok(base.into())
    }

    async fn list_bases(&self, req: ListBasesRequest) -> ImResult<Page<DataBase>> {
        let mut path = format!(
            "/open-apis/bitable/v1/apps?page_size={}",
            req.limit.unwrap_or(20),
        );
        if let Some(ref cursor) = req.cursor {
            path.push_str(&format!("&page_token={}", cursor));
        }
        let resp = self.get(&path).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<ListBasesData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let list = data.data.unwrap_or(ListBasesData { items: vec![], page_token: None, has_more: None });
        Ok(Page {
            items: list.items.into_iter().map(Into::into).collect(),
            has_more: list.has_more.unwrap_or(false),
            next_cursor: list.page_token,
        })
    }

    async fn get_base(&self, base_id: &str) -> ImResult<DataBase> {
        let path = format!("/open-apis/bitable/v1/apps/{}", base_id);
        let resp = self.get(&path).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<GetBaseData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let base = data.data.and_then(|d| d.app).ok_or_else(|| ImError::NotFound {
            resource: base_id.into(),
        })?;
        Ok(base.into())
    }

    async fn delete_base(&self, _base_id: &str) -> ImResult<()> {
        Err(ImError::NotSupported {
            feature: "delete_base (Lark Bitable does not support base deletion via API)".into(),
            platform: "lark".into(),
        })
    }

    // ── Table operations ──

    async fn create_table(&self, req: CreateTableRequest) -> ImResult<DataTable> {
        let fields: Vec<serde_json::Value> = req.fields.iter().map(|f| {
            serde_json::json!({
                "field_name": f.name,
                "type": match f.field_type.as_str() {
                    "text" => 1,
                    "number" => 2,
                    "select" => 3,
                    "multi_select" => 4,
                    "date" => 5,
                    "checkbox" => 7,
                    _ => 1,
                },
            })
        }).collect();
        let body = serde_json::json!({
            "table": {
                "name": req.name,
                "default_view_name": "Grid View",
                "fields": fields,
            }
        });
        let path = format!("/open-apis/bitable/v1/apps/{}/tables", req.base_id);
        let resp = self.post(&path, &body).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<CreateTableData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let table_id = data.data.and_then(|d| d.table_id).unwrap_or_default();
        Ok(DataTable {
            id: table_id,
            name: req.name,
            revision: None,
        })
    }

    async fn list_tables(&self, base_id: &str) -> ImResult<Vec<DataTable>> {
        let path = format!("/open-apis/bitable/v1/apps/{}/tables", base_id);
        let resp = self.get(&path).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<ListTablesData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let list = data.data.unwrap_or(ListTablesData { items: vec![] });
        Ok(list.items.into_iter().map(Into::into).collect())
    }

    async fn delete_table(&self, base_id: &str, table_id: &str) -> ImResult<()> {
        let path = format!("/open-apis/bitable/v1/apps/{}/tables/{}", base_id, table_id);
        let resp = self.delete(&path).await?;
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

    // ── Field operations ──

    async fn list_fields(&self, base_id: &str, table_id: &str) -> ImResult<Vec<DataField>> {
        let path = format!("/open-apis/bitable/v1/apps/{}/tables/{}/fields", base_id, table_id);
        let resp = self.get(&path).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<ListFieldsData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let list = data.data.unwrap_or(ListFieldsData { items: vec![] });
        Ok(list.items.into_iter().map(Into::into).collect())
    }

    async fn create_field(&self, base_id: &str, table_id: &str, req: CreateFieldRequest) -> ImResult<DataField> {
        let type_num = match req.field_type.as_str() {
            "text" => 1,
            "number" => 2,
            "select" => 3,
            "multi_select" => 4,
            "date" => 5,
            "checkbox" => 7,
            "person" => 11,
            "phone" => 13,
            "url" => 15,
            "attachment" => 17,
            _ => 1,
        };
        let body = serde_json::json!({
            "field_name": req.name,
            "type": type_num,
            "property": req.property,
        });
        let path = format!("/open-apis/bitable/v1/apps/{}/tables/{}/fields", base_id, table_id);
        let resp = self.post(&path, &body).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<FieldData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let field = data.data.and_then(|d| d.field).ok_or_else(|| ImError::Internal("empty field".into()))?;
        Ok(field.into())
    }

    // ── Record operations ──

    async fn list_records(&self, req: ListRecordsRequest) -> ImResult<Page<DataRecord>> {
        let mut path = format!(
            "/open-apis/bitable/v1/apps/{}/tables/{}/records?page_size={}",
            req.base_id, req.table_id,
            req.limit.unwrap_or(100),
        );
        if let Some(ref cursor) = req.cursor {
            path.push_str(&format!("&page_token={}", cursor));
        }
        if let Some(ref view_id) = req.view_id {
            path.push_str(&format!("&view_id={}", view_id));
        }
        if let Some(ref filter) = req.filter {
            path.push_str(&format!("&filter={}", url::form_urlencoded::byte_serialize(filter.as_bytes()).collect::<String>()));
        }
        let resp = self.get(&path).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: LarkResp<ListRecordsData> = serde_json::from_str(&text)?;
        if data.code.unwrap_or(0) != 0 {
            return Err(ImError::Platform {
                code: data.code.unwrap_or(-1),
                message: data.msg.unwrap_or(text),
            });
        }
        let list = data.data.unwrap_or(ListRecordsData {
            items: vec![], page_token: None, has_more: None, total: None,
        });
        Ok(Page {
            items: list.items.into_iter().map(Into::into).collect(),
            has_more: list.has_more.unwrap_or(false),
            next_cursor: list.page_token,
        })
    }

    async fn write_records(&self, req: WriteRecordsRequest) -> ImResult<Vec<DataRecord>> {
        let mut to_create = Vec::new();
        let mut to_update = Vec::new();
        for rec in &req.records {
            if let Some(ref id) = rec.id {
                to_update.push(serde_json::json!({
                    "record_id": id,
                    "fields": rec.fields,
                }));
            } else {
                to_create.push(serde_json::json!({
                    "fields": rec.fields,
                }));
            }
        }
        let mut results = Vec::new();

        if !to_create.is_empty() {
            let body = serde_json::json!({ "records": to_create });
            let path = format!(
                "/open-apis/bitable/v1/apps/{}/tables/{}/records/batch_create",
                req.base_id, req.table_id,
            );
            let resp = self.post(&path, &body).await?;
            let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
            let data: LarkResp<BatchRecordData> = serde_json::from_str(&text)?;
            if data.code.unwrap_or(0) != 0 {
                return Err(ImError::Platform {
                    code: data.code.unwrap_or(-1),
                    message: data.msg.unwrap_or(text),
                });
            }
            if let Some(d) = data.data {
                results.extend(d.records.into_iter().map(Into::into));
            }
        }

        if !to_update.is_empty() {
            let body = serde_json::json!({ "records": to_update });
            let path = format!(
                "/open-apis/bitable/v1/apps/{}/tables/{}/records/batch_update",
                req.base_id, req.table_id,
            );
            let resp = self.post(&path, &body).await?;
            let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
            let data: LarkResp<BatchRecordData> = serde_json::from_str(&text)?;
            if data.code.unwrap_or(0) != 0 {
                return Err(ImError::Platform {
                    code: data.code.unwrap_or(-1),
                    message: data.msg.unwrap_or(text),
                });
            }
            if let Some(d) = data.data {
                results.extend(d.records.into_iter().map(Into::into));
            }
        }

        Ok(results)
    }

    async fn delete_records(&self, base_id: &str, table_id: &str, record_ids: &[String]) -> ImResult<()> {
        let body = serde_json::json!({ "records": record_ids });
        let path = format!(
            "/open-apis/bitable/v1/apps/{}/tables/{}/records/batch_delete",
            base_id, table_id,
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
}
