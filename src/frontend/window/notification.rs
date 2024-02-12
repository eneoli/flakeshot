use notify_rust::Urgency;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notification {
    pub msg: String,
    pub urgency: Urgency,
}
