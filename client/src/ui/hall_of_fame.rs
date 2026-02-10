// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

use common::entity::EntityType;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::MouseEvent;
use yew::{html, use_effect_with_deps, use_mut_ref, use_state, function_component, Html, Properties};

/// Shared drag state persisted across renders.
#[derive(Default)]
struct DragState {
    dragging: bool,
    offset_x: i32,
    offset_y: i32,
}

#[derive(PartialEq, Properties)]
pub struct HallOfFameProps {
    pub score: u32,
    pub kill_log: Vec<(EntityType, u32)>,
    /// Whether to show as a compact in-game panel (true) or full death-screen overlay (false).
    #[prop_or(false)]
    pub compact: bool,
}

#[function_component(HallOfFame)]
pub fn hall_of_fame(props: &HallOfFameProps) -> Html {
    let mut kills = props.kill_log.clone();
    kills.sort_by(|a, b| b.1.cmp(&a.1));
    let total_kills: u32 = kills.iter().map(|(_, c)| c).sum();

    let compact = props.compact;

    // Drag position state (compact mode only).
    let pos_x = use_state(|| 16i32);
    let pos_y = use_state(|| 200i32); // default: 200px from top

    // Persistent drag state across renders.
    let drag_ref = use_mut_ref(DragState::default);

    // onmousedown on title bar starts drag.
    let onmousedown = {
        let drag_ref = drag_ref.clone();
        let px = *pos_x;
        let py = *pos_y;
        yew::Callback::from(move |e: MouseEvent| {
            if !compact {
                return;
            }
            e.prevent_default();
            let mut ds = drag_ref.borrow_mut();
            ds.dragging = true;
            ds.offset_x = e.client_x() - px;
            ds.offset_y = e.client_y() - py;
        })
    };

    // Document-level mousemove/mouseup listeners.
    {
        let pos_x = pos_x.clone();
        let pos_y = pos_y.clone();
        let drag_ref = drag_ref.clone();

        use_effect_with_deps(
            move |_| {
                let mut move_closure: Option<Closure<dyn Fn(MouseEvent)>> = None;
                let mut up_closure: Option<Closure<dyn Fn(MouseEvent)>> = None;
                let mut doc_ref: Option<web_sys::Document> = None;

                if compact {
                    let window = web_sys::window().unwrap();
                    let document = window.document().unwrap();

                    let dr_move = drag_ref.clone();
                    let px = pos_x.clone();
                    let py = pos_y.clone();
                    let on_mousemove = Closure::<dyn Fn(MouseEvent)>::new(move |e: MouseEvent| {
                        let ds = dr_move.borrow();
                        if ds.dragging {
                            px.set(e.client_x() - ds.offset_x);
                            py.set(e.client_y() - ds.offset_y);
                        }
                    });

                    let dr_up = drag_ref.clone();
                    let on_mouseup = Closure::<dyn Fn(MouseEvent)>::new(move |_: MouseEvent| {
                        dr_up.borrow_mut().dragging = false;
                    });

                    document
                        .add_event_listener_with_callback(
                            "mousemove",
                            on_mousemove.as_ref().unchecked_ref(),
                        )
                        .unwrap();
                    document
                        .add_event_listener_with_callback(
                            "mouseup",
                            on_mouseup.as_ref().unchecked_ref(),
                        )
                        .unwrap();

                    doc_ref = Some(document);
                    move_closure = Some(on_mousemove);
                    up_closure = Some(on_mouseup);
                }

                move || {
                    if let (Some(doc), Some(mc), Some(uc)) = (doc_ref, move_closure, up_closure) {
                        let _ = doc.remove_event_listener_with_callback(
                            "mousemove",
                            mc.as_ref().unchecked_ref(),
                        );
                        let _ = doc.remove_event_listener_with_callback(
                            "mouseup",
                            uc.as_ref().unchecked_ref(),
                        );
                    }
                }
            },
            compact,
        );
    }

    // --- Styles ---
    let divider_style = "border: none; border-top: 1px solid rgba(255,255,255,0.15); margin: 6px 0;";

    let (outer_style, container_style, title_style, score_style) = if compact {
        (
            format!(
                "position: fixed; left: {}px; top: {}px; z-index: 5; pointer-events: auto;",
                *pos_x, *pos_y
            ),
            "background: rgba(0,0,0,0.65); border-radius: 8px; padding: 8px 12px; \
             color: white; font-family: monospace, sans-serif; font-size: 12px; \
             min-width: 160px; max-width: 200px; backdrop-filter: blur(4px); \
             border: 1px solid rgba(255,255,255,0.1);"
                .to_string(),
            "font-weight: bold; margin-bottom: 4px; text-align: center; font-size: 13px; \
             letter-spacing: 1px; cursor: grab; user-select: none;"
                .to_string(),
            "text-align: center; font-size: 14px; color: #f1c40f; font-weight: bold; margin-bottom: 4px;"
                .to_string(),
        )
    } else {
        (
            String::new(),
            "background: rgba(0,0,0,0.75); border-radius: 12px; padding: 16px 24px; \
             color: white; font-family: monospace, sans-serif; font-size: 14px; \
             min-width: 240px; max-width: 320px; backdrop-filter: blur(8px); \
             border: 1px solid rgba(241,196,15,0.3); box-shadow: 0 4px 24px rgba(0,0,0,0.5); \
             margin: 0 auto 16px auto;"
                .to_string(),
            "font-weight: bold; margin-bottom: 8px; text-align: center; font-size: 18px; letter-spacing: 2px;"
                .to_string(),
            "text-align: center; font-size: 20px; color: #f1c40f; font-weight: bold; margin-bottom: 8px;"
                .to_string(),
        )
    };

    let inner = html! {
        <div style={container_style}>
            <div style={title_style} onmousedown={onmousedown}>
                {"🏆 名人堂"}
            </div>
            <div style={score_style}>
                {format!("分数: {}", props.score)}
            </div>

            if !kills.is_empty() {
                <hr style={divider_style} />
                <div style="font-weight: bold; text-align: center; font-size: 11px; margin-bottom: 4px; opacity: 0.7;">
                    {"击杀统计"}
                </div>
                { for kills.iter().map(|(entity_type, count)| {
                    let data = entity_type.data();
                    let label = data.label.clone();
                    html! {
                        <div style="display: flex; justify-content: space-between; padding: 2px 0; font-size: 12px;">
                            <span style="overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 70%;">
                                {format!("🚢 {}", label)}
                            </span>
                            <span style="color: #f1c40f; font-weight: bold;">
                                {format!("×{}", count)}
                            </span>
                        </div>
                    }
                })}
                <hr style={divider_style} />
                <div style="text-align: center; font-weight: bold; font-size: 12px;">
                    {format!("总击杀: {}", total_kills)}
                </div>
            } else {
                <div style="text-align: center; opacity: 0.5; font-size: 11px; margin-top: 4px;">
                    {"暂无击杀记录"}
                </div>
            }
        </div>
    };

    if compact {
        html! { <div style={outer_style}>{inner}</div> }
    } else {
        inner
    }
}
