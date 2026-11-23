#[allow(clippy::wildcard_imports)]
use super::*;

impl PendingConnection {
    pub async fn handle_plugin_response(
        &mut self,
        server: &Arc<Server>,
        plugin_response: SLoginPluginResponse,
    ) -> Option<PacketHandlerResult> {
        debug!("Handling plugin");
        let proxy_config = &server.advanced_config.networking.proxy;
        // The request side is gated on this too, so forwarding must not stay
        // reachable through an unsolicited response while proxy mode is off.
        if !proxy_config.enabled {
            return None;
        }
        if proxy_config.vine.enabled {
            let expected_challenge = self.vine_challenge.take();
            match vine::receive_vine_plugin_response(
                self.address.port(),
                &proxy_config.vine,
                plugin_response,
                expected_challenge,
            ) {
                Ok((profile, new_address)) => {
                    self.gameprofile = Some(profile.clone());
                    self.address = new_address;
                    self.finish_login(server, &profile).await
                }
                Err(error) => {
                    self.kick(TextComponent::text(error.to_string())).await;
                    Some(PacketHandlerResult::Stop)
                }
            }
        } else if proxy_config.velocity.enabled {
            match velocity::receive_velocity_plugin_response(
                self.address.port(),
                &proxy_config.velocity,
                plugin_response,
                &mut self.velocity_message_id,
            ) {
                Ok((profile, new_address)) => {
                    self.gameprofile = Some(profile.clone());
                    self.address = new_address;
                    if let Some(old_player) = server.get_player_by_uuid(profile.id) {
                        tracing::warn!(
                            "Disconnecting stale session for {} ({}) on new login",
                            profile.name, profile.id
                        );
                        old_player.kick(
                            crate::DisconnectReason::Kicked,
                            &TextComponent::text("Logged in from another location"),
                        );
                        if let crate::ClientPlatform::Java(client) = old_player.client.as_ref() {
                            client.close();
                        }
                    }
                    self.finish_login(server, &profile).await
                }
                Err(error) => {
                    self.kick(TextComponent::text(error.to_string())).await;
                    Some(PacketHandlerResult::Stop)
                }
            }
        } else {
            None
        }
    }
}
