// SPDX-FileCopyrightText: 2021 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::ui::sprite::Sprite;
use crate::Mk48Route;
use common::entity::{EntityData, EntityKind, EntitySubKind, EntityType};
use common::util::level_to_score;
use stylist::yew::styled_component;
use yew::{html, html_nested, Callback, Html};
use yew_router::prelude::use_navigator;
use yew_frontend::component::positioner::Align;
use yew_frontend::dialog::dialog::Dialog;
use yew_frontend::translation::{use_translation, Translation};

#[styled_component(LevelsDialog)]
pub fn levels_dialog() -> Html {
    let sprite_style = css!(
        r#"
        display: inline-block;
		margin: 0.5em;
        "#
    );

    let t = use_translation();
    let navigator = use_navigator();

    html! {
        <Dialog title={"Levels"} align={Align::Center}>
            {(1..=EntityData::MAX_BOAT_LEVEL).map(move |level| html_nested!{
                <div>
                    <h3>{format!("Level {} ({})", level, t.score(level_to_score(level)))}</h3>
                    {EntityType::iter().filter(move |entity_type| entity_type.data().kind == EntityKind::Boat && entity_type.data().level == level && entity_type.data().sub_kind != EntitySubKind::Drone).map(|entity_type| {
                        let id = entity_type.as_str().to_string();
                        let on_click = {
                            let navigator = navigator.clone();
                            let id = id.clone();
                            Callback::from(move |_| {
                                if let Some(nav) = navigator.clone() {
                                    nav.push(&Mk48Route::ShipsDetail);
                                }
                                if let Some(win) = web_sys::window() {
                                    let _ = win.location().set_hash(&id);
                                }
                            })
                        };
                        html_nested! {
                            <a onclick={on_click} style="display: inline-block; cursor: pointer;">
                                <Sprite {entity_type} class={sprite_style.clone()}/>
                            </a>
                        }
                    }).collect::<Html>()}
                </div>
            }).collect::<Html>()}
        </Dialog>
    }
}
