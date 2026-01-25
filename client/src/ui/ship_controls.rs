// SPDX-FileCopyrightText: 2021 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::armament::{group_armaments, Group};
use crate::translation::Mk48Translation;
use crate::ui::sprite::Sprite;
use crate::ui::{UiEvent, UiStatusPlaying};
use crate::Mk48Game;
use common::altitude::Altitude;
use common::entity::{EntityData, EntitySubKind, EntityType};
use core_protocol::id::LanguageId;
use stylist::yew::styled_component;
use stylist::{css, StyleSource};
use web_sys::MouseEvent;
use yew::{classes, html, html_nested, AttrValue, Callback, Html, Properties};
use yew_frontend::component::positioner::Position;
use yew_frontend::component::section::Section;
use yew_frontend::frontend::use_ui_event_callback;
use yew_frontend::translation::use_translation;

#[derive(Properties, PartialEq)]
pub struct ShipControlsProps {
    pub position: Position,
    #[prop_or(None)]
    pub style: Option<AttrValue>,
    pub status: UiStatusPlaying,
}

#[styled_component(ShipControls)]
pub fn ship_controls(props: &ShipControlsProps) -> Html {
    let button_style = css!(
        r#"
        color: white;
		padding: 0.5em;
		filter: brightness(0.8);
		user-select: none;
		cursor: pointer;

		:hover {
            background-color: #44444440;
            filter: brightness(0.9);
        }
    "#
    );

    // !important to override the :hover.
    let button_selected_style = css!(
        r#"
        background-color: #44444480 !important;
        cursor: default;
        filter: brightness(1.2) !important;
        padding: 0.5em;
        "#
    );

    let consumption_style = css!(
        r#"
        float: right;
		color: white;
    "#
    );

    let consumed_style = css!(
        r#"
        opacity: 0.6;
        "#
    );

    let disabled_style = css!(
        r#"
        opacity: 0.5;
        cursor: not-allowed;
        "#
    );

    let warp_image_style = css!(
        r#"
        filter: hue-rotate(60deg) brightness(1.2);
        "#
    );

    let zero_pulse_image_style = css!(
        r#"
        filter: hue-rotate(200deg) brightness(1.25);
        "#
    );

    let zero_pulse_label_style = css!(
        r#"
        display: block;
        margin-left: 0.5em;
        font-size: 0.85em;
        opacity: 0.85;
        "#,
    );

    let data: &'static EntityData = props.status.entity_type.data();

    let ui_event_callback = use_ui_event_callback::<Mk48Game>();
    let select_factory = {
        let ui_event_callback = ui_event_callback.clone();
        move |entity_type: EntityType| {
            (!props.status.armament.contains(&entity_type)).then(move || {
                ui_event_callback.reform(move |_: MouseEvent| UiEvent::Armament(Some(entity_type)))
            })
        }
    };

    let t = use_translation();
    let status = &props.status;
    html! {
        <Section id="controls" name={data.label.clone()} position={props.position} style={props.style.clone()} closable={false}>
            if status.entity_type.data().armaments.is_empty() {
                <small>{t.entity_kind_hint(status.entity_type.data().kind, status.entity_type.data().sub_kind)}</small>
            } else {
                {group_armaments(&status.entity_type.data().armaments, &*status.armament_consumption).into_iter().map(|Group{entity_type, total, ready}| {
                    let onclick = select_factory.clone()(entity_type);
                    html_nested!{
                        <div class={classes!(button_style.clone(), onclick.is_none().then(|| button_selected_style.clone()))} {onclick}>
                            <Sprite {entity_type} class={classes!((ready == 0).then(|| consumed_style.clone()))}/>
                            <span class={consumption_style.clone()}>{format!("{ready}/{total}")}</span>
                        </div>
                    }
                }).collect::<Html>()}
            }
            {surface_button(t, props.status.entity_type, props.status.submerge, &button_style, &button_selected_style, &ui_event_callback)}
            {active_sensor_button(t, props.status.entity_type, props.status.active, props.status.altitude, &button_style, &button_selected_style, &ui_event_callback)}
            {warp_button(props.status.clone(), &button_style, &button_selected_style, &disabled_style, &warp_image_style, &consumption_style, &ui_event_callback)}
            {zero_pulse_button(props.status.clone(), &button_style, &disabled_style, &zero_pulse_image_style, &zero_pulse_label_style, &ui_event_callback)}
            {iaigiri_button(props.status.clone(), &button_style, &button_selected_style, &disabled_style, &ui_event_callback)}
            {engine_boost_button(props.status.clone(), &button_style, &button_selected_style, &disabled_style, &ui_event_callback)}
            {sonar_pulse_button(props.status.clone(), &button_style, &disabled_style, &ui_event_callback)}
            {depth_charge_barrage_button(props.status.clone(), &button_style, &disabled_style, &ui_event_callback)}
            {air_superiority_button(props.status.clone(), &button_style, &disabled_style, &ui_event_callback)}
            {emergency_repair_button(props.status.clone(), &button_style, &disabled_style, &ui_event_callback)}
            {smoke_screen_button(props.status.clone(), &button_style, &disabled_style, &ui_event_callback)}
            {burst_loading_button(props.status.clone(), &button_style, &disabled_style, &ui_event_callback)}
            {nuclear_strike_button(props.status.clone(), &button_style, &disabled_style, &ui_event_callback)}
            {energy_shield_button(props.status.clone(), &button_style, &disabled_style, &ui_event_callback)}
        </Section>
    }
}

fn surface_button(
    t: LanguageId,
    entity_type: EntityType,
    submerge: bool,
    button_style: &StyleSource,
    button_selected_style: &StyleSource,
    ui_event_callback: &Callback<UiEvent>,
) -> Html {
    if entity_type.data().sub_kind != EntitySubKind::Submarine {
        Html::default()
    } else {
        let onclick = ui_event_callback.reform(move |_: MouseEvent| UiEvent::Submerge(!submerge));
        let surface_or_dive = if submerge {
            t.ship_surface_label()
        } else {
            t.ship_dive_label()
        };

        html! {
            <div class={classes!(button_style.clone(), (submerge).then(|| button_selected_style.clone()))} {onclick} title={t.ship_surface_hint()}>
                {surface_or_dive}
            </div>
        }
    }
}

fn active_sensor_button(
    t: LanguageId,
    entity_type: EntityType,
    active: bool,
    altitude: Altitude,
    button_style: &StyleSource,
    button_selected_style: &StyleSource,
    ui_event_callback: &Callback<UiEvent>,
) -> Html {
    let data: &'static EntityData = entity_type.data();
    let sensors = &data.sensors;
    if !(sensors.radar.range > 0.0 || sensors.sonar.range > 0.0) {
        Html::default()
    } else {
        let sensors = (sensors.radar.range >= 0.0 && !altitude.is_submerged())
            .then(|| t.sensor_radar_label())
            .into_iter()
            .chain(
                (sensors.sonar.range >= 0.0 && !altitude.is_airborne())
                    .then(|| t.sensor_sonar_label()),
            )
            .intersperse(" / ")
            .collect::<String>();
        let title = t.sensor_active_hint(&sensors);
        let onclick = ui_event_callback.reform(move |_: MouseEvent| UiEvent::Active(!active));

        html! {
            <div class={classes!(button_style.clone(), active.then(|| button_selected_style.clone()))} {onclick} {title}>
                {t.sensor_active_label()}
            </div>
        }
    }
}

fn warp_button(
    status: UiStatusPlaying,
    button_style: &StyleSource,
    button_selected_style: &StyleSource,
    disabled_style: &StyleSource,
    warp_image_style: &StyleSource,
    consumption_style: &StyleSource,
    ui_event_callback: &Callback<UiEvent>,
) -> Html {
    if status.entity_type != EntityType::StarDestroyer
        && status.entity_type != EntityType::XystonStarDestroyer
        && status.entity_type != EntityType::UnscInfinite
        && status.entity_type != EntityType::StellarFrigate
    {
        return Html::default();
    }

    let charging = status.warp_charge_remaining > 0.0;
    let cooling = status.warp_cooldown_remaining > 0.0;
    let selecting = status.warp_selecting && !charging && !cooling;

    let label = if charging {
        format!("跃迁充能 {:.1}s", status.warp_charge_remaining)
    } else if cooling {
        format!("冷却 {:.1}s", status.warp_cooldown_remaining)
    } else if selecting {
        "选择跃迁点".to_string()
    } else {
        "空间跃迁".to_string()
    };

    let onclick = (!charging && !cooling)
        .then(|| ui_event_callback.reform(|_: MouseEvent| UiEvent::WarpToggle));

    html! {
        <div class={classes!(button_style.clone(), selecting.then(|| button_selected_style.clone()), (charging || cooling).then(|| disabled_style.clone()))} {onclick} title={"在视野内选定跃迁目标，3秒充能后瞬移"}>
            <Sprite entity_type={EntityType::GreenBlaster} image_class={classes!(warp_image_style.clone())}/>
            <span class={consumption_style.clone()}>{label}</span>
        </div>
    }
}

fn zero_pulse_button(
    status: UiStatusPlaying,
    button_style: &StyleSource,
    disabled_style: &StyleSource,
    zero_pulse_image_style: &StyleSource,
    zero_pulse_label_style: &StyleSource,
    ui_event_callback: &Callback<UiEvent>,
) -> Html {
    if status.entity_type != EntityType::Leviathan
        && status.entity_type != EntityType::StarDestroyer
    {
        return Html::default();
    }

    let cooling = status.zero_pulse_cooldown_remaining > 0.0;
    let label = if cooling {
        format!("冷却 {:.1}s", status.zero_pulse_cooldown_remaining)
    } else {
        "就绪".to_string()
    };

    let onclick = (!cooling).then(|| ui_event_callback.reform(|_: MouseEvent| UiEvent::ZeroPulse));

    html! {
        <div class={classes!(button_style.clone(), cooling.then(|| disabled_style.clone()))} {onclick} title={"Q 触发，冻结范围内敌方目标"}>
            <Sprite entity_type={EntityType::Blaster} image_class={classes!(zero_pulse_image_style.clone())}/>
            <span class={zero_pulse_label_style.clone()}>{format!("绝对零度 · {}", label)}</span>
        </div>
    }
}

fn iaigiri_button(
    status: UiStatusPlaying,
    button_style: &StyleSource,
    button_selected_style: &StyleSource,
    disabled_style: &StyleSource,
    ui_event_callback: &Callback<UiEvent>,
) -> Html {
    if status.entity_type != EntityType::Minelayer49 {
        return Html::default();
    }

    let cooling = status.iaigiri_cooldown_remaining > 0.0;
    let selecting = status.iaigiri_selecting && !cooling;

    let label = if cooling {
        format!("冷却 {:.1}s", status.iaigiri_cooldown_remaining)
    } else if selecting {
        "选择目标点".to_string()
    } else {
        "就绪".to_string()
    };

    let onclick = (!cooling).then(|| ui_event_callback.reform(|_: MouseEvent| UiEvent::IaigiriToggle));

    html! {
        <div class={classes!(button_style.clone(), selecting.then(|| button_selected_style.clone()), cooling.then(|| disabled_style.clone()))} {onclick} title={"J 触发，瞬移并在路径上布置水雷"}>
            <Sprite entity_type={EntityType::IaigiriMine}/>
            <span style="margin-left:0.5em;font-size:0.85em;">{"居合斩 · "}{label}</span>
        </div>
    }
}

fn engine_boost_button(
    status: UiStatusPlaying,
    button_style: &StyleSource,
    button_selected_style: &StyleSource,
    disabled_style: &StyleSource,
    ui_event_callback: &Callback<UiEvent>,
) -> Html {
    if status.entity_type != EntityType::Minelayer49 {
        return Html::default();
    }

    let boosting = status.engine_boost_remaining > 0.0;
    let cooling = status.engine_boost_cooldown_remaining > 0.0;
    let disabled = boosting || cooling;

    let label = if boosting {
        format!("加速中 {:.1}s", status.engine_boost_remaining)
    } else if cooling {
        format!("冷却 {:.1}s", status.engine_boost_cooldown_remaining)
    } else {
        "就绪".to_string()
    };

    let onclick = (!disabled).then(|| ui_event_callback.reform(|_: MouseEvent| UiEvent::EngineBoostToggle));

    html! {
        <div class={classes!(button_style.clone(), boosting.then(|| button_selected_style.clone()), disabled.then(|| disabled_style.clone()))} {onclick} title={"K 触发，速度提升至106节"}>
            <span>{"引擎增压 · "}{label}</span>
        </div>
    }
}

fn sonar_pulse_button(
    status: UiStatusPlaying,
    button_style: &StyleSource,
    disabled_style: &StyleSource,
    ui_event_callback: &Callback<UiEvent>,
) -> Html {
    if status.entity_type != EntityType::HunterKiller77 {
        return Html::default();
    }

    let cooling = status.sonar_pulse_cooldown_remaining > 0.0;
    let label = if cooling {
        format!("冷却 {:.1}s", status.sonar_pulse_cooldown_remaining)
    } else {
        "就绪".to_string()
    };

    let onclick = (!cooling).then(|| ui_event_callback.reform(|_: MouseEvent| UiEvent::SonarPulse));

    html! {
        <div class={classes!(button_style.clone(), cooling.then(|| disabled_style.clone()))} {onclick} title={"J 触发，1500m范围内探测潜航潜艇"}>
            <span>{"主动声纳 · "}{label}</span>
        </div>
    }
}

fn depth_charge_barrage_button(
    status: UiStatusPlaying,
    button_style: &StyleSource,
    disabled_style: &StyleSource,
    ui_event_callback: &Callback<UiEvent>,
) -> Html {
    if status.entity_type != EntityType::HunterKiller77 {
        return Html::default();
    }

    let cooling = status.depth_charge_barrage_cooldown_remaining > 0.0;
    let label = if cooling {
        format!("冷却 {:.1}s", status.depth_charge_barrage_cooldown_remaining)
    } else {
        "就绪".to_string()
    };

    let onclick = (!cooling).then(|| ui_event_callback.reform(|_: MouseEvent| UiEvent::DepthChargeBarrage));

    html! {
        <div class={classes!(button_style.clone(), cooling.then(|| disabled_style.clone()))} {onclick} title={"K 触发，120度扇形发射12枚深弹"}>
            <span>{"深弹齐射 · "}{label}</span>
        </div>
    }
}

fn air_superiority_button(
    status: UiStatusPlaying,
    button_style: &StyleSource,
    disabled_style: &StyleSource,
    ui_event_callback: &Callback<UiEvent>,
) -> Html {
    if status.entity_type != EntityType::FortressCarrier {
        return Html::default();
    }

    let cooling = status.air_superiority_cooldown_remaining > 0.0;
    let label = if cooling {
        format!("冷却 {:.1}s", status.air_superiority_cooldown_remaining)
    } else {
        "就绪".to_string()
    };

    let onclick = (!cooling).then(|| ui_event_callback.reform(|_: MouseEvent| UiEvent::AirSuperiority));

    html! {
        <div class={classes!(button_style.clone(), cooling.then(|| disabled_style.clone()))} {onclick} title={"J 触发，释放10架无人机攻击敌人"}>
            <span>{"制空权 · "}{label}</span>
        </div>
    }
}

fn emergency_repair_button(
    status: UiStatusPlaying,
    button_style: &StyleSource,
    disabled_style: &StyleSource,
    ui_event_callback: &Callback<UiEvent>,
) -> Html {
    if status.entity_type != EntityType::FortressCarrier
        && status.entity_type != EntityType::Battleship750k
    {
        return Html::default();
    }

    let cooling = status.emergency_repair_cooldown_remaining > 0.0;
    let label = if cooling {
        format!("冷却 {:.1}s", status.emergency_repair_cooldown_remaining)
    } else {
        "就绪".to_string()
    };

    let onclick = (!cooling).then(|| ui_event_callback.reform(|_: MouseEvent| UiEvent::EmergencyRepair));

    html! {
        <div class={classes!(button_style.clone(), cooling.then(|| disabled_style.clone()))} {onclick} title={"K 触发，15秒内恢复20%生命值"}>
            <span>{"紧急维修 · "}{label}</span>
        </div>
    }
}

fn smoke_screen_button(
    status: UiStatusPlaying,
    button_style: &StyleSource,
    disabled_style: &StyleSource,
    ui_event_callback: &Callback<UiEvent>,
) -> Html {
    if status.entity_type != EntityType::Tianwangxing
        && status.entity_type != EntityType::Richelieu
        && status.entity_type != EntityType::Battleship750k
    {
        return Html::default();
    }

    let active = status.smoke_screen_active_remaining > 0.0;
    let cooling = status.smoke_screen_cooldown_remaining > 0.0 && !active;
    
    let label = if active {
        format!("生效中 {:.1}s", status.smoke_screen_active_remaining)
    } else if cooling {
        format!("冷却 {:.1}s", status.smoke_screen_cooldown_remaining)
    } else {
        "就绪".to_string()
    };

    let disabled = active || cooling;
    let onclick = (!disabled).then(|| ui_event_callback.reform(|_: MouseEvent| UiEvent::SmokeScreen));

    html! {
        <div class={classes!(button_style.clone(), disabled.then(|| disabled_style.clone()))} {onclick} title={"L 触发，释放持续30秒的烟幕干扰敌方制导"}>
            <span>{"烟幕 · "}{label}</span>
        </div>
    }
}

fn burst_loading_button(
    status: UiStatusPlaying,
    button_style: &StyleSource,
    disabled_style: &StyleSource,
    ui_event_callback: &Callback<UiEvent>,
) -> Html {
    if status.entity_type != EntityType::Richelieu {
        return Html::default();
    }

    let active = status.burst_loading_active_remaining > 0.0;
    let cooling = status.burst_loading_cooldown_remaining > 0.0 && !active;
    
    let label = if active {
        format!("生效中 {:.1}s", status.burst_loading_active_remaining)
    } else if cooling {
        format!("冷却 {:.1}s", status.burst_loading_cooldown_remaining)
    } else {
        "就绪".to_string()
    };

    let disabled = active || cooling;
    let onclick = (!disabled).then(|| ui_event_callback.reform(|_: MouseEvent| UiEvent::BurstLoading));

    html! {
        <div class={classes!(button_style.clone(), disabled.then(|| disabled_style.clone()))} {onclick} title={"B 触发，30秒内武器装填速度提升200倍"}>
            <span>{"爆发装填 · "}{label}</span>
        </div>
    }
}

fn nuclear_strike_button(
    status: UiStatusPlaying,
    button_style: &StyleSource,
    disabled_style: &StyleSource,
    ui_event_callback: &Callback<UiEvent>,
) -> Html {
    if status.entity_type != EntityType::UnscInfinite {
        return Html::default();
    }

    let cooling = status.nuclear_strike_cooldown_remaining > 0.0;
    
    let label = if cooling {
        format!("冷却 {:.1}s", status.nuclear_strike_cooldown_remaining)
    } else {
        "就绪".to_string()
    };

    let disabled = cooling;
    let onclick = (!disabled).then(|| ui_event_callback.reform(|_: MouseEvent| UiEvent::NuclearStrike));

    html! {
        <div class={classes!(button_style.clone(), disabled.then(|| disabled_style.clone()))} {onclick} title={"核打击 - 1000米范围内敌人全灭，120秒冷却"}>
            <span>{"☢ 核打击 · "}{label}</span>
        </div>
    }
}

fn energy_shield_button(
    status: UiStatusPlaying,
    button_style: &StyleSource,
    disabled_style: &StyleSource,
    ui_event_callback: &Callback<UiEvent>,
) -> Html {
    if status.entity_type != EntityType::StellarFrigate {
        return Html::default();
    }

    let active = status.energy_shield_active_remaining > 0.0;
    let cooling = status.energy_shield_cooldown_remaining > 0.0 && !active;
    
    let label = if active {
        format!("生效中 {:.1}s", status.energy_shield_active_remaining)
    } else if cooling {
        format!("冷却 {:.1}s", status.energy_shield_cooldown_remaining)
    } else {
        "就绪".to_string()
    };

    let disabled = active || cooling;
    let onclick = (!disabled).then(|| ui_event_callback.reform(|_: MouseEvent| UiEvent::EnergyShield));

    html! {
        <div class={classes!(button_style.clone(), disabled.then(|| disabled_style.clone()))} {onclick} title={"能量护盾 - 8秒内吸收90%伤害，45秒冷却"}>
            <span>{"🛡 能量护盾 · "}{label}</span>
        </div>
    }
}
