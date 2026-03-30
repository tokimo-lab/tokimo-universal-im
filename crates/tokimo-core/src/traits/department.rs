use async_trait::async_trait;
use crate::error::ImResult;
use crate::types::{
    DepartmentDetail, User, Page,
    ListDepartmentsRequest, ListDepartmentMembersRequest,
};

/// Department / organizational structure management.
#[async_trait]
pub trait DepartmentService: Send + Sync {
    /// List departments (optionally under a parent).
    async fn list_departments(&self, req: ListDepartmentsRequest) -> ImResult<Page<DepartmentDetail>>;

    /// Get a single department by ID.
    async fn get_department(&self, department_id: &str) -> ImResult<DepartmentDetail>;

    /// List members of a department.
    async fn list_department_members(&self, req: ListDepartmentMembersRequest) -> ImResult<Page<User>>;
}
