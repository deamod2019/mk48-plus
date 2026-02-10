use crate::contact_ref::ContactRef;
use crate::player::Status;
use crate::server::Server;
use crate::world::World;
use atomic_refcell::AtomicRef;
use common::complete::CompleteTrait;
use common::contact::ContactTrait;
use common::death_reason::DeathReason;
use common::protocol::{FactionId, FactionUpdate, Update, WorldEvent};
use common::terrain;
use common::terrain::{ChunkSet, Terrain};
use common::ticks::{Ticks, TicksRepr};
use common::velocity::Velocity;
use game_server::player::PlayerData;
use glam::Vec2;
use std::ops::RangeInclusive;

/// A "Complete" server to client update that references world data to avoid additional allocation.
pub struct CompleteRef<'a, I: Iterator<Item = ContactRef<'a>>> {
    /// Always some, until taken.
    contacts: Option<I>,
    player: AtomicRef<'a, PlayerData<Server>>,
    world: &'a World,
    camera_pos: Vec2,
    camera_dims: Vec2,
}

impl<'a, I: Iterator<Item = ContactRef<'a>>> CompleteRef<'a, I> {
    pub fn new(
        contacts: I,
        player: AtomicRef<'a, PlayerData<Server>>,
        world: &'a World,
        camera_pos: Vec2,
        camera_dims: Vec2,
    ) -> Self {
        Self {
            contacts: Some(contacts),
            player,
            world,
            camera_pos,
            camera_dims,
        }
    }

    pub fn into_update(
        self,
        counter: Ticks,
        loaded_chunks: &mut ChunkSet,
        faction_update: Option<FactionUpdate>,
        altar_position: Option<Vec2>,
        altar_sacrifice_counts: [u8; FactionId::COUNT],
    ) -> Update {
        let death_reason = if let Status::Dead { reason, .. } = &self.player.data.status {
            Some(reason.clone())
        } else {
            None
        };

        // Any updated chunks are now no longer loaded.
        let mut new_loaded_chunks = loaded_chunks.and(&self.world.terrain.updated.not());

        // All chunks that are currently visible (on screen).
        // Uses a rect instead of a circle because that is what the client renders,
        // even though it is slightly less realistic.
        let visible = ChunkSet::new_rect(
            self.camera_pos,
            self.camera_dims + Vec2::splat(terrain::SCALE * 2.0),
        );

        // Actually load more chunks.
        let loading = visible.and(&new_loaded_chunks.not());

        // The chunks that will be loaded following this message.
        new_loaded_chunks = visible.or(&new_loaded_chunks);

        let terrain = loading
            .into_iter()
            .map(|id| {
                (
                    id,
                    self.world.terrain.get_chunk(id).to_serialized_chunk(
                        loaded_chunks.contains(id),
                        &self.world.terrain,
                        id,
                    ),
                )
            })
            .collect();

        *loaded_chunks = new_loaded_chunks;

        // Read bot alliance setting from hot-reloadable config.
        let bot_alliance_enabled = crate::runtime_config::hot_bot_alliance_enabled();

        // Filter events per-faction: strip server-internal events and
        // only send faction-specific events to the correct faction.
        let my_faction = self.player.data.faction;
        let events: Vec<WorldEvent> = self.world.events.iter().filter(|e| match e {
            // Server-internal event, never send to clients.
            WorldEvent::AltarSacrifice { .. } => false,
            // Only send to the faction that discovered.
            WorldEvent::AltarDiscovered { faction, .. } => my_faction == Some(*faction),
            // Global events.
            _ => true,
        }).cloned().collect();

        Update {
            contacts: self
                .contacts
                .unwrap()
                .filter_map(|contact| {
                    let modulus = if let Some(entity_type) = contact.entity_type() {
                        let range: RangeInclusive<Ticks> = entity_type.data().kind.keep_alive();

                        if contact.transform().velocity.abs() > Velocity::from_mps(1.0) {
                            // Send more often if moving.
                            *range.start()
                        } else {
                            *range.end()
                        }
                    } else {
                        Ticks::from_repr(5)
                    };

                    let send = counter
                        .wrapping_add(Ticks::from_repr(contact.id().get() as TicksRepr))
                        % (modulus + Ticks::ONE)
                        == Ticks::ZERO;
                    send.then(|| contact.into_contact())
                })
                .collect(),
            events,
            death_reason,
            score: self.player.score,
            kill_log: self.player.data.kill_log.iter().map(|(k, v)| (*k, *v)).collect(),
            world_radius: self.world.radius,
            terrain,
            bot_alliance_enabled,
            faction_data: faction_update,
            my_faction,
            altar_position,
            altar_sacrifice_counts,
        }
    }
}

impl<'a, I: Iterator<Item = ContactRef<'a>>> CompleteTrait<'a> for CompleteRef<'a, I> {
    type Contact = ContactRef<'a>;
    type Iterator = I;

    fn contacts(&mut self) -> Self::Iterator {
        self.contacts.take().unwrap()
    }

    fn collect_contacts(&mut self) -> Vec<Self::Contact> {
        self.contacts.take().unwrap().collect()
    }

    fn death_reason(&self) -> Option<&DeathReason> {
        if let Status::Dead { reason, .. } = &self.player.data.status {
            Some(reason)
        } else {
            None
        }
    }

    #[inline]
    fn score(&self) -> u32 {
        self.player.score
    }

    #[inline]
    fn world_radius(&self) -> f32 {
        self.world.radius
    }

    #[inline]
    fn terrain(&self) -> &Terrain {
        // TODO limit visibility of terrain.
        &self.world.terrain
    }
}
