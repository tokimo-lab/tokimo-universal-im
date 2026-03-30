use async_trait::async_trait;
use serde::Deserialize;
use tokimo_core::{
    DataTableService, ImResult, ImError,
    DataBase, DataTable, DataField, DataRecord, Page,
    CreateBaseRequest, ListBasesRequest, CreateTableRequest,
    CreateFieldRequest, ListRecordsRequest, WriteRecordsRequest,
};
use crate::client::WeComClient;

const PLATFORM: &str = "wecom";

#[derive(Deserialize)]
#[allow(dead_code)]
struct GetRecordsResp {
    errcode: Option<i64>,
    errmsg: Option<String>,
    #[serde(default)]
    records: Vec<WcRecord>,
    has_more: Option<bool>,
    next: Option<String>,
    total: Option<u64>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct WcRecord {
    record_id: Option<String>,
    values: Option<serde_json::Value>,
    created_time: Option<i64>,
}

#[derive(Deserialize)]
struct AddRecordsResp {
    errcode: Option<i64>,
    errmsg: Option<String>,
    #[serde(default)]
    records: Vec<WcRecord>,
}

#[derive(Deserialize)]
struct UpdateRecordsResp {
    errcode: Option<i64>,
    errmsg: Option<String>,
    #[serde(default)]
    records: Vec<WcRecord>,
}

#[derive(Deserialize)]
struct DeleteRecordsResp {
    errcode: Option<i64>,
    errmsg: Option<String>,
}

#[derive(Deserialize)]
struct GetFieldsResp {
    errcode: Option<i64>,
    errmsg: Option<String>,
    #[serde(default)]
    fields: Vec<WcField>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct WcField {
    field_id: Option<String>,
    field_title: Option<String>,
    field_type: Option<String>,
}

fn check_errcode(errcode: Option<i64>, errmsg: Option<String>, raw: String) -> ImResult<()> {
    if errcode.unwrap_or(0) != 0 {
        return Err(ImError::Platform {
            code: errcode.unwrap_or(-1),
            message: errmsg.unwrap_or(raw),
        });
    }
    Ok(())
}

fn wc_record_to_data_record(r: WcRecord) -> DataRecord {
    DataRecord {
        id: r.record_id.unwrap_or_default(),
        fields: r.values.unwrap_or(serde_json::Value::Null),
        created_at: r.created_time.map(|ts| {
            chrono::DateTime::from_timestamp(ts, 0).unwrap_or_else(chrono::Utc::now)
        }),
    }
}

fn not_supported(feature: &str) -> ImError {
    ImError::NotSupported {
        feature: feature.into(),
        platform: PLATFORM.into(),
    }
}

#[async_trait]
impl DataTableService for WeComClient {
    // ── Base / App operations (not supported — WeCom smartsheets are managed via document service) ──

    async fn create_base(&self, _req: CreateBaseRequest) -> ImResult<DataBase> {
        Err(not_supported("create_base"))
    }

    async fn list_bases(&self, _req: ListBasesRequest) -> ImResult<Page<DataBase>> {
        Err(not_supported("list_bases"))
    }

    async fn get_base(&self, _base_id: &str) -> ImResult<DataBase> {
        Err(not_supported("get_base"))
    }

    async fn delete_base(&self, _base_id: &str) -> ImResult<()> {
        Err(not_supported("delete_base"))
    }

    // ── Table operations (not supported) ──

    async fn create_table(&self, _req: CreateTableRequest) -> ImResult<DataTable> {
        Err(not_supported("create_table"))
    }

    async fn list_tables(&self, _base_id: &str) -> ImResult<Vec<DataTable>> {
        Err(not_supported("list_tables"))
    }

    async fn delete_table(&self, _base_id: &str, _table_id: &str) -> ImResult<()> {
        Err(not_supported("delete_table"))
    }

    // ── Field / Column operations ──

    async fn list_fields(&self, base_id: &str, table_id: &str) -> ImResult<Vec<DataField>> {
        let body = serde_json::json!({
            "docid": base_id,
            "sheet_id": table_id,
        });
        let resp = self.post("/cgi-bin/wedoc/smartsheet/get_fields", &body).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: GetFieldsResp = serde_json::from_str(&text)?;
        check_errcode(data.errcode, data.errmsg, text)?;

        Ok(data.fields.into_iter().map(|f| DataField {
            id: f.field_id.unwrap_or_default(),
            name: f.field_title.unwrap_or_default(),
            field_type: f.field_type.unwrap_or_default(),
            property: serde_json::Value::Null,
        }).collect())
    }

    async fn create_field(&self, _base_id: &str, _table_id: &str, _req: CreateFieldRequest) -> ImResult<DataField> {
        Err(not_supported("create_field"))
    }

    // ── Record / Row operations ──

    async fn list_records(&self, req: ListRecordsRequest) -> ImResult<Page<DataRecord>> {
        let mut body = serde_json::json!({
            "docid": req.base_id,
            "sheet_id": req.table_id,
        });
        if let Some(ref cursor) = req.cursor {
            body["offset"] = serde_json::Value::String(cursor.clone());
        }
        if let Some(limit) = req.limit {
            body["limit"] = serde_json::json!(limit.min(500));
        }

        let resp = self.post("/cgi-bin/wedoc/smartsheet/get_records", &body).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: GetRecordsResp = serde_json::from_str(&text)?;
        check_errcode(data.errcode, data.errmsg, text)?;

        let items: Vec<DataRecord> = data.records.into_iter().map(wc_record_to_data_record).collect();
        Ok(Page {
            items,
            has_more: data.has_more.unwrap_or(false),
            next_cursor: data.next,
        })
    }

    async fn write_records(&self, req: WriteRecordsRequest) -> ImResult<Vec<DataRecord>> {
        // Separate creates (no id) from updates (has id)
        let mut creates = vec![];
        let mut updates = vec![];
        for rec in &req.records {
            if rec.id.is_some() {
                updates.push(rec);
            } else {
                creates.push(rec);
            }
        }

        let mut results = Vec::new();

        // Handle creates in batches of 500
        for chunk in creates.chunks(500) {
            let records: Vec<serde_json::Value> = chunk.iter().map(|r| {
                serde_json::json!({ "values": r.fields })
            }).collect();
            let body = serde_json::json!({
                "docid": req.base_id,
                "sheet_id": req.table_id,
                "records": records,
            });
            let resp = self.post("/cgi-bin/wedoc/smartsheet/add_records", &body).await?;
            let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
            let data: AddRecordsResp = serde_json::from_str(&text)?;
            check_errcode(data.errcode, data.errmsg, text)?;
            results.extend(data.records.into_iter().map(wc_record_to_data_record));
        }

        // Handle updates in batches of 500
        for chunk in updates.chunks(500) {
            let records: Vec<serde_json::Value> = chunk.iter().map(|r| {
                serde_json::json!({
                    "record_id": r.id,
                    "values": r.fields,
                })
            }).collect();
            let body = serde_json::json!({
                "docid": req.base_id,
                "sheet_id": req.table_id,
                "records": records,
            });
            let resp = self.post("/cgi-bin/wedoc/smartsheet/update_records", &body).await?;
            let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
            let data: UpdateRecordsResp = serde_json::from_str(&text)?;
            check_errcode(data.errcode, data.errmsg, text)?;
            results.extend(data.records.into_iter().map(wc_record_to_data_record));
        }

        Ok(results)
    }

    async fn delete_records(&self, base_id: &str, table_id: &str, record_ids: &[String]) -> ImResult<()> {
        let body = serde_json::json!({
            "docid": base_id,
            "sheet_id": table_id,
            "record_ids": record_ids,
        });
        let resp = self.post("/cgi-bin/wedoc/smartsheet/delete_records", &body).await?;
        let text = resp.text().await.map_err(|e| ImError::Network(e.to_string()))?;
        let data: DeleteRecordsResp = serde_json::from_str(&text)?;
        check_errcode(data.errcode, data.errmsg, text)?;
        Ok(())
    }
}
