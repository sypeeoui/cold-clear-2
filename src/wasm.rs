use wasm_bindgen::prelude::*;
use crate::data::{Board, GameState, Piece, Rotation, Spin};
use crate::bot::{Bot, BotConfig, BotOptions};
use std::sync::Arc;
use enumset::EnumSet;
use serde::{Deserialize, Serialize};

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

#[derive(Serialize, Deserialize)]
pub struct WasmMove {
    pub piece: u8,
    pub rotation: u8,
    pub x: i8,
    pub y: i8,
    pub spin: u8,
}

#[derive(Serialize, Deserialize)]
pub struct WasmSearchResponse {
    pub best_move: WasmMove,
    pub score: f32,
    pub hold_used: bool,
    pub pv: Vec<WasmMove>,
}

#[wasm_bindgen]
pub fn find_best_move(
    board_rows: &[u64],
    current_piece: u8,
    queue_slice: &[u8],
    hold_piece: Option<u8>,
    b2b: bool,
    combo: u32,
    nodes: u32,
) -> JsValue {
    let config = BotConfig::default();
    
    let current = match piece_from_external(current_piece) {
        Some(p) => p,
        None => return JsValue::NULL,
    };

    let mut queue: Vec<Piece> = queue_slice.iter().filter_map(|&p| piece_from_external(p)).collect();
    let hold = hold_piece.and_then(piece_from_external);

    let reserve = hold.unwrap_or_else(|| if !queue.is_empty() { queue.remove(0) } else { current });

    let mut cols = [0u64; 10];
    for (i, &r) in board_rows.iter().enumerate().take(40) {
        for x in 0..10 {
            if (r & (1 << x)) != 0 {
                cols[x] |= 1 << i;
            }
        }
    }

    let state = GameState {
        board: Board { cols },
        reserve,
        back_to_back: b2b,
        combo: combo.try_into().unwrap_or(255),
        bag: EnumSet::all(),
    };

    let options = BotOptions {
        speculate: true,
        config: Arc::new(config),
    };

    let mut bot = Bot::new(options, state, &queue);

    for _ in 0..nodes {
        bot.do_work();
    }

    let suggestions = bot.suggest();
    if suggestions.is_empty() {
        return JsValue::NULL;
    }

    let best = suggestions[0];
    let hold_used = best.location.piece != current;

    let response = WasmSearchResponse {
        best_move: WasmMove {
            piece: piece_to_external(best.location.piece),
            rotation: rotation_to_external(best.location.rotation),
            x: best.location.x,
            y: best.location.y,
            spin: spin_to_external(best.spin),
        },
        score: 0.0,
        hold_used,
        pv: Vec::new(),
    };

    serde_wasm_bindgen::to_value(&response).unwrap_or(JsValue::NULL)
}

fn piece_from_external(v: u8) -> Option<Piece> {
    match v {
        0 => Some(Piece::I),
        1 => Some(Piece::O),
        2 => Some(Piece::T),
        3 => Some(Piece::S),
        4 => Some(Piece::Z),
        5 => Some(Piece::J),
        6 => Some(Piece::L),
        _ => None,
    }
}

fn piece_to_external(p: Piece) -> u8 {
    match p {
        Piece::I => 0,
        Piece::O => 1,
        Piece::T => 2,
        Piece::S => 3,
        Piece::Z => 4,
        Piece::J => 5,
        Piece::L => 6,
    }
}

fn rotation_to_external(r: Rotation) -> u8 {
    match r {
        Rotation::North => 0,
        Rotation::East => 1,
        Rotation::South => 2,
        Rotation::West => 3,
    }
}

fn spin_to_external(s: Spin) -> u8 {
    match s {
        Spin::None => 0,
        Spin::Mini => 1,
        Spin::Full => 2,
    }
}
