use serde::{Deserialize, Serialize};

/// A data table (AITable / Bitable / Smartsheet).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataTable {
    /// Table ID.
    pub id: String,
    /// Table name.
    pub name: String,
    /// Revision / version.
    pub revision: Option<i64>,
}

/// A data table base/app (container for multiple tables).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataBase {
    /// Base / App ID.
    pub id: String,
    /// Base name.
    pub name: String,
    /// URL to access.
    pub url: Option<String>,
    /// Tables in this base.
    #[serde(default)]
    pub tables: Vec<DataTable>,
}

/// A field / column definition in a data table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataField {
    /// Field ID.
    pub id: String,
    /// Field name.
    pub name: String,
    /// Field type (text, number, date, select, etc.).
    pub field_type: String,
    /// Additional type-specific properties.
    #[serde(default)]
    pub property: serde_json::Value,
}

/// A record / row in a data table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecord {
    /// Record ID.
    pub id: String,
    /// Cell values as field_id/name -> value.
    pub fields: serde_json::Value,
    /// Creation time.
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Request to create a data table base.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBaseRequest {
    /// Base name.
    pub name: String,
    /// Optional folder/space to create in.
    pub folder_id: Option<String>,
}

/// Request to list bases.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListBasesRequest {
    pub cursor: Option<String>,
    pub limit: Option<u32>,
}

/// Request to create a table in a base.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTableRequest {
    /// Base ID that this table belongs to.
    pub base_id: String,
    /// Table name.
    pub name: String,
    /// Initial field definitions.
    #[serde(default)]
    pub fields: Vec<CreateFieldRequest>,
}

/// Request to create a field/column.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFieldRequest {
    /// Field name.
    pub name: String,
    /// Field type.
    pub field_type: String,
    /// Type-specific properties.
    #[serde(default)]
    pub property: serde_json::Value,
}

/// Request to query records.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRecordsRequest {
    /// Base ID.
    pub base_id: String,
    /// Table ID.
    pub table_id: String,
    /// View ID (optional).
    pub view_id: Option<String>,
    /// Filter formula / expression.
    pub filter: Option<String>,
    /// Sort specification.
    pub sort: Option<serde_json::Value>,
    /// Fields to return.
    #[serde(default)]
    pub field_names: Vec<String>,
    pub cursor: Option<String>,
    pub limit: Option<u32>,
}

/// Request to create/update records.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteRecordsRequest {
    /// Base ID.
    pub base_id: String,
    /// Table ID.
    pub table_id: String,
    /// Records to create/update.
    pub records: Vec<DataRecordWrite>,
}

/// A record to write (create or update).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRecordWrite {
    /// Record ID (for update; None for create).
    pub id: Option<String>,
    /// Field values.
    pub fields: serde_json::Value,
}
