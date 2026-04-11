// SPDX-FileCopyrightText: 2021 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::game::Mk48Game;
use crate::translation::Mk48Translation;
use crate::ui::about_dialog::AboutDialog;
use crate::ui::changelog_dialog::ChangelogDialog;
use crate::ui::help_dialog::HelpDialog;
use crate::ui::hint::Hint;
pub use crate::ui::instructions::InstructionStatus;
use crate::ui::levels_dialog::LevelsDialog;
use crate::ui::logo::logo;
use crate::ui::respawn_overlay::RespawnOverlay;
use crate::ui::settings_dialog::SettingsDialog;
use crate::ui::ship_controls::ShipControls;
use crate::ui::ships_detail_dialog::ShipsDetailDialog;
use crate::ui::status_overlay::StatusOverlay;
use crate::ui::upgrade_overlay::UpgradeOverlay;
use client_util::context::Context;
use common::altitude::Altitude;
use common::angle::Angle;
use common::death_reason::DeathReason;
use common::entity::EntityType;
use common::velocity::Velocity;
use core_protocol::id::{LanguageId, TeamId};
use core_protocol::name::PlayerAlias;
use engine_macros::SmolRoutable;
use glam::Vec2;
use std::collections::HashMap;
use stylist::yew::styled_component;
use yew::prelude::*;
use yew_frontend::component::discord_icon::DiscordIcon;
use yew_frontend::component::github_icon::GithubIcon;
use yew_frontend::component::language_menu::LanguageMenu;
use yew_frontend::component::positioner::{Flex, Position, Positioner};
use yew_frontend::component::settings_icon::SettingsIcon;
use yew_frontend::component::volume_icon::VolumeIcon;
use yew_frontend::component::x_button::XButton;
//use yew_frontend::component::zoom_icon::ZoomIcon;
use yew_frontend::frontend::{use_gctw, use_outbound_enabled};
use yew_frontend::frontend::{use_rewarded_ad, PropertiesWrapper};
use yew_frontend::overlay::chat::ChatOverlay;
use yew_frontend::overlay::leaderboard::LeaderboardOverlay;
use yew_frontend::overlay::spawn::SpawnOverlay;
use yew_frontend::overlay::team::TeamOverlay;
use yew_frontend::translation::{use_translation, Translation};
use yew_router::{Routable, Switch};

mod about_dialog;
mod changelog_dialog;
mod faction_board;
mod hall_of_fame;
mod help_dialog;
mod hint;
mod instructions;
mod levels_dialog;
mod logo;
mod respawn_overlay;
mod settings_dialog;
mod ship_controls;
mod ship_menu;
mod ships_detail_dialog;
mod ships_dialog;
mod sprite;
mod status_overlay;
mod upgrade_overlay;

#[styled_component(Mk48Ui)]
pub fn mk48_ui(props: &PropertiesWrapper<UiProps>) -> Html {
    let cinematic_style = css!(
        r#"
        transition: opacity 0.25s;

        :not(:hover) {
		    opacity: 0;
	    }
    "#
    );

    let gctw = use_gctw::<Mk48Game>();
    let t = use_translation();
    let on_play = gctw.send_ui_event_callback.reform(|alias| {
        let is_arena = web_sys::window()
            .and_then(|w| w.location().search().ok())
            .map(|s| s.contains("arena"))
            .unwrap_or(false);
        let entity_type = if is_arena {
            EntityType::TypeViic
        } else {
            EntityType::G5
        };
        UiEvent::Spawn { alias, entity_type }
    });

    let margin = "0.75rem";
    let status = props.status.clone();
    let outbound_enabled = use_outbound_enabled();

    /*
       if (msg.includes('how')) {
           if (msg.includes('move')) {
               return 'If you are asking how you move, you click and hold (or right click) outside the inner ring of your ship to set your speed and direction (or use WASD)';
           }
           if (msg.includes('play')) {
               return '';
           }
           if (msg.includes('shoot') || msg.includes('use weapons') || msg.includes('fire')) {
               return '';
           }
       }
    */

    let shoot_hint = "First, select an available weapon. Then, click in the direction to fire. If you hold the click for too long, you won't shoot.";
    let hints = vec![
        ("Invitation links cannot currently be accepted by players that are already in game. They must send a join request instead.", vec!["/invite"]),
        ("If you are asking how you move, you click and hold to set your speed and direction (or use WASD).", vec!["how", "move"]),
        ("The controls are click and hold (or WASD) to move, click (or Space) to shoot.", vec!["how", "play"]),
        (shoot_hint, vec!["how", "shoot"]),
        (shoot_hint, vec!["how", "use weapons"]),
        (shoot_hint, vec!["how", "fire"])
    ];

    use yew_frontend::frontend::RewardedAd;
    use yew_icons::{Icon, IconId};
    let rewarded_ad = use_rewarded_ad();
    let rewarded_style = css!(
        r#"
        display: flex;
        flex-direction: row;
        align-items: center;
        gap: 0.5rem;
        background-color: #c0392b;
        border: 2px solid #e74c3c;
        border-radius: 0.5rem;
        color: white;
        padding: 0.25rem 0.5rem;
        font-size: 1rem;

        :disabled {
            filter: brightness(0.9);
        }
    "#
    );

    html! {
        <>
            if let UiStatus::Playing(playing) = status {
                <div class={classes!(gctw.settings_cache.cinematic.then_some(cinematic_style))}>
                    <Positioner id="status" position={Position::BottomMiddle{margin}} max_width="45%">
                        <StatusOverlay
                            status={playing.clone()}
                            score={props.score}
                            fps={gctw.settings_cache.fps_shown.then_some(props.fps)}
                        />
                    </Positioner>
                    <UpgradeOverlay
                        position={Position::TopMiddle{margin}}
                        status={playing.clone()}
                        score={props.score}
                    />
                    <ShipControls
                        position={Position::BottomLeft{margin}}
                        style="max-width:25%;"
                        status={playing.clone()}
                    />
                    <Positioner id="sidebar" position={Position::CenterRight{margin}} flex={Flex::Column}>
                        <VolumeIcon/>
                        <SettingsIcon<Mk48Route> route={Mk48Route::Settings}/>
                        <LanguageMenu/>
                    </Positioner>
                    <TeamOverlay
                        position={Position::TopLeft{margin}}
                        style="max-width:25%;"
                        team_proximity={playing.team_proximity.clone()}
                        label={LanguageId::team_fleet_label as fn(LanguageId) -> &'static str}
                        name_placeholder={LanguageId::team_fleet_name_placeholder as fn(LanguageId) -> &'static str}
                    />
                    <LeaderboardOverlay
                        position={Position::TopRight{margin}}
                        style="max-width:25%;"
                        bot_alliance_enabled={playing.bot_alliance_enabled}
                    />
                    if let (Some(ref fd), Some(mode)) = (&playing.faction_data, playing.faction_mode) {
                        <faction_board::FactionBoard
                            faction_data={fd.clone()}
                            my_faction={playing.my_faction}
                            faction_mode={mode}
                            altar_position={playing.altar_position}
                            altar_sacrifice_counts={playing.altar_sacrifice_counts.clone()}
                        />
                    }
                    if gctw.settings_cache.hall_of_fame {
                        <hall_of_fame::HallOfFame
                            score={props.score}
                            kill_log={playing.kill_log.clone()}
                            compact={true}
                        />
                    }
                    <ChatOverlay
                        position={Position::BottomRight{margin}}
                        style="max-width:25%;"
                        {hints}
                        label={LanguageId::chat_radio_label as fn(LanguageId) -> &'static str}
                        on_cheat={gctw.send_ui_event_callback.reform(|text: String| UiEvent::Cheat(text))}
                    />
                </div>
                if !gctw.settings_cache.cinematic {
                    <Hint entity_type={playing.entity_type}/>
                }
            } else if let UiStatus::Respawning(respawning) = status {
                <RespawnOverlay status={respawning.clone()} score={props.score}/>
                <div style="position: fixed; top: 50%; left: 50%; transform: translate(-50%, -50%); z-index: 10; margin-top: 120px;">
                    <hall_of_fame::HallOfFame
                        score={props.score}
                        kill_log={respawning.kill_log.clone()}
                        compact={false}
                    />
                </div>
                <Positioner position={Position::TopRight{margin}} max_width="25%">
                    <XButton onclick={gctw.send_ui_event_callback.reform(|_| UiEvent::OverrideRespawn)}/>
                </Positioner>
            } else {
                <SpawnOverlay {on_play}>
                    {logo()}
                </SpawnOverlay>
                <Positioner id="back" position={Position::TopRight{margin}} flex={Flex::Row}>
                    <LanguageMenu/>
                </Positioner>
                // ---- Mode Switch Buttons ----
                <div style="
                    position: fixed;
                    bottom: 2.5rem;
                    left: 50%;
                    transform: translateX(-50%);
                    display: flex;
                    gap: 0.8rem;
                    z-index: 100;
                ">
                    <button
                        id="arena_toggle"
                        style="
                            background: linear-gradient(135deg, #e84393, #fd79a8);
                            border: none;
                            border-radius: 2rem;
                            color: white;
                            cursor: pointer;
                            font-size: 1rem;
                            font-weight: bold;
                            padding: 0.5em 1.4em;
                            box-shadow: 0 4px 15px rgba(232, 67, 147, 0.4);
                            transition: all 0.3s ease;
                            letter-spacing: 0.05em;
                        "
                        onclick={Callback::from(|_| {
                            let window = web_sys::window().unwrap();
                            let location = window.location();
                            let search = location.search().unwrap_or_default();
                            let has_arena = search.contains("arena");
                            let new_url = if has_arena {
                                "/".to_string()
                            } else {
                                "/?arena".to_string()
                            };
                            let _ = location.set_href(&new_url);
                        })}
                    >
                        {
                            if web_sys::window().unwrap().location().search().unwrap_or_default().contains("arena") {
                                "\u{2693} Normal"
                            } else {
                                "\u{2694} Arena"
                            }
                        }
                    </button>
                    <button
                        id="faction_mode_toggle"
                        style="
                            background: linear-gradient(135deg, #0984e3, #74b9ff);
                            border: none;
                            border-radius: 2rem;
                            color: white;
                            cursor: pointer;
                            font-size: 1rem;
                            font-weight: bold;
                            padding: 0.5em 1.4em;
                            box-shadow: 0 4px 15px rgba(9, 132, 227, 0.4);
                            transition: all 0.3s ease;
                            letter-spacing: 0.05em;
                        "
                        onclick={gctw.send_ui_event_callback.reform(|_| UiEvent::CycleFactionMode)}
                    >
                        {
                            match props.faction_mode {
                                Some(common::protocol::FactionMode::TwoTeam) => "\u{2694}\u{fe0f} \u{8230}\u{961f}\u{5bf9}\u{6297}",
                                Some(common::protocol::FactionMode::ThreeTeam) => "\u{1f6e1}\u{fe0f} \u{9635}\u{8425}\u{6218}\u{4e89}",
                                None => "\u{2694}\u{fe0f} \u{65e0}\u{9635}\u{8425}",
                            }
                        }
                    </button>
                    <button
                        id="hardcore_toggle"
                        style={format!("
                            background: linear-gradient(135deg, {}, {});
                            border: none;
                            border-radius: 2rem;
                            color: white;
                            cursor: pointer;
                            font-size: 1rem;
                            font-weight: bold;
                            padding: 0.5em 1.4em;
                            box-shadow: 0 4px 15px rgba(214, 48, 49, 0.4);
                            transition: all 0.3s ease;
                            letter-spacing: 0.05em;
                        ",
                            if props.hardcore_mode { "#d63031" } else { "#636e72" },
                            if props.hardcore_mode { "#ff7675" } else { "#b2bec3" },
                        )}
                        onclick={gctw.send_ui_event_callback.reform(|_| UiEvent::ToggleHardcoreMode)}
                    >
                        {
                            if props.hardcore_mode {
                                "\u{1f480} \u{6781}\u{9650}\u{6a21}\u{5f0f}"
                            } else {
                                "\u{1f480} \u{6781}\u{9650}\u{5173}"
                            }
                        }
                    </button>
                </div>
            }
            if !matches!(props.status, UiStatus::Playing(_)) {
                if outbound_enabled {
                    <Positioner id="social" position={Position::BottomRight{margin}} flex={Flex::Row}>
                        <DiscordIcon/>
                        <GithubIcon repository_link={"https://github.com/Sheumais/mk48-plus"}/>
                    </Positioner>
                }
                if !matches!(rewarded_ad, RewardedAd::Unavailable) {
                    <button
                        id="rewarded"
                        onclick={if let RewardedAd::Available{request} = &rewarded_ad { Some(request.reform(|_| {})) } else { None }}
                        disabled={!matches!(rewarded_ad, RewardedAd::Available{..})}
                        style={Position::TopLeft{margin}.to_string()}
                        class={rewarded_style}
                    >
                        <Icon icon_id={IconId::OcticonsVideo16}/>
                        {t.rewarded_ad(&rewarded_ad)}
                    </button>
                }
            }
            <Switch<Mk48Route> render={switch}/>
        </>
    }
}

#[derive(Debug, Clone, Copy, PartialEq, SmolRoutable)]
pub enum Mk48Route {
    #[at("/about/")]
    About,
    #[at("/changelog/")]
    Changelog,
    #[at("/help/")]
    Help,
    #[at("/ships_detail/")]
    ShipsDetail,
    #[at("/levels/")]
    Levels,
    #[at("/settings/")]
    Settings,
    #[not_found]
    #[at("/")]
    Home,
}

/// State of UI inputs.
pub struct UiState {
    pub active: bool,
    pub submerge: bool,
    pub armament: Option<EntityType>,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            active: true,
            submerge: false,
            armament: None,
        }
    }
}

#[derive(Clone)]
pub enum UiEvent {
    /// Sensors active.
    Active(bool),
    Armament(Option<EntityType>),
    GraphicsSettingsChanged,
    /// Go from respawning to spawning.
    #[allow(unused)]
    OverrideRespawn,
    Respawn(EntityType),
    Spawn {
        alias: PlayerAlias,
        entity_type: EntityType,
    },
    Submerge(bool),
    Upgrade(EntityType),
    UseSkill(common::skill::SkillType),
    CycleFactionMode,
    ToggleHardcoreMode,
    Cheat(String),
}

#[derive(PartialEq, Clone, Default)]
pub struct UiProps {
    pub fps: f32,
    pub score: u32,
    pub arena_mode: bool,
    pub faction_mode: Option<common::protocol::FactionMode>,
    pub hardcore_mode: bool,
    pub status: UiStatus,
}

/// Mutually exclusive statuses.
#[derive(Default, PartialEq, Clone)]
pub enum UiStatus {
    #[default]
    Spawning,
    Playing(UiStatusPlaying),
    Respawning(UiStatusRespawning),
}

#[derive(PartialEq, Clone)]
pub struct UiStatusPlaying {
    pub entity_type: EntityType,
    pub velocity: Velocity,
    pub direction: Angle,
    pub position: Vec2,
    pub altitude: Altitude,
    pub submerge: bool,
    /// Active sensors.
    pub active: bool,
    pub instruction_status: InstructionStatus,
    pub armament: Option<EntityType>,
    pub armament_consumption: Box<[bool]>,
    pub team_proximity: HashMap<TeamId, f32>,
    pub pending_skill: Option<common::skill::SkillType>,
    pub skills: Vec<common::protocol::SkillSnapshot>,
    /// Whether bot alliance mode is enabled.
    pub bot_alliance_enabled: bool,
    /// Faction war data.
    pub faction_data: Option<common::protocol::FactionUpdate>,
    /// Current player's faction.
    pub my_faction: Option<common::protocol::FactionId>,
    /// Known altar position for this player's faction.
    pub altar_position: Option<glam::Vec2>,
    /// Per-faction sacrifice counts.
    pub altar_sacrifice_counts: Vec<u8>,
    /// Kill log for hall of fame display.
    pub kill_log: Vec<(common::entity::EntityType, u32)>,
    /// Whether in arena mode.
    pub arena_mode: bool,
    /// Current faction mode.
    pub faction_mode: Option<common::protocol::FactionMode>,
}

/// Skill runtime state for UI rendering.
#[derive(PartialEq, Clone, Debug)]
pub enum SkillState {
    Ready,
    Selecting,
    Charging(f32),
    Active(f32),
    Cooling(f32),
}

impl UiStatusPlaying {
    fn skill_snapshot(
        &self,
        skill: common::skill::SkillType,
    ) -> Option<&common::protocol::SkillSnapshot> {
        self.skills.iter().find(|snapshot| snapshot.skill == skill)
    }

    /// Query the current state of a skill for UI rendering.
    pub fn get_skill_state(&self, skill: common::skill::SkillType) -> SkillState {
        let selecting = self.pending_skill == Some(skill);
        if selecting {
            SkillState::Selecting
        } else if let Some(snapshot) = self.skill_snapshot(skill) {
            if snapshot.charge_remaining > common::ticks::Ticks::ZERO {
                SkillState::Charging(snapshot.charge_remaining.to_secs())
            } else if snapshot.active_remaining > common::ticks::Ticks::ZERO {
                SkillState::Active(snapshot.active_remaining.to_secs())
            } else if snapshot.cooldown_remaining > common::ticks::Ticks::ZERO {
                SkillState::Cooling(snapshot.cooldown_remaining.to_secs())
            } else {
                SkillState::Ready
            }
        } else {
            SkillState::Ready
        }
    }
}

#[derive(PartialEq, Clone)]
pub struct UiStatusRespawning {
    pub death_reason: DeathReason,
    /// Kill log snapshot at death for hall of fame display.
    pub kill_log: Vec<(EntityType, u32)>,
    /// Whether in arena mode.
    pub arena_mode: bool,
}

impl Mk48Game {
    pub(crate) fn update_ui_props(&self, context: &mut Context<Self>, status: UiStatus) {
        let props = UiProps {
            fps: self.fps_counter.last_sample().unwrap_or(0.0),
            score: context.state.game.score,
            arena_mode: context.state.game.arena_mode,
            faction_mode: context.state.game.faction_mode,
            hardcore_mode: self.hardcore_mode_local,
            status,
        };

        context.set_ui_props(props);
    }
}

fn switch(routes: Mk48Route) -> Html {
    match routes {
        Mk48Route::About => html! {
            <AboutDialog/>
        },
        Mk48Route::Changelog => html! {
            <ChangelogDialog/>
        },
        Mk48Route::Help => html! {
            <HelpDialog/>
        },
        // Ships tab intentionally omitted from the switch to hide it from UI navigation.
        Mk48Route::ShipsDetail => html! {
            <ShipsDetailDialog/>
        },
        Mk48Route::Levels => html! {
            <LevelsDialog/>
        },
        Mk48Route::Settings => html! {
            <SettingsDialog/>
        },
        Mk48Route::Home => html! {},
    }
}
