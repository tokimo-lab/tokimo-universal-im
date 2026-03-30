use crate::types::Platform;

/// The top-level provider that ties all service traits together.
///
/// Each platform crate (dingtalk, wecom, lark) implements this trait to
/// expose its supported capabilities. Optional services return `None` if
/// the platform does not support that domain.
pub trait ImProvider: Send + Sync {
    /// Which platform this provider represents.
    fn platform(&self) -> Platform;

    /// Authentication service — always required.
    fn auth(&self) -> &dyn super::AuthService;

    /// Messaging service (send / receive / recall).
    fn messaging(&self) -> Option<&dyn super::MessagingService>;

    /// Extended messaging (reply, forward, reactions, read status).
    fn message_ext(&self) -> Option<&dyn super::MessageExtService>;

    /// Contact / address-book service.
    fn contact(&self) -> Option<&dyn super::ContactService>;

    /// Group chat management service.
    fn group(&self) -> Option<&dyn super::GroupService>;

    /// Chat list / conversation list service.
    fn chat_list(&self) -> Option<&dyn super::ChatListService>;

    /// Calendar / schedule service.
    fn calendar(&self) -> Option<&dyn super::CalendarService>;

    /// Task / to-do service.
    fn task(&self) -> Option<&dyn super::TaskService>;

    /// Meeting / video conference service.
    fn meeting(&self) -> Option<&dyn super::MeetingService>;

    /// Media upload / download service.
    fn media(&self) -> Option<&dyn super::MediaService>;

    /// Document service.
    fn document(&self) -> Option<&dyn super::DocumentService>;
}
