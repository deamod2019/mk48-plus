// Faction scoreboard overlay — shows 3-faction war stats when faction_mode is on.

use common::protocol::{FactionId, FactionUpdate};
use glam::Vec2;
use yew::{function_component, html, Html, Properties};

#[derive(Properties, PartialEq)]
pub struct FactionBoardProps {
    pub faction_data: FactionUpdate,
    pub my_faction: Option<FactionId>,
    /// Known altar position for this player's faction (None = not yet discovered).
    #[prop_or_default]
    pub altar_position: Option<Vec2>,
    /// Per-faction sacrifice progress [red, blue, green].
    #[prop_or_default]
    pub altar_sacrifice_counts: [u8; FactionId::COUNT],
}

#[function_component(FactionBoard)]
pub fn faction_board(props: &FactionBoardProps) -> Html {
    let factions = &props.faction_data.factions;
    let labels = ["🔴 红军", "🔵 蓝军", "🟢 绿军"];
    let colors = ["#e74c3c", "#3498db", "#2ecc71"];
    let bg_colors = ["rgba(231,76,60,0.15)", "rgba(52,152,219,0.15)", "rgba(46,204,113,0.15)"];
    let highlight_bg = ["rgba(231,76,60,0.35)", "rgba(52,152,219,0.35)", "rgba(46,204,113,0.35)"];

    // Find faction with highest total score to scale bars.
    let max_score = factions.iter().map(|f| f.total_score).max().unwrap_or(1).max(1);

    // Build altar sacrifice progress strings.
    let altar_progress: Vec<String> = (0..FactionId::COUNT).map(|i| {
        let count = props.altar_sacrifice_counts[i] as usize;
        let filled = "█".repeat(count.min(5));
        let empty = "░".repeat(5_usize.saturating_sub(count));
        format!("{}{} {}/5", filled, empty, count.min(5))
    }).collect();

    let altar_coord_text = if let Some(pos) = props.altar_position {
        format!("({:.0}, {:.0})", pos.x, pos.y)
    } else {
        "未发现".to_string()
    };

    html! {
        <div style="
            background: rgba(0,0,0,0.65);
            border-radius: 8px;
            padding: 8px 12px;
            color: white;
            font-family: monospace, sans-serif;
            font-size: 13px;
            min-width: 180px;
            backdrop-filter: blur(4px);
            border: 1px solid rgba(255,255,255,0.1);
        ">
            <div style="font-weight: bold; margin-bottom: 6px; text-align: center; font-size: 14px; letter-spacing: 1px;">
                {"⚔️ 阵营战争"}
            </div>
            { for (0..FactionId::COUNT).map(|i| {
                let is_mine = props.my_faction.map_or(false, |f| f.index() == i);
                let bar_width = if max_score > 0 {
                    (factions[i].total_score as f64 / max_score as f64 * 100.0) as u32
                } else { 0 };
                let bg = if is_mine { highlight_bg[i] } else { bg_colors[i] };
                let border = if is_mine {
                    format!("border: 1px solid {};", colors[i])
                } else {
                    "border: 1px solid transparent;".to_string()
                };
                let top_info = factions[i].top_player.as_ref().map(|name| {
                    format!("👑 {} ({})", name, factions[i].top_score)
                }).unwrap_or_default();

                html! {
                    <div style={format!(
                        "margin: 3px 0; padding: 4px 6px; border-radius: 4px; background: {}; {}",
                        bg, border
                    )}>
                        <div style="display: flex; justify-content: space-between; align-items: center;">
                            <span style={format!("color: {}; font-weight: bold;", colors[i])}>
                                { labels[i] }
                                if is_mine {
                                    <span style="font-size: 10px; margin-left: 4px;">{"(你)"}</span>
                                }
                            </span>
                            <span style="font-size: 12px; opacity: 0.8;">
                                { format!("{}人", factions[i].player_count) }
                            </span>
                        </div>
                        <div style="
                            margin-top: 3px;
                            background: rgba(255,255,255,0.1);
                            border-radius: 2px;
                            height: 14px;
                            position: relative;
                            overflow: hidden;
                        ">
                            <div style={format!(
                                "width: {}%; height: 100%; background: {}; border-radius: 2px; transition: width 0.5s ease;",
                                bar_width, colors[i]
                            )}></div>
                            <span style="
                                position: absolute;
                                top: 0; left: 4px;
                                line-height: 14px;
                                font-size: 11px;
                                font-weight: bold;
                                text-shadow: 0 0 3px black;
                            ">
                                { format!("{}", factions[i].total_score) }
                            </span>
                        </div>
                        if !top_info.is_empty() {
                            <div style="font-size: 10px; opacity: 0.6; margin-top: 2px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;">
                                { top_info }
                            </div>
                        }
                    </div>
                }
            })}

            // ---- Altar info section ----
            <div style="
                margin-top: 8px;
                padding-top: 6px;
                border-top: 1px solid rgba(255,255,255,0.15);
            ">
                <div style="font-weight: bold; text-align: center; font-size: 12px; margin-bottom: 4px;">
                    {"🏛️ 水滴祭坛"}
                </div>
                <div style="font-size: 11px; text-align: center; opacity: 0.8; margin-bottom: 4px;">
                    { format!("📍 {}", altar_coord_text) }
                </div>
                { for (0..FactionId::COUNT).map(|i| {
                    let is_mine = props.my_faction.map_or(false, |f| f.index() == i);
                    html! {
                        <div style={format!(
                            "display: flex; justify-content: space-between; align-items: center; font-size: 11px; padding: 1px 2px; {}",
                            if is_mine { "font-weight: bold;" } else { "opacity: 0.75;" }
                        )}>
                            <span style={format!("color: {};", colors[i])}>
                                { labels[i] }
                            </span>
                            <span style="letter-spacing: 1px;">
                                { &altar_progress[i] }
                            </span>
                        </div>
                    }
                })}
            </div>
        </div>
    }
}
