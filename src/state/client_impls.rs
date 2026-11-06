use smithay::reexports::wayland_server::backend::{ClientData, ClientId, DisconnectReason};

use crate::state::NocturaClients;

impl ClientData for NocturaClients {
    fn initialized(&self, _client_id: ClientId) {}
    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason  ) {}
}