#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub fn handle_confirm_teleport(&self, player: &Player, confirm_teleport: &SConfirmTeleport) {
        let mut awaiting_teleport = player
            .awaiting_teleport
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        if let Some((id, position)) = awaiting_teleport.as_ref() {
            if id == &confirm_teleport.teleport_id {
                player.get_entity().set_pos(*position);
                *awaiting_teleport = None;
            } else if confirm_teleport.teleport_id.0 < id.0 {
                // Client confirmed an older in-flight teleport that was superseded by a newer one.
                // Do not kick; the client will confirm the newer one shortly.
                debug!(
                    "Client confirmed superseded teleport id {} (latest expected is {})",
                    confirm_teleport.teleport_id.0,
                    id.0
                );
            } else {
                warn!(
                    "Unexpected teleport id {} (expected {})",
                    confirm_teleport.teleport_id.0,
                    id.0
                );
            }
        }
    }
}
