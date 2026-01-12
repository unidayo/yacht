use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use rand::Rng;

mod dp_table;

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

    /// 使用済みカテゴリのビットマスクを取得
    pub fn used_hands_mask(&self) -> usize {
        let mut mask = 0usize;
        for (i, s) in self.scores.iter().enumerate() {
            if s.is_some() {
                mask |= 1 << i;
            }
        }
        mask
    }

    /// 上段スコアの累計を取得（63上限）
    pub fn upper_sum_capped(&self) -> usize {
        self.get_upper_total().min(63) as usize
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

    /// AI用: 現在のAIの上段累計スコア（63上限）
    pub fn ai_upper_sum_capped(&self) -> usize {
        self.ai_score.upper_sum_capped()
    }

    /// AI用: AIの使用済みカテゴリマスク
    pub fn ai_used_hands_mask(&self) -> usize {
        self.ai_score.used_hands_mask()
    }

    /// プレイヤー用: 上段累計スコア（63上限）
    pub fn player_upper_sum_capped(&self) -> usize {
        self.player_score.upper_sum_capped()
    }

    /// プレイヤー用: 使用済みカテゴリマスク
    pub fn player_used_hands_mask(&self) -> usize {
        self.player_score.used_hands_mask()
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

// ========== AI Engine (DPテーブルベース) ==========

/// 出目パターンから得点を計算
fn calculate_score_from_pattern(pattern: &dp_table::DicePattern, category: Category) -> u8 {
    match category {
        Category::Ones => pattern[0] * 1,
        Category::Twos => pattern[1] * 2,
        Category::Threes => pattern[2] * 3,
        Category::Fours => pattern[3] * 4,
        Category::Fives => pattern[4] * 5,
        Category::Sixes => pattern[5] * 6,
        Category::FullHouse => {
            let has_three = pattern.iter().any(|&c| c == 3);
            let has_two = pattern.iter().any(|&c| c == 2);
            if has_three && has_two {
                dp_table::pattern_pips(pattern)
            } else {
                0
            }
        }
        Category::FourOfAKind => {
            if pattern.iter().any(|&c| c >= 4) {
                dp_table::pattern_pips(pattern)
            } else {
                0
            }
        }
        Category::LittleStraight => {
            // 4連続: 1-4, 2-5, 3-6
            let has_1234 = pattern[0] >= 1 && pattern[1] >= 1 && pattern[2] >= 1 && pattern[3] >= 1;
            let has_2345 = pattern[1] >= 1 && pattern[2] >= 1 && pattern[3] >= 1 && pattern[4] >= 1;
            let has_3456 = pattern[2] >= 1 && pattern[3] >= 1 && pattern[4] >= 1 && pattern[5] >= 1;
            if has_1234 || has_2345 || has_3456 { 15 } else { 0 }
        }
        Category::BigStraight => {
            let has_12345 = pattern[0] >= 1 && pattern[1] >= 1 && pattern[2] >= 1 && pattern[3] >= 1 && pattern[4] >= 1;
            let has_23456 = pattern[1] >= 1 && pattern[2] >= 1 && pattern[3] >= 1 && pattern[4] >= 1 && pattern[5] >= 1;
            if has_12345 || has_23456 { 30 } else { 0 }
        }
        Category::Choice => dp_table::pattern_pips(pattern),
        Category::Yacht => {
            if pattern.iter().any(|&c| c == 5) { 50 } else { 0 }
        }
    }
}

#[wasm_bindgen]
pub struct YachtAI {}

#[wasm_bindgen]
impl YachtAI {
    #[wasm_bindgen(constructor)]
    pub fn new() -> YachtAI {
        YachtAI {}
    }

    /// AIの手番を実行（ロールとカテゴリ選択を含む）
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

    /// AIが選ぶべきホールドパターンを取得（JS用）
    pub fn get_holds_decision(&self, game: &GameState) -> Vec<u8> {
        self.decide_holds(game).iter().map(|&h| if h { 1 } else { 0 }).collect()
    }

    /// AIが選ぶべきカテゴリを取得（JS用）
    pub fn get_category_decision(&self, game: &GameState) -> usize {
        self.decide_category(game)
    }

    /// どのサイコロを保持するか決定（DPテーブルベース）
    fn decide_holds(&self, game: &GameState) -> Vec<bool> {
        let dice = game.get_dice_values();
        let rolls_left = game.get_rolls_left();
        let upper_sum = game.ai_upper_sum_capped();
        let used_hands = game.ai_used_hands_mask();

        let current_pattern = dp_table::dice_to_pattern(&dice);
        let keep_patterns = dp_table::enumerate_keep_patterns(&current_pattern);

        let mut best_holds = vec![false; 5];
        let mut best_expected = f32::NEG_INFINITY;

        let no_locks: [bool; 5] = [false; 5];
        for keep in &keep_patterns {
            let expected = if rolls_left == 1 {
                self.evaluate_final_roll(keep, upper_sum, used_hands)
            } else {
                // rolls_left == 2: 2回振り直し可能
                self.evaluate_two_rolls(keep, upper_sum, used_hands)
            };

            if expected > best_expected {
                best_expected = expected;
                // キープパターンからホールド配列を復元
                best_holds = self.pattern_to_holds(&dice, keep, &no_locks);
            }
        }

        best_holds
    }

    /// 最終振り（1回）の期待値
    fn evaluate_final_roll(
        &self,
        keep: &dp_table::DicePattern,
        upper_sum: usize,
        used_hands: usize,
    ) -> f32 {
        let num_reroll = 5 - dp_table::pattern_count(keep) as usize;
        if num_reroll == 0 {
            return self.best_category_value(keep, upper_sum, used_hands);
        }

        let patterns = dp_table::dice_patterns::get_patterns(num_reroll);
        let mut total = 0.0f32;

        for pp in patterns {
            let final_dice = dp_table::add_patterns(keep, &pp.pattern);
            let value = self.best_category_value(&final_dice, upper_sum, used_hands);
            total += pp.probability * value;
        }

        total
    }

    /// 2回振り直しの期待値
    fn evaluate_two_rolls(
        &self,
        keep: &dp_table::DicePattern,
        upper_sum: usize,
        used_hands: usize,
    ) -> f32 {
        let num_reroll = 5 - dp_table::pattern_count(keep) as usize;
        if num_reroll == 0 {
            return self.best_category_value(keep, upper_sum, used_hands);
        }

        let patterns = dp_table::dice_patterns::get_patterns(num_reroll);
        let mut total = 0.0f32;

        for pp in patterns {
            let after_roll1 = dp_table::add_patterns(keep, &pp.pattern);
            // この出目から最適なキープを選んで、さらに1回振る
            let best_keep_value = self.find_best_keep_for_final(&after_roll1, upper_sum, used_hands);
            total += pp.probability * best_keep_value;
        }

        total
    }

    /// 最終振り前の最適キープ期待値
    fn find_best_keep_for_final(
        &self,
        dice_pattern: &dp_table::DicePattern,
        upper_sum: usize,
        used_hands: usize,
    ) -> f32 {
        let keep_patterns = dp_table::enumerate_keep_patterns(dice_pattern);
        let mut best = f32::NEG_INFINITY;

        for keep in &keep_patterns {
            let value = self.evaluate_final_roll(keep, upper_sum, used_hands);
            if value > best {
                best = value;
            }
        }

        best
    }

    /// 出目パターンに対する最良カテゴリの価値
    fn best_category_value(
        &self,
        dice: &dp_table::DicePattern,
        upper_sum: usize,
        used_hands: usize,
    ) -> f32 {
        let mut best = f32::NEG_INFINITY;

        for cat_idx in 0..12 {
            if (used_hands >> cat_idx) & 1 == 1 {
                continue;
            }
            let category = Category::from_index(cat_idx).unwrap();
            let score = calculate_score_from_pattern(dice, category);
            let value = dp_table::evaluate_category_choice(upper_sum, used_hands, cat_idx, score);
            if value > best {
                best = value;
            }
        }

        best
    }

    /// キープパターンからホールド配列を復元
    /// locks: ロックされたダイス（同じ目が複数ある場合にロック済みを優先キープ）
    fn pattern_to_holds(&self, dice: &[u8], keep: &dp_table::DicePattern, locks: &[bool]) -> Vec<bool> {
        let mut holds = vec![false; 5];
        let mut remaining = *keep;

        // 第1パス: ロックされたダイスを優先的にキープ
        for (i, &d) in dice.iter().enumerate() {
            let face = (d - 1) as usize;
            if locks.get(i).copied().unwrap_or(false) && remaining[face] > 0 {
                holds[i] = true;
                remaining[face] -= 1;
            }
        }

        // 第2パス: 残りのキープ枠をロックされていないダイスに割り当て
        for (i, &d) in dice.iter().enumerate() {
            let face = (d - 1) as usize;
            if !locks.get(i).copied().unwrap_or(false) && remaining[face] > 0 {
                holds[i] = true;
                remaining[face] -= 1;
            }
        }

        holds
    }

    /// カテゴリを選択（DPテーブルベース）
    fn decide_category(&self, game: &GameState) -> usize {
        let dice = game.get_dice_values();
        let upper_sum = game.ai_upper_sum_capped();
        let used_hands = game.ai_used_hands_mask();
        let pattern = dp_table::dice_to_pattern(&dice);

        let mut best_category = 0;
        let mut best_value = f32::NEG_INFINITY;

        for cat_idx in 0..12 {
            if (used_hands >> cat_idx) & 1 == 1 {
                continue;
            }
            let category = Category::from_index(cat_idx).unwrap();
            let score = calculate_score_from_pattern(&pattern, category);
            let value = dp_table::evaluate_category_choice(upper_sum, used_hands, cat_idx, score);

            if value > best_value {
                best_value = value;
                best_category = cat_idx;
            }
        }

        best_category
    }

    // ========== プレイヤー向け推奨機能 ==========

    /// プレイヤー向け: カテゴリ選択の上位3つを取得
    /// 戻り値: JSON配列 [{"category": index, "score": immediate, "expected": value}, ...]
    /// expected は最終的な合計点数の期待値
    pub fn get_top_category_choices(&self, game: &GameState) -> String {
        let dice = game.get_dice_values();
        let upper_sum = game.player_upper_sum_capped();
        let used_hands = game.player_used_hands_mask();
        let current_total = game.get_player_total() as f32;
        let pattern = dp_table::dice_to_pattern(&dice);

        let mut choices: Vec<(usize, u8, f32)> = Vec::new();

        for cat_idx in 0..12 {
            if (used_hands >> cat_idx) & 1 == 1 {
                continue;
            }
            let category = Category::from_index(cat_idx).unwrap();
            let score = calculate_score_from_pattern(&pattern, category);
            let future_value = dp_table::evaluate_category_choice(upper_sum, used_hands, cat_idx, score);
            // 現在の合計 + 将来の期待値 = 最終的な合計点数の期待値
            let total_expected = current_total + future_value;
            choices.push((cat_idx, score, total_expected));
        }

        // 期待値でソート（降順）
        choices.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());

        // 上位3つを取得
        let top3: Vec<_> = choices.into_iter().take(3).collect();

        // JSON形式で返す
        let json_array: Vec<String> = top3
            .iter()
            .map(|(cat, score, expected)| {
                format!(
                    r#"{{"category":{},"score":{},"expected":{:.1}}}"#,
                    cat, score, expected
                )
            })
            .collect();

        format!("[{}]", json_array.join(","))
    }

    /// プレイヤー向け: キープパターンの上位3つを取得
    /// 戻り値: JSON配列 [{"holds": [0,1,1,0,1], "expected": value}, ...]
    /// expected は最終的な合計点数の期待値
    pub fn get_top_hold_choices(&self, game: &GameState) -> String {
        let dice = game.get_dice_values();
        let locks = game.get_dice_locks();
        let rolls_left = game.get_rolls_left();
        let upper_sum = game.player_upper_sum_capped();
        let used_hands = game.player_used_hands_mask();
        let current_total = game.get_player_total() as f32;

        // ロックされたダイスは必ずキープ
        let locked: Vec<bool> = locks.iter().map(|&l| l == 1).collect();

        let current_pattern = dp_table::dice_to_pattern(&dice);
        let keep_patterns = dp_table::enumerate_keep_patterns(&current_pattern);

        let mut choices: Vec<(Vec<bool>, f32)> = Vec::new();

        for keep in &keep_patterns {
            let holds = self.pattern_to_holds(&dice, keep, &locked);

            // ロックされたダイスを解除するパターンはスキップ
            let violates_lock = (0..5).any(|i| locked[i] && !holds[i]);
            if violates_lock {
                continue;
            }

            let future_expected = if rolls_left == 1 {
                self.evaluate_final_roll(keep, upper_sum, used_hands)
            } else {
                self.evaluate_two_rolls(keep, upper_sum, used_hands)
            };

            // 現在の合計 + 将来の期待値 = 最終的な合計点数の期待値
            let total_expected = current_total + future_expected;
            choices.push((holds, total_expected));
        }

        // 期待値でソート（降順）
        choices.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // 上位3つを取得（重複を除去）
        let mut top3: Vec<(Vec<bool>, f32)> = Vec::new();
        let mut seen: std::collections::HashSet<Vec<bool>> = std::collections::HashSet::new();
        for (holds, expected) in choices {
            if !seen.contains(&holds) {
                seen.insert(holds.clone());
                top3.push((holds, expected));
                if top3.len() >= 3 {
                    break;
                }
            }
        }

        // JSON形式で返す
        let json_array: Vec<String> = top3
            .iter()
            .map(|(holds, expected)| {
                let holds_str: Vec<String> = holds.iter().map(|&h| if h { "1".to_string() } else { "0".to_string() }).collect();
                format!(
                    r#"{{"holds":[{}],"expected":{:.1}}}"#,
                    holds_str.join(","),
                    expected
                )
            })
            .collect();

        format!("[{}]", json_array.join(","))
    }

    /// プレイヤー向け: 現在の状態からの総合期待値を取得
    pub fn get_player_expected_score(&self, game: &GameState) -> f32 {
        let upper_sum = game.player_upper_sum_capped();
        let used_hands = game.player_used_hands_mask();
        dp_table::get_expected_score(upper_sum, used_hands)
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
