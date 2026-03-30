use async_trait::async_trait;
use crate::error::ImResult;
use crate::types::{
    DataBase, DataTable, DataField, DataRecord, Page,
    CreateBaseRequest, ListBasesRequest, CreateTableRequest,
    CreateFieldRequest, ListRecordsRequest, WriteRecordsRequest,
};

/// Data table / AITable / Bitable / Smartsheet service.
#[async_trait]
pub trait DataTableService: Send + Sync {
    // ── Base / App operations ──

    /// Create a new base/app.
    async fn create_base(&self, req: CreateBaseRequest) -> ImResult<DataBase>;

    /// List bases/apps.
    async fn list_bases(&self, req: ListBasesRequest) -> ImResult<Page<DataBase>>;

    /// Get a base by ID.
    async fn get_base(&self, base_id: &str) -> ImResult<DataBase>;

    /// Delete a base.
    async fn delete_base(&self, base_id: &str) -> ImResult<()>;

    // ── Table operations ──

    /// Create a table within a base.
    async fn create_table(&self, req: CreateTableRequest) -> ImResult<DataTable>;

    /// List tables in a base.
    async fn list_tables(&self, base_id: &str) -> ImResult<Vec<DataTable>>;

    /// Delete a table.
    async fn delete_table(&self, base_id: &str, table_id: &str) -> ImResult<()>;

    // ── Field / Column operations ──

    /// List fields for a table.
    async fn list_fields(&self, base_id: &str, table_id: &str) -> ImResult<Vec<DataField>>;

    /// Create a field.
    async fn create_field(&self, base_id: &str, table_id: &str, req: CreateFieldRequest) -> ImResult<DataField>;

    // ── Record / Row operations ──

    /// Query records from a table.
    async fn list_records(&self, req: ListRecordsRequest) -> ImResult<Page<DataRecord>>;

    /// Create or update records (batch).
    async fn write_records(&self, req: WriteRecordsRequest) -> ImResult<Vec<DataRecord>>;

    /// Delete records by IDs.
    async fn delete_records(&self, base_id: &str, table_id: &str, record_ids: &[String]) -> ImResult<()>;
}
