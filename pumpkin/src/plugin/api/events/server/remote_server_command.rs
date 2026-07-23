use pumpkin_macros::{Event, cancellable};

/// A command received through the authenticated remote-console transport.
///
/// This is distinct from
/// [`super::server_command::ServerCommandEvent`], which represents a command
/// entered at the local server console.
#[cancellable]
#[derive(Event, Clone)]
pub struct RemoteServerCommandEvent {
    pub command: String,
}

impl RemoteServerCommandEvent {
    #[must_use]
    pub const fn new(command: String) -> Self {
        Self {
            command,
            cancelled: false,
        }
    }
}
