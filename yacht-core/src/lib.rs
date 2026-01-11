use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use rand::Rng;

// ヨットの役（カテゴリ）
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Category {
    Ones = 0,
    Twos = 1,
    Threes = 2,
    Fours = 3,
    Fives = 4,
    Sixes = 5,
    FullHouse = 6,
    FourOfAKind = 7,
    LittleStraight = 8,
    BigStraight = 9,
    Choice = 10,
    Yacht = 11,
}

impl Category {
    pub fn all() -> [Category; 12] {
        [
            Category::Ones,
            Category::Twos,
            Category::Threes,
            Category::Fours,
            Category::Fives,
            Category::Sixes,
            Category::FullHouse,
            Category::FourOfAKind,
            Category::LittleStraight,
            Category::BigStraight,
            Category::Choice,
            Category::Yacht,
        ]
    }

    pub fn from_index(index: usize) -> Option<Category> {
        match index {
            0 => Some(Category::Ones),
            1 => Some(Category::Twos),
            2 => Some(Category::Threes),
            3 => Some(Category::Fours),
            4 => Some(Category::Fives),
            5 => Some(Category::Sixes),
            6 => Some(Category::FullHouse),
            7 => Some(Category::FourOfAKind),
            8 => Some(Category::LittleStraight),
            9 => Some(Category::BigStraight),
            10 => Some(Category::Choice),
            11 => Some(Category::Yacht),
            _ => None,
        }
    }
}

// サイコロの状態
#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Dice {
    values: [u8; 5],
    held: [bool; 5],
    locked: [bool; 5],  // ロール時に確定したキープ（解除不可）
}

#[wasm_bindgen]
impl Dice {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Dice {
        Dice {
            values: [1, 1, 1, 1, 1],
            held: [false; 5],
            locked: [false; 5],
        }
    }

    pub fn roll(&mut self) {
        let mut rng = rand::thread_rng();
        // ロール時にheldをlockedに確定
        for i in 0..5 {
            if self.held[i] {
                self.locked[i] = true;
            }
        }
        for i in 0..5 {
            if !self.held[i] {
                self.values[i] = rng.gen_range(1..=6);
            }
        }
    }

    pub fn set_hold(&mut self, index: usize, hold: bool) {
        if index < 5 {
            self.held[index] = hold;
        }
    }

    pub fn toggle_hold(&mut self, index: usize) {
        if index < 5 && !self.locked[index] {
            self.held[index] = !self.held[index];
        }
    }

    pub fn reset_holds(&mut self) {
        self.held = [false; 5];
        self.locked = [false; 5];
    }

    pub fn is_locked(&self, index: usize) -> bool {
        self.locked[index]
    }

    pub fn get_locks(&self) -> Vec<u8> {
        self.locked.iter().map(|&l| if l { 1 } else { 0 }).collect()
    }

    pub fn get_value(&self, index: usize) -> u8 {
        self.values[index]
    }

    pub fn get_values(&self) -> Vec<u8> {
        self.values.to_vec()
    }

    pub fn is_held(&self, index: usize) -> bool {
        self.held[index]
    }

    pub fn get_holds(&self) -> Vec<u8> {
        self.held.iter().map(|&h| if h { 1 } else { 0 }).collect()
    }

    pub fn set_values(&mut self, values: Vec<u8>) {
        for (i, v) in values.iter().enumerate() {
            if i < 5 {
                self.values[i] = *v;
            }
        }
    }
}

impl Default for Dice {
    fn default() -> Self {
        Self::new()
    }
}

// 得点計算
pub fn calculate_score(dice: &[u8; 5], category: Category) -> u8 {
    let mut counts = [0u8; 7]; // index 1-6 for dice values
    for &v in dice {
        counts[v as usize] += 1;
    }
    let sum: u8 = dice.iter().sum();

    match category {
        Category::Ones => counts[1] * 1,
        Category::Twos => counts[2] * 2,
        Category::Threes => counts[3] * 3,
        Category::Fours => counts[4] * 4,
        Category::Fives => counts[5] * 5,
        Category::Sixes => counts[6] * 6,
        Category::FullHouse => {
            let has_three = counts[1..=6].iter().any(|&c| c == 3);
            let has_two = counts[1..=6].iter().any(|&c| c == 2);
            if has_three && has_two {
                sum
            } else {
                0
            }
        }
        Category::FourOfAKind => {
            // 4つ以上同じ目があれば、全ての出目の合計
            if counts[1..=6].iter().any(|&c| c >= 4) {
                sum
            } else {
                0
            }
        }
        Category::LittleStraight => {
            // スモールストレート: 4つ連続で15点
            let has_1234 = counts[1] >= 1 && counts[2] >= 1 && counts[3] >= 1 && counts[4] >= 1;
            let has_2345 = counts[2] >= 1 && counts[3] >= 1 && counts[4] >= 1 && counts[5] >= 1;
            let has_3456 = counts[3] >= 1 && counts[4] >= 1 && counts[5] >= 1 && counts[6] >= 1;
            if has_1234 || has_2345 || has_3456 {
                15
            } else {
                0
            }
        }
        Category::BigStraight => {
            // ビッグストレート: 5つ連続で30点
            let has_12345 = counts[1] >= 1 && counts[2] >= 1 && counts[3] >= 1 && counts[4] >= 1 && counts[5] >= 1;
            let has_23456 = counts[2] >= 1 && counts[3] >= 1 && counts[4] >= 1 && counts[5] >= 1 && counts[6] >= 1;
            if has_12345 || has_23456 {
                30
            } else {
                0
            }
        }
        Category::Choice => sum,
        Category::Yacht => {
            if counts[1..=6].iter().any(|&c| c == 5) {
                50
            } else {
                0
            }
        }
    }
}

// プレイヤーのスコアボード
#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScoreBoard {
    scores: [Option<u8>; 12],
}

#[wasm_bindgen]
impl ScoreBoard {
    #[wasm_bindgen(constructor)]
    pub fn new() -> ScoreBoard {
        ScoreBoard {
            scores: [None; 12],
        }
    }

    pub fn set_score(&mut self, category: Category, score: u8) -> bool {
        let idx = category as usize;
        if self.scores[idx].is_none() {
            self.scores[idx] = Some(score);
            true
        } else {
            false
        }
    }

    pub fn get_score(&self, category: Category) -> i16 {
        self.scores[category as usize].map(|s| s as i16).unwrap_or(-1)
    }

    pub fn is_used(&self, category: Category) -> bool {
        self.scores[category as usize].is_some()
    }

    pub fn get_upper_total(&self) -> u16 {
        let mut total: u16 = 0;
        for cat in [Category::Ones, Category::Twos, Category::Threes,
                    Category::Fours, Category::Fives, Category::Sixes] {
            if let Some(s) = self.scores[cat as usize] {
                total += s as u16;
            }
        }
        total
    }

    pub fn get_upper_bonus(&self) -> u16 {
        if self.get_upper_total() >= 63 { 35 } else { 0 }
    }

    pub fn get_lower_total(&self) -> u16 {
        let mut total: u16 = 0;
        for cat in [Category::FullHouse, Category::FourOfAKind, Category::LittleStraight,
                    Category::BigStraight, Category::Choice, Category::Yacht] {
            if let Some(s) = self.scores[cat as usize] {
                total += s as u16;
            }
        }
        total
    }

    pub fn get_total(&self) -> u16 {
        self.get_upper_total() + self.get_upper_bonus() + self.get_lower_total()
    }

    pub fn available_categories(&self) -> Vec<u8> {
        self.scores
            .iter()
            .enumerate()
            .filter(|(_, s)| s.is_none())
            .map(|(i, _)| i as u8)
            .collect()
    }

    pub fn is_complete(&self) -> bool {
        self.scores.iter().all(|s| s.is_some())
    }
}

impl Default for ScoreBoard {
    fn default() -> Self {
        Self::new()
    }
}

// ゲーム状態
#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameState {
    dice: Dice,
    player_score: ScoreBoard,
    ai_score: ScoreBoard,
    current_player: u8, // 0 = player, 1 = AI
    rolls_left: u8,
    game_over: bool,
}

#[wasm_bindgen]
impl GameState {
    #[wasm_bindgen(constructor)]
    pub fn new() -> GameState {
        GameState {
            dice: Dice::new(),
            player_score: ScoreBoard::new(),
            ai_score: ScoreBoard::new(),
            current_player: 0,
            rolls_left: 3,
            game_over: false,
        }
    }

    pub fn roll_dice(&mut self) -> bool {
        if self.rolls_left > 0 && !self.game_over {
            self.dice.roll();
            self.rolls_left -= 1;
            true
        } else {
            false
        }
    }

    pub fn toggle_hold(&mut self, index: usize) {
        if self.rolls_left < 3 && self.rolls_left > 0 {
            self.dice.toggle_hold(index);
        }
    }

    pub fn get_dice_locks(&self) -> Vec<u8> {
        self.dice.get_locks()
    }

    pub fn reset_holds(&mut self) {
        self.dice.reset_holds();
    }

    pub fn select_category(&mut self, category_index: usize) -> bool {
        if self.game_over || self.rolls_left == 3 {
            return false;
        }

        let category = match Category::from_index(category_index) {
            Some(c) => c,
            None => return false,
        };

        let dice_values: [u8; 5] = self.dice.values;
        let score = calculate_score(&dice_values, category);

        let success = if self.current_player == 0 {
            self.player_score.set_score(category, score)
        } else {
            self.ai_score.set_score(category, score)
        };

        if success {
            self.end_turn();
        }

        success
    }

    fn end_turn(&mut self) {
        self.dice.reset_holds();
        self.rolls_left = 3;

        if self.player_score.is_complete() && self.ai_score.is_complete() {
            self.game_over = true;
        } else {
            self.current_player = 1 - self.current_player;
        }
    }

    pub fn get_dice_values(&self) -> Vec<u8> {
        self.dice.get_values()
    }

    pub fn get_dice_holds(&self) -> Vec<u8> {
        self.dice.get_holds()
    }

    pub fn get_rolls_left(&self) -> u8 {
        self.rolls_left
    }

    pub fn get_current_player(&self) -> u8 {
        self.current_player
    }

    pub fn is_game_over(&self) -> bool {
        self.game_over
    }

    pub fn get_player_score(&self, category_index: usize) -> i16 {
        Category::from_index(category_index)
            .map(|c| self.player_score.get_score(c))
            .unwrap_or(-1)
    }

    pub fn get_ai_score(&self, category_index: usize) -> i16 {
        Category::from_index(category_index)
            .map(|c| self.ai_score.get_score(c))
            .unwrap_or(-1)
    }

    pub fn get_player_total(&self) -> u16 {
        self.player_score.get_total()
    }

    pub fn get_ai_total(&self) -> u16 {
        self.ai_score.get_total()
    }

    pub fn get_player_upper_total(&self) -> u16 {
        self.player_score.get_upper_total()
    }

    pub fn get_ai_upper_total(&self) -> u16 {
        self.ai_score.get_upper_total()
    }

    pub fn get_player_upper_bonus(&self) -> u16 {
        self.player_score.get_upper_bonus()
    }

    pub fn get_ai_upper_bonus(&self) -> u16 {
        self.ai_score.get_upper_bonus()
    }

    pub fn get_available_categories(&self) -> Vec<u8> {
        if self.current_player == 0 {
            self.player_score.available_categories()
        } else {
            self.ai_score.available_categories()
        }
    }

    pub fn get_potential_score(&self, category_index: usize) -> u8 {
        Category::from_index(category_index)
            .map(|c| calculate_score(&self.dice.values, c))
            .unwrap_or(0)
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

// ========== AI Engine ==========

#[wasm_bindgen]
pub struct YachtAI {}

#[wasm_bindgen]
impl YachtAI {
    #[wasm_bindgen(constructor)]
    pub fn new() -> YachtAI {
        YachtAI {}
    }

    // AIの手番を実行（ロールとカテゴリ選択を含む）
    pub fn play_turn(&self, game: &mut GameState) -> String {
        let mut actions = Vec::new();

        // 最初のロール
        game.roll_dice();
        actions.push(format!("Roll: {:?}", game.get_dice_values()));

        // 2回目のロール判断
        if game.get_rolls_left() > 0 {
            let holds = self.decide_holds(game);
            for (i, hold) in holds.iter().enumerate() {
                if *hold && !game.dice.is_held(i) {
                    game.toggle_hold(i);
                }
            }
            game.roll_dice();
            actions.push(format!("Hold: {:?}, Roll: {:?}", holds, game.get_dice_values()));
        }

        // 3回目のロール判断
        if game.get_rolls_left() > 0 {
            let holds = self.decide_holds(game);
            for (i, hold) in holds.iter().enumerate() {
                if *hold {
                    game.toggle_hold(i);
                }
            }
            game.roll_dice();
            actions.push(format!("Hold: {:?}, Roll: {:?}", holds, game.get_dice_values()));
        }

        // カテゴリ選択
        let category = self.decide_category(game);
        game.select_category(category);
        actions.push(format!("Selected category: {}", category));

        actions.join("\n")
    }

    // AIが選ぶべきホールドパターンを取得（JS用）
    pub fn get_holds_decision(&self, game: &GameState) -> Vec<u8> {
        self.decide_holds(game).iter().map(|&h| if h { 1 } else { 0 }).collect()
    }

    // AIが選ぶべきカテゴリを取得（JS用）
    pub fn get_category_decision(&self, game: &GameState) -> usize {
        self.decide_category(game)
    }

    // どのサイコロを保持するか決定
    fn decide_holds(&self, game: &GameState) -> Vec<bool> {
        let dice = game.get_dice_values();
        let available = game.get_available_categories();

        let mut best_holds = vec![false; 5];
        let mut best_expected = f64::NEG_INFINITY;

        // 全ての保持パターン（2^5 = 32通り）を評価
        for hold_mask in 0u8..32 {
            let holds: Vec<bool> = (0..5).map(|i| (hold_mask >> i) & 1 == 1).collect();
            let expected = self.evaluate_hold_pattern(&dice, &holds, &available);

            if expected > best_expected {
                best_expected = expected;
                best_holds = holds;
            }
        }

        best_holds
    }

    // 保持パターンの期待値を計算
    fn evaluate_hold_pattern(&self, dice: &[u8], holds: &[bool], available: &[u8]) -> f64 {
        let held_dice: Vec<u8> = dice.iter()
            .enumerate()
            .filter(|(i, _)| holds[*i])
            .map(|(_, &v)| v)
            .collect();

        let num_reroll = 5 - held_dice.len();

        if num_reroll == 0 {
            // 全部保持なら、最良のカテゴリスコアを返す
            let dice_arr: [u8; 5] = [dice[0], dice[1], dice[2], dice[3], dice[4]];
            return self.best_category_score(&dice_arr, available) as f64;
        }

        // リロール結果の期待値を計算
        let mut total_expected = 0.0;
        let total_outcomes = 6_u32.pow(num_reroll as u32);

        self.enumerate_reroll_outcomes(&held_dice, num_reroll, available, &mut total_expected);

        total_expected / total_outcomes as f64
    }

    // リロール結果を列挙して期待値を計算
    fn enumerate_reroll_outcomes(&self, held: &[u8], num_reroll: usize, available: &[u8], total: &mut f64) {
        let mut reroll = vec![1u8; num_reroll];

        loop {
            // 現在の組み合わせでの最良スコア
            let mut all_dice: Vec<u8> = held.to_vec();
            all_dice.extend(&reroll);
            all_dice.sort();

            let dice_arr: [u8; 5] = [all_dice[0], all_dice[1], all_dice[2], all_dice[3], all_dice[4]];
            *total += self.best_category_score(&dice_arr, available) as f64;

            // 次の組み合わせ
            let mut carry = true;
            for i in 0..num_reroll {
                if carry {
                    reroll[i] += 1;
                    if reroll[i] > 6 {
                        reroll[i] = 1;
                    } else {
                        carry = false;
                    }
                }
            }
            if carry {
                break;
            }
        }
    }

    // 最良のカテゴリとそのスコアを返す
    fn best_category_score(&self, dice: &[u8; 5], available: &[u8]) -> u8 {
        let mut best_score = 0u8;

        for &cat_idx in available {
            if let Some(category) = Category::from_index(cat_idx as usize) {
                let score = calculate_score(dice, category);

                // カテゴリの価値を調整（上段ボーナスを考慮）
                let adjusted_score = self.adjust_score_for_strategy(dice, category, score);

                if adjusted_score > best_score {
                    best_score = adjusted_score;
                }
            }
        }

        best_score
    }

    // 戦略的なスコア調整
    fn adjust_score_for_strategy(&self, _dice: &[u8; 5], category: Category, score: u8) -> u8 {
        match category {
            // 上段カテゴリ：目標値（n×3）に対する達成度で加点
            Category::Ones => {
                if score >= 3 { score + 2 } else { score }
            }
            Category::Twos => {
                if score >= 6 { score + 2 } else { score }
            }
            Category::Threes => {
                if score >= 9 { score + 2 } else { score }
            }
            Category::Fours => {
                if score >= 12 { score + 2 } else { score }
            }
            Category::Fives => {
                if score >= 15 { score + 2 } else { score }
            }
            Category::Sixes => {
                if score >= 18 { score + 2 } else { score }
            }
            // Yachtは高価値
            Category::Yacht => {
                if score == 50 { 55 } else { 0 }
            }
            // ストレートも高価値
            Category::LittleStraight => {
                if score == 15 { 17 } else { 0 }
            }
            Category::BigStraight => {
                if score == 30 { 33 } else { 0 }
            }
            _ => score,
        }
    }

    // カテゴリを選択
    fn decide_category(&self, game: &GameState) -> usize {
        let dice = game.get_dice_values();
        let dice_arr: [u8; 5] = [dice[0], dice[1], dice[2], dice[3], dice[4]];
        let available = game.get_available_categories();

        let mut best_category = available[0] as usize;
        let mut best_value = f64::NEG_INFINITY;

        for &cat_idx in &available {
            if let Some(category) = Category::from_index(cat_idx as usize) {
                let score = calculate_score(&dice_arr, category);
                let value = self.evaluate_category_choice(category, score, &available);

                if value > best_value {
                    best_value = value;
                    best_category = cat_idx as usize;
                }
            }
        }

        best_category
    }

    // カテゴリ選択の価値を評価
    fn evaluate_category_choice(&self, category: Category, score: u8, available: &[u8]) -> f64 {
        let base_score = score as f64;

        // 上段カテゴリの場合、ボーナス達成への影響を考慮
        let bonus_factor = match category {
            Category::Ones => {
                let target = 3.0;
                let achieved = score as f64;
                (achieved - target) * 0.5 // 目標との差分で評価
            }
            Category::Twos => {
                let target = 6.0;
                let achieved = score as f64;
                (achieved - target) * 0.5
            }
            Category::Threes => {
                let target = 9.0;
                let achieved = score as f64;
                (achieved - target) * 0.5
            }
            Category::Fours => {
                let target = 12.0;
                let achieved = score as f64;
                (achieved - target) * 0.5
            }
            Category::Fives => {
                let target = 15.0;
                let achieved = score as f64;
                (achieved - target) * 0.5
            }
            Category::Sixes => {
                let target = 18.0;
                let achieved = score as f64;
                (achieved - target) * 0.5
            }
            _ => 0.0,
        };

        // 0点で使う「捨てカテゴリ」の評価
        let waste_penalty = if score == 0 {
            match category {
                Category::Yacht => -25.0, // Yachtは捨てるべきでない
                Category::LittleStraight | Category::BigStraight => -15.0,
                Category::FullHouse | Category::FourOfAKind => -10.0,
                _ => -5.0,
            }
        } else {
            0.0
        };

        // 残りカテゴリ数による調整
        let scarcity_bonus = if available.len() <= 3 {
            base_score * 0.2 // 終盤は実スコア重視
        } else {
            0.0
        };

        base_score + bonus_factor + waste_penalty + scarcity_bonus
    }
}

impl Default for YachtAI {
    fn default() -> Self {
        Self::new()
    }
}

// ========== Utility Functions ==========

#[wasm_bindgen]
pub fn calculate_score_js(dice: Vec<u8>, category_index: usize) -> u8 {
    if dice.len() != 5 {
        return 0;
    }
    let dice_arr: [u8; 5] = [dice[0], dice[1], dice[2], dice[3], dice[4]];
    Category::from_index(category_index)
        .map(|c| calculate_score(&dice_arr, c))
        .unwrap_or(0)
}

#[wasm_bindgen]
pub fn get_category_name(category_index: usize) -> String {
    match category_index {
        0 => "Ones".to_string(),
        1 => "Twos".to_string(),
        2 => "Threes".to_string(),
        3 => "Fours".to_string(),
        4 => "Fives".to_string(),
        5 => "Sixes".to_string(),
        6 => "Full House".to_string(),
        7 => "Four of a Kind".to_string(),
        8 => "Little Straight".to_string(),
        9 => "Big Straight".to_string(),
        10 => "Choice".to_string(),
        11 => "Yacht".to_string(),
        _ => "Unknown".to_string(),
    }
}

#[wasm_bindgen]
pub fn get_category_name_ja(category_index: usize) -> String {
    match category_index {
        0 => "1の目".to_string(),
        1 => "2の目".to_string(),
        2 => "3の目".to_string(),
        3 => "4の目".to_string(),
        4 => "5の目".to_string(),
        5 => "6の目".to_string(),
        6 => "フルハウス".to_string(),
        7 => "フォーオブアカインド".to_string(),
        8 => "スモールストレート".to_string(),
        9 => "ビッグストレート".to_string(),
        10 => "チョイス".to_string(),
        11 => "ヨット".to_string(),
        _ => "不明".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yacht_score() {
        let dice = [6, 6, 6, 6, 6];
        assert_eq!(calculate_score(&dice, Category::Yacht), 50);
    }

    #[test]
    fn test_full_house() {
        let dice = [2, 2, 3, 3, 3];
        assert_eq!(calculate_score(&dice, Category::FullHouse), 13);
    }

    #[test]
    fn test_little_straight() {
        let dice = [1, 2, 3, 4, 5];
        assert_eq!(calculate_score(&dice, Category::LittleStraight), 15);
    }

    #[test]
    fn test_big_straight() {
        let dice = [2, 3, 4, 5, 6];
        assert_eq!(calculate_score(&dice, Category::BigStraight), 30);
    }

    #[test]
    fn test_four_of_kind() {
        let dice = [4, 4, 4, 4, 2];
        assert_eq!(calculate_score(&dice, Category::FourOfAKind), 18);
    }

    #[test]
    fn test_ones() {
        let dice = [1, 1, 2, 3, 4];
        assert_eq!(calculate_score(&dice, Category::Ones), 2);
    }
}
