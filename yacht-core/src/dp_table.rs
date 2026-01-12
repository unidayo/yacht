//! DPテーブルと期待値計算モジュール
//!
//! 前計算されたDPテーブルを使って、最適な意思決定を行う。

/// DPテーブルデータ（f32 little-endian）
/// 構造: dp_table[upper_sum][used_hands]
/// - upper_sum: 0-63 (64通り)
/// - used_hands: 0-4095 (4096通り、12ビットマスク)
static DP_TABLE_DATA: &[u8] = include_bytes!("dp_table.bin");

const UPPER_SUM_MAX: usize = 64;
const USED_HANDS_MAX: usize = 4096;
const UPPER_BONUS_THRESHOLD: usize = 63;
const UPPER_BONUS_POINTS: f32 = 35.0;

/// DPテーブルから期待得点を取得
#[inline]
pub fn get_expected_score(upper_sum: usize, used_hands: usize) -> f32 {
    debug_assert!(upper_sum < UPPER_SUM_MAX);
    debug_assert!(used_hands < USED_HANDS_MAX);

    let idx = upper_sum * USED_HANDS_MAX + used_hands;
    let offset = idx * 4;
    let bytes = &DP_TABLE_DATA[offset..offset + 4];
    f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}

/// 初期期待得点を取得
pub fn get_initial_expected_score() -> f32 {
    get_expected_score(0, 0)
}

/// カテゴリ選択後の価値を計算
///
/// 戻り値: 即時スコア + ボーナス + 将来の期待値
pub fn evaluate_category_choice(
    current_upper_sum: usize,
    current_used_hands: usize,
    category: usize,
    immediate_score: u8,
) -> f32 {
    let new_used_hands = current_used_hands | (1 << category);
    let is_upper = category < 6;

    if is_upper {
        let new_upper_sum = (current_upper_sum + immediate_score as usize).min(UPPER_BONUS_THRESHOLD);
        let bonus = if current_upper_sum < UPPER_BONUS_THRESHOLD && new_upper_sum >= UPPER_BONUS_THRESHOLD {
            UPPER_BONUS_POINTS
        } else {
            0.0
        };
        immediate_score as f32 + bonus + get_expected_score(new_upper_sum, new_used_hands)
    } else {
        immediate_score as f32 + get_expected_score(current_upper_sum, new_used_hands)
    }
}

// =============================================================================
// 出目パターンと確率テーブル
// =============================================================================

/// 出目パターン: 各目の個数 [1の個数, 2の個数, ..., 6の個数]
pub type DicePattern = [u8; 6];

/// 出目パターンと確率のペア
#[derive(Clone, Copy)]
pub struct PatternProbability {
    pub pattern: DicePattern,
    pub probability: f32,
}

/// n個のサイコロを振った時のパターン一覧（事前計算）
///
/// - PATTERNS_1: 1個振る → 6通り
/// - PATTERNS_2: 2個振る → 21通り
/// - PATTERNS_3: 3個振る → 56通り
/// - PATTERNS_4: 4個振る → 126通り
/// - PATTERNS_5: 5個振る → 252通り
pub mod dice_patterns {
    use super::*;

    /// 階乗テーブル（0! ~ 5!）
    const FACTORIAL: [u32; 6] = [1, 1, 2, 6, 24, 120];

    /// 多項係数を計算: n! / (k1! * k2! * ... * k6!)
    fn multinomial(counts: [u8; 6]) -> u32 {
        let n = (counts[0] + counts[1] + counts[2] + counts[3] + counts[4] + counts[5]) as usize;
        if n > 5 {
            return 0;
        }
        let numerator = FACTORIAL[n];
        let denominator = FACTORIAL[counts[0] as usize]
            * FACTORIAL[counts[1] as usize]
            * FACTORIAL[counts[2] as usize]
            * FACTORIAL[counts[3] as usize]
            * FACTORIAL[counts[4] as usize]
            * FACTORIAL[counts[5] as usize];
        numerator / denominator
    }

    /// パターンを生成するマクロ代わりの関数
    fn generate_patterns(num_dice: u8) -> Vec<PatternProbability> {
        let total_outcomes = 6_u32.pow(num_dice as u32);
        let mut patterns = Vec::new();

        // 全パターンを列挙
        fn recurse(
            dice_left: u8,
            face: usize,
            current: &mut DicePattern,
            patterns: &mut Vec<PatternProbability>,
            total_outcomes: u32,
        ) {
            if face == 6 {
                if dice_left == 0 {
                    let count = multinomial(*current);
                    let prob = count as f32 / total_outcomes as f32;
                    patterns.push(PatternProbability {
                        pattern: *current,
                        probability: prob,
                    });
                }
                return;
            }
            for k in 0..=dice_left {
                current[face] = k;
                recurse(dice_left - k, face + 1, current, patterns, total_outcomes);
            }
            current[face] = 0;
        }

        let mut current = [0u8; 6];
        recurse(num_dice, 0, &mut current, &mut patterns, total_outcomes);
        patterns
    }

    lazy_static::lazy_static! {
        pub static ref PATTERNS_1: Vec<PatternProbability> = generate_patterns(1);
        pub static ref PATTERNS_2: Vec<PatternProbability> = generate_patterns(2);
        pub static ref PATTERNS_3: Vec<PatternProbability> = generate_patterns(3);
        pub static ref PATTERNS_4: Vec<PatternProbability> = generate_patterns(4);
        pub static ref PATTERNS_5: Vec<PatternProbability> = generate_patterns(5);
    }

    pub fn get_patterns(num_dice: usize) -> &'static [PatternProbability] {
        match num_dice {
            0 => &[],
            1 => &PATTERNS_1,
            2 => &PATTERNS_2,
            3 => &PATTERNS_3,
            4 => &PATTERNS_4,
            5 => &PATTERNS_5,
            _ => &[],
        }
    }
}

// =============================================================================
// キープパターン列挙
// =============================================================================

/// 現在の出目から可能なキープパターンを列挙
pub fn enumerate_keep_patterns(current_dice: &DicePattern) -> Vec<DicePattern> {
    let mut patterns = Vec::new();

    fn recurse(
        face: usize,
        current: &DicePattern,
        keep: &mut DicePattern,
        patterns: &mut Vec<DicePattern>,
    ) {
        if face == 6 {
            patterns.push(*keep);
            return;
        }
        for k in 0..=current[face] {
            keep[face] = k;
            recurse(face + 1, current, keep, patterns);
        }
        keep[face] = 0;
    }

    let mut keep = [0u8; 6];
    recurse(0, current_dice, &mut keep, &mut patterns);
    patterns
}

/// 出目パターンを合成
pub fn add_patterns(a: &DicePattern, b: &DicePattern) -> DicePattern {
    [
        a[0] + b[0],
        a[1] + b[1],
        a[2] + b[2],
        a[3] + b[3],
        a[4] + b[4],
        a[5] + b[5],
    ]
}

/// 出目パターンの個数合計
pub fn pattern_count(p: &DicePattern) -> u8 {
    p[0] + p[1] + p[2] + p[3] + p[4] + p[5]
}

/// 出目配列を出目パターンに変換
pub fn dice_to_pattern(dice: &[u8]) -> DicePattern {
    let mut pattern = [0u8; 6];
    for &d in dice {
        if d >= 1 && d <= 6 {
            pattern[(d - 1) as usize] += 1;
        }
    }
    pattern
}

/// 出目パターンの合計値（ピップ数）
pub fn pattern_pips(p: &DicePattern) -> u8 {
    p[0] * 1 + p[1] * 2 + p[2] * 3 + p[3] * 4 + p[4] * 5 + p[5] * 6
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_score() {
        let score = get_initial_expected_score();
        assert!((score - 190.158733).abs() < 0.01);
    }

    #[test]
    fn test_patterns_count() {
        // 5個振る → 252パターン
        assert_eq!(dice_patterns::PATTERNS_5.len(), 252);
        // 確率の合計は1.0
        let total: f32 = dice_patterns::PATTERNS_5.iter().map(|p| p.probability).sum();
        assert!((total - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_dice_to_pattern() {
        let dice = [1, 1, 2, 3, 6];
        let pattern = dice_to_pattern(&dice);
        assert_eq!(pattern, [2, 1, 1, 0, 0, 1]);
    }
}
