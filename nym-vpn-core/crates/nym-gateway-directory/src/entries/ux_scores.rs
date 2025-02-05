// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(Clone)]
pub struct UxScore {
    pub max_score: u8,
    pub current_score: u8,
    pub color_hex: String,
}

#[derive(Clone)]
pub struct UxScores {
    pub mix_score: UxScore,
    pub wg_score: UxScore,
}
