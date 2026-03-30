use async_trait::async_trait;
use serde::Deserialize;
use tokimo_core::{
    DataTableService, ImResult, ImError,
    DataBase, DataTable, DataField, DataRecord, Page,
    CreateBaseRequest, ListBasesRequest, CreateTableRequest,
    CreateFieldRequest, ListRecordsRequest, WriteRecordsRequest,
};
use crate::client::DingTalkClient;

// ── DingTalk AITable response types ──

#[allow(dead_code)]
#[derive(Deserialize)]
struct DtBase {
    id: Option<String>,
    name: Option<String>,
    url: Option<String>,
    #[serde(default)]
    tables: Vec<DtTable>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct DtTable {
    id: Option<String>,
    name: Option<String>,
    revision: Option<i64>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct DtField {
    id: Option<String>,
    name: Option<String>,
    #[serde(rename = "fieldType")]
    field_type: Option<String>,
    #[serde(default)]
    property: serde_json::Value,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct DtRecord {
    id: Option<String>,
    #[serde(default)]
    fields: serde_json::Value,
    #[serde(rename = "createdTime")]
    created_time: Option<String>,
}

fn parse_dt_time(s: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    chrono::DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .or_else(|| {
            s.parse::<i64>().ok().and_then(|ms| {
                chrono::DateTime::from_timestamp_millis(ms)
            })
        })
}

impl From<DtBase> for DataBase {
    fn from(b: DtBase) -> Self {
        DataBase {
            id: b.id.unwrap_or_default(),
            name: b.name.unwrap_or_default(),
            url: b.url,
            tables: b.tables.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<DtTable> for DataTable {
    fn from(t: DtTable) -> Self {
        DataTable {
            id: t.id.unwrap_or_default(),
            name: t.name.unwrap_or_default(),
            revision: t.revision,
        }
    }
}

impl From<DtField> for DataField {
    fn from(f: DtField) -> Self {
        DataField {
            id: f.id.unwrap_or_default(),
            name: f.name.unwrap_or_default(),
            field_type: f.field_type.unwrap_or_default(),
            property: f.property,
        }
    }
}

impl From<DtRecord> for DataRecord {
    fn from(r: DtRecord) -> Self {
        DataRecord {
            id: r.id.unwrap_or_default(),
            fields: r.fields,
            created_at: r.created_time.as_deref().and_then(parse_dt_time),
        }
    }
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct DtBaseListResponse {
    #[serde(default)]
    bases: Vec<DtBase>,
    #[serde(rename = "hasMore")]
    has_more: Option<bool>,
    #[serde(rename = "nextCursor")]
    next_cursor: Option<String>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct DtTableListResponse {
    #[serde(default)]
    tables: Vec<DtTable>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct DtFieldListResponse {
    #[serde(default)]
    fields: Vec<DtField>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct DtRecordListResponse {
    #[serde(default)]
    records: Vec<DtRecord>,
    #[serde(rename = "hasMore")]
    has_more: Option<bool>,
    #[serde(rename = "nextCursor")]
    next_cursor: Option<String>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct DtRecordWriteResponse {
    #[serde(default)]
    records: Vec<DtRecord>,
}

#[async_trait]
impl DataTableService for DingTalkClient {
    // ── Base / App operations ──

    async fn create_base(&self, req: CreateBaseRequest) -> ImResult<DataBase> {
        let mut body = serde_json::Map::new();
        body.insert("name".into(), serde_json::json!(req.name));
        if let Some(ref folder) = req.folder_id {
            body.insert("folderId".into(), serde_json::json!(folder));
        }

        let resp = self.post("/v1.0/aitable/bases", &serde_json::Value::Object(body)).await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        let b: DtBase = serde_json::from_str(&text)?;
        Ok(b.into())
    }

    async fn list_bases(&self, req: ListBasesRequest) -> ImResult<Page<DataBase>> {
        let mut params = Vec::new();
        if let Some(ref cursor) = req.cursor {
            params.push(format!("nextCursor={}", cursor));
        }
        if let Some(limit) = req.limit {
            params.push(format!("maxResults={}", limit));
        }
        let path = if params.is_empty() {
            "/v1.0/aitable/bases".to_string()
        } else {
            format!("/v1.0/aitable/bases?{}", params.join("&"))
        };

        let resp = self.get(&path).await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        let data: DtBaseListResponse = serde_json::from_str(&text)?;
        Ok(Page {
            items: data.bases.into_iter().map(Into::into).collect(),
            has_more: data.has_more.unwrap_or(false),
            next_cursor: data.next_cursor,
        })
    }

    async fn get_base(&self, base_id: &str) -> ImResult<DataBase> {
        let path = format!("/v1.0/aitable/bases/{}", base_id);
        let resp = self.get(&path).await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        let b: DtBase = serde_json::from_str(&text)?;
        Ok(b.into())
    }

    async fn delete_base(&self, base_id: &str) -> ImResult<()> {
        let token = self.access_token.read().await.clone().ok_or_else(|| ImError::Auth {
            message: "no access token".into(),
        })?;
        let url = format!("{}/v1.0/aitable/bases/{}", self.base_url, base_id);
        let resp = self.http
            .delete(&url)
            .header("x-acs-dingtalk-access-token", &token)
            .send()
            .await
            .map_err(|e| ImError::Network(e.to_string()))?;
        if !resp.status().is_success() {
            let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
            return Err(ImError::Platform { code: 0, message: text });
        }
        Ok(())
    }

    // ── Table operations ──

    async fn create_table(&self, req: CreateTableRequest) -> ImResult<DataTable> {
        let fields: Vec<serde_json::Value> = req.fields.iter().map(|f| {
            serde_json::json!({
                "name": f.name,
                "fieldType": f.field_type,
                "property": f.property
            })
        }).collect();
        let body = serde_json::json!({
            "name": req.name,
            "fields": fields
        });
        let path = format!("/v1.0/aitable/bases/{}/tables", req.base_id);
        let resp = self.post(&path, &body).await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        let t: DtTable = serde_json::from_str(&text)?;
        Ok(t.into())
    }

    async fn list_tables(&self, base_id: &str) -> ImResult<Vec<DataTable>> {
        let path = format!("/v1.0/aitable/bases/{}/tables", base_id);
        let resp = self.get(&path).await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        let data: DtTableListResponse = serde_json::from_str(&text)?;
        Ok(data.tables.into_iter().map(Into::into).collect())
    }

    async fn delete_table(&self, base_id: &str, table_id: &str) -> ImResult<()> {
        let token = self.access_token.read().await.clone().ok_or_else(|| ImError::Auth {
            message: "no access token".into(),
        })?;
        let url = format!(
            "{}/v1.0/aitable/bases/{}/tables/{}",
            self.base_url, base_id, table_id
        );
        let resp = self.http
            .delete(&url)
            .header("x-acs-dingtalk-access-token", &token)
            .send()
            .await
            .map_err(|e| ImError::Network(e.to_string()))?;
        if !resp.status().is_success() {
            let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
            return Err(ImError::Platform { code: 0, message: text });
        }
        Ok(())
    }

    // ── Field / Column operations ──

    async fn list_fields(&self, base_id: &str, table_id: &str) -> ImResult<Vec<DataField>> {
        let path = format!("/v1.0/aitable/bases/{}/tables/{}/fields", base_id, table_id);
        let resp = self.get(&path).await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        let data: DtFieldListResponse = serde_json::from_str(&text)?;
        Ok(data.fields.into_iter().map(Into::into).collect())
    }

    async fn create_field(
        &self,
        base_id: &str,
        table_id: &str,
        req: CreateFieldRequest,
    ) -> ImResult<DataField> {
        let body = serde_json::json!({
            "name": req.name,
            "fieldType": req.field_type,
            "property": req.property
        });
        let path = format!("/v1.0/aitable/bases/{}/tables/{}/fields", base_id, table_id);
        let resp = self.post(&path, &body).await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        let f: DtField = serde_json::from_str(&text)?;
        Ok(f.into())
    }

    // ── Record / Row operations ──

    async fn list_records(&self, req: ListRecordsRequest) -> ImResult<Page<DataRecord>> {
        let mut body = serde_json::Map::new();
        if let Some(ref view_id) = req.view_id {
            body.insert("viewId".into(), serde_json::json!(view_id));
        }
        if let Some(ref filter) = req.filter {
            body.insert("filter".into(), serde_json::json!(filter));
        }
        if let Some(ref sort) = req.sort {
            body.insert("sort".into(), sort.clone());
        }
        if !req.field_names.is_empty() {
            body.insert("fieldNames".into(), serde_json::json!(req.field_names));
        }
        if let Some(ref cursor) = req.cursor {
            body.insert("nextCursor".into(), serde_json::json!(cursor));
        }
        if let Some(limit) = req.limit {
            body.insert("maxResults".into(), serde_json::json!(limit));
        }

        let path = format!(
            "/v1.0/aitable/bases/{}/tables/{}/records/query",
            req.base_id, req.table_id
        );
        let resp = self.post(&path, &serde_json::Value::Object(body)).await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        let data: DtRecordListResponse = serde_json::from_str(&text)?;
        Ok(Page {
            items: data.records.into_iter().map(Into::into).collect(),
            has_more: data.has_more.unwrap_or(false),
            next_cursor: data.next_cursor,
        })
    }

    async fn write_records(&self, req: WriteRecordsRequest) -> ImResult<Vec<DataRecord>> {
        let records: Vec<serde_json::Value> = req.records.iter().map(|r| {
            let mut obj = serde_json::Map::new();
            if let Some(ref id) = r.id {
                obj.insert("id".into(), serde_json::json!(id));
            }
            obj.insert("fields".into(), r.fields.clone());
            serde_json::Value::Object(obj)
        }).collect();
        let body = serde_json::json!({ "records": records });
        let path = format!(
            "/v1.0/aitable/bases/{}/tables/{}/records",
            req.base_id, req.table_id
        );
        let resp = self.post(&path, &body).await?;
        let status = resp.status();
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        if !status.is_success() {
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        let data: DtRecordWriteResponse = serde_json::from_str(&text)?;
        Ok(data.records.into_iter().map(Into::into).collect())
    }

    async fn delete_records(
        &self,
        base_id: &str,
        table_id: &str,
        record_ids: &[String],
    ) -> ImResult<()> {
        let body = serde_json::json!({ "recordIds": record_ids });
        let path = format!(
            "/v1.0/aitable/bases/{}/tables/{}/records/delete",
            base_id, table_id
        );
        let resp = self.post(&path, &body).await?;
        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
            return Err(ImError::Platform { code: status.as_u16() as i64, message: text });
        }
        Ok(())
    }
}
