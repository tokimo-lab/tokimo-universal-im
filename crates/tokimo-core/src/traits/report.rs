use async_trait::async_trait;
use crate::error::ImResult;
use crate::types::{
    Report, ReportTemplate, ReportStatistics, Page,
    ListReportsRequest, CreateReportRequest,
};

/// Report / daily / weekly management.
#[async_trait]
pub trait ReportService: Send + Sync {
    /// List report templates.
    async fn list_templates(&self) -> ImResult<Vec<ReportTemplate>>;

    /// Get report template detail.
    async fn get_template(&self, template_name: &str) -> ImResult<ReportTemplate>;

    /// Create/submit a report.
    async fn create_report(&self, req: CreateReportRequest) -> ImResult<Report>;

    /// List submitted reports.
    async fn list_reports(&self, req: ListReportsRequest) -> ImResult<Page<Report>>;

    /// Get a single report.
    async fn get_report(&self, report_id: &str) -> ImResult<Report>;

    /// Get report statistics.
    async fn get_statistics(&self, report_id: &str) -> ImResult<ReportStatistics>;
}
