// SPDX-FileCopyrightText: 2021 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::armament::{group_armaments, Group};
use crate::translation::Mk48Translation;
use crate::ui::sprite::Sprite;
use common::altitude::Altitude;
use common::entity::{EntityData, EntityKind, EntitySubKind, EntityType};
use common::ticks::Ticks;
use common::velocity::Velocity;
use core_protocol::id::LanguageId;
use stylist::yew::styled_component;
use stylist::StyleSource;
use web_sys::window;
use yew::{html, html_nested, use_effect_with_deps, Html};
use yew_frontend::component::link::Link;
use yew_frontend::dialog::dialog::Dialog;
use yew_frontend::translation::use_translation;

#[styled_component(ShipsDetailDialog)]
pub fn ships_detail_dialog() -> Html {
    let t = use_translation();
    let table_style = css!(
        r#"
        border-spacing: 1em;
		text-align: left;
		width: 100%;
		"#
    );

    // 读取 hash 作为目标 ID。
    let mut target_id: Option<String> = None;
    if let Some(id) = window()
        .and_then(|w| w.location().hash().ok())
        .map(|h| h.trim_start_matches('#').to_string())
    {
        if !id.is_empty() {
            target_id = Some(id);
        }
    }

    // 默认第一个船只
    let entity = target_id
        .as_deref()
        .and_then(|id| EntityType::iter().find(|e| e.as_str() == id && e.data().kind == EntityKind::Boat))
        .or_else(|| EntityType::iter().find(|e| e.data().kind == EntityKind::Boat));

    // 与原 Ships 页一致：页面加载后滚动到该行（此处仅一个实体，scroll 作用有限）。
    use_effect_with_deps(
        move |_| {
            if let Some(hash) = window().and_then(|w| w.location().hash().ok()) {
                let id = hash.trim_start_matches('#');
                if !id.is_empty() {
                    if let Some(elem) = window()
                        .and_then(|w| w.document())
                        .and_then(|d| d.get_element_by_id(id))
                    {
                        elem.scroll_into_view()
                    }
                }
            }
            || ()
        },
        (),
    );

    html! {
        <Dialog title={"Ship Detail"}>
            {
                if let Some(entity_type) = entity {
                    html! {
                        <table>
                            <tr id={entity_type.as_str()}>
                                <td>
                                    {entity_card(t, &table_style, entity_type, None)}
                                </td>
                            </tr>
                        </table>
                    }
                } else {
                    html! { <p>{format!("未找到实体（hash）: {}", target_id.unwrap_or_else(|| "<空>".into()))}</p> }
                }
            }
        </Dialog>
    }
}

fn entity_card(
    t: LanguageId,
    table_style: &StyleSource,
    entity_type: EntityType,
    count: Option<u8>,
) -> Html {
    let data: &'static EntityData = entity_type.data();
    html! {
        <table class={table_style.clone()}>
            <tr>
            if data.sub_kind != EntitySubKind::Drone {
                <td>
                    <h3>
                        {data.label.clone()}
                        if let Some(count) = count {
                            {format!(" × {count}")}
                        }
                    </h3>
                    <i>
                        if data.kind == EntityKind::Boat {
                            {format!("Level {} ", data.level)}
                        }
                        {t.entity_kind_name(data.kind, data.sub_kind)}
                    </i>
                    if let Some(href) = data.link.clone() {
                        {" ("} <Link {href}>{"Learn more"}</Link>{")"}
                    }
                </td>
                <td rowspan="2">
                    <ul>
                        if data.length != 0.0 {
                            <li>{format!("Length: {:.1}m", data.length)}</li>
                        }
                        if data.draft != Altitude::ZERO {
                            <li>{format!("Draft: {:.1}m", data.draft.to_meters())}</li>
                        }
                        if data.speed != Velocity::ZERO {
                            <li>{format!("Speed: {:.1}m/s ({:.1}kn)", data.speed.to_mps(), data.speed.to_knots())}</li>
                        }
                        if data.range != 0.0 {
                            <li>{format!("Range: {}m", data.range as u32)}</li>
                        }
                        if data.depth != Altitude::ZERO {
                            <li>{format!("Max Depth: {}m", data.depth.to_meters() as u16)}</li>
                        }
                        if data.lifespan != Ticks::ZERO {
                            <li>{format!("Lifespan: {:.1}s", data.lifespan.to_secs())}</li>
                        }
                        if data.reload != Ticks::ZERO {
                            <li>{format!("Reload: {:.1}s", data.reload.to_secs())}</li>
                        }
                        if data.damage != 0.0 {
                            <li>{format!("{}: {:.2}", if data.kind == EntityKind::Boat { "Health" } else { "Damage" }, data.damage)}</li>
                        }
                        if data.anti_aircraft != 0.0 {
                            <li>{format!("Anti-Aircraft: {:.2}", data.anti_aircraft)}</li>
                        }
                        if data.torpedo_resistance != 0.0 {
                            <li>{format!("Torpedo Resistance: {}%", (data.torpedo_resistance * 100.0) as u16)}</li>
                        }
                        if data.stealth != 0.0 {
                            <li>{format!("Stealth: {}%", (data.stealth * 100.0) as u16)}</li>
                        }
                        if data.npc {
                            <li>{"NPC only"}</li>
                        }
                    </ul>
                </td>}
                else {
                    <td>
                        <h3>
                            {"???"}
                        </h3>
                </td>}
            </tr>
            <tr>
                <td>
                    <Sprite {entity_type}/>
                </td>
            </tr>
            {group_armaments(&data.armaments, &[]).into_iter().map(|Group{entity_type, total, ..}| html_nested!{
                <tr>
                    <td colspan="2">
                        {entity_card(t, table_style, entity_type, Some(total))}
                    </td>
                </tr>
            }).collect::<Html>()}
        </table>
    }
}
