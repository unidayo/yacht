/**
 * ヨット（Yacht）期待値計算プログラム - 高速化版
 * 
 * 最適化:
 *   出目パターンの確率を事前計算し、ユニークなパターン（252通り）のみを列挙。
 *   これにより 6^5 = 7776 回のループが 252 回に削減される。
 * 
 * 出力:
 *   yacht_dp_table.hpp - dp[upper_sum][used_hands] の二重vector
 */

#include <bits/stdc++.h>

// ankerl::unordered_dense が利用可能な場合はそちらを使用
#ifdef USE_ANKERL
#include <ankerl/unordered_dense.h>
template<typename K, typename V> using HashMap = ankerl::unordered_dense::map<K, V>;
template<typename K> using HashSet = ankerl::unordered_dense::set<K>;
#else
template<typename K, typename V> using HashMap = std::unordered_map<K, V>;
template<typename K> using HashSet = std::unordered_set<K>;
#endif

using namespace std;

// =============================================================================
// 型定義
// =============================================================================

using Dice = array<int, 6>;

// =============================================================================
// 定数
// =============================================================================

namespace Constants {
    constexpr int NUM_FACES = 6;
    constexpr int NUM_DICE = 5;
    constexpr int NUM_HANDS = 12;
    constexpr int UPPER_BONUS_THRESHOLD = 63;
    constexpr int UPPER_BONUS_POINTS = 35;
    constexpr int UPPER_SUM_MAX = 64;
    constexpr int USED_HANDS_MAX = 1 << NUM_HANDS;
}

namespace Hand {
    constexpr int ONES = 0, TWOS = 1, THREES = 2, FOURS = 3, FIVES = 4, SIXES = 5;
    constexpr int FULL_HOUSE = 6, FOUR_OF_A_KIND = 7;
    constexpr int LITTLE_STRAIGHT = 8, BIG_STRAIGHT = 9;
    constexpr int CHOICE = 10, YACHT = 11;
    
    inline bool is_upper(int hand) { return hand < 6; }
}

// =============================================================================
// Dice ユーティリティ
// =============================================================================

namespace DiceUtil {
    
    inline int count(const Dice& d) {
        return d[0] + d[1] + d[2] + d[3] + d[4] + d[5];
    }
    
    inline int total_pips(const Dice& d) {
        return d[0]*1 + d[1]*2 + d[2]*3 + d[3]*4 + d[4]*5 + d[5]*6;
    }
    
    inline int encode(const Dice& d) {
        return d[0] | (d[1]<<3) | (d[2]<<6) | (d[3]<<9) | (d[4]<<12) | (d[5]<<15);
    }
    
    inline Dice decode(int code) {
        return {code&7, (code>>3)&7, (code>>6)&7, (code>>9)&7, (code>>12)&7, (code>>15)&7};
    }
    
    inline int count_encoded(int code) {
        return (code&7) + ((code>>3)&7) + ((code>>6)&7) + 
               ((code>>9)&7) + ((code>>12)&7) + ((code>>15)&7);
    }
    
    inline Dice add(const Dice& a, const Dice& b) {
        return {a[0]+b[0], a[1]+b[1], a[2]+b[2], a[3]+b[3], a[4]+b[4], a[5]+b[5]};
    }
    
    inline Dice empty() { return {0,0,0,0,0,0}; }
}

// =============================================================================
// 得点計算
// =============================================================================

namespace Scoring {
    
    inline int calculate(int hand, const Dice& d) {
        switch (hand) {
            case Hand::ONES:   return d[0] * 1;
            case Hand::TWOS:   return d[1] * 2;
            case Hand::THREES: return d[2] * 3;
            case Hand::FOURS:  return d[3] * 4;
            case Hand::FIVES:  return d[4] * 5;
            case Hand::SIXES:  return d[5] * 6;
            case Hand::FULL_HOUSE: {
                bool h3 = false, h2 = false;
                for (int i = 0; i < 6; i++) {
                    if (d[i] == 3) h3 = true;
                    if (d[i] == 2) h2 = true;
                }
                return (h3 && h2) ? DiceUtil::total_pips(d) : 0;
            }
            case Hand::FOUR_OF_A_KIND: {
                for (int i = 0; i < 6; i++) if (d[i] >= 4) return DiceUtil::total_pips(d);
                return 0;
            }
            case Hand::LITTLE_STRAIGHT: {
                for (int i = 0; i <= 2; i++)
                    if (d[i]>=1 && d[i+1]>=1 && d[i+2]>=1 && d[i+3]>=1) return 15;
                return 0;
            }
            case Hand::BIG_STRAIGHT: {
                for (int i = 0; i <= 1; i++)
                    if (d[i]>=1 && d[i+1]>=1 && d[i+2]>=1 && d[i+3]>=1 && d[i+4]>=1) return 30;
                return 0;
            }
            case Hand::CHOICE: return DiceUtil::total_pips(d);
            case Hand::YACHT: {
                for (int i = 0; i < 6; i++) if (d[i] == 5) return 50;
                return 0;
            }
        }
        return 0;
    }
}

// =============================================================================
// 出目パターンと確率の事前計算
// =============================================================================

/// 出目パターンとその出現確率
struct DicePattern {
    Dice dice;
    int code;
    double probability;  // このパターンが出る確率
};

/// n個のサイコロを振ったときの全ユニークパターンと確率
class DicePatternTable {
public:
    /// patterns[n] = n個のサイコロを振ったときのパターン一覧
    vector<vector<DicePattern>> patterns;
    
    DicePatternTable() {
        patterns.resize(Constants::NUM_DICE + 1);
        
        for (int n = 0; n <= Constants::NUM_DICE; n++) {
            compute_patterns(n);
        }
    }
    
    const vector<DicePattern>& get(int num_dice) const {
        return patterns[num_dice];
    }
    
private:
    void compute_patterns(int num_dice) {
        if (num_dice == 0) {
            // 0個振る場合は空のダイスのみ
            Dice empty = DiceUtil::empty();
            patterns[0].push_back({empty, DiceUtil::encode(empty), 1.0});
            return;
        }
        
        // 全ユニークパターンを列挙し、出現回数をカウント
        HashMap<int, int> count_map;
        int total_outcomes = static_cast<int>(pow(6, num_dice));
        
        for (int outcome = 0; outcome < total_outcomes; outcome++) {
            Dice d = DiceUtil::empty();
            int temp = outcome;
            for (int i = 0; i < num_dice; i++) {
                d[temp % 6]++;
                temp /= 6;
            }
            count_map[DiceUtil::encode(d)]++;
        }
        
        // パターンと確率を格納
        patterns[num_dice].reserve(count_map.size());
        for (auto& [code, count] : count_map) {
            Dice d = DiceUtil::decode(code);
            double prob = static_cast<double>(count) / total_outcomes;
            patterns[num_dice].push_back({d, code, prob});
        }
    }
};

// =============================================================================
// キープパターンの事前計算
// =============================================================================

/// キープパターン（どの出目を何個キープするか）
struct KeepPattern {
    Dice keep;
    int code;
};

class KeepPatternTable {
public:
    /// keep_patterns[roll_code] = その振り出しに対する全キープパターン
    HashMap<int, vector<KeepPattern>> patterns;
    
    void compute(const DicePatternTable& dice_table) {
        HashSet<int> seen;
        
        for (int n = 1; n <= Constants::NUM_DICE; n++) {
            for (const auto& pattern : dice_table.get(n)) {
                if (seen.contains(pattern.code)) continue;
                seen.insert(pattern.code);
                
                const Dice& roll = pattern.dice;
                vector<KeepPattern>& keep_list = patterns[pattern.code];
                
                for (int k0 = 0; k0 <= roll[0]; k0++) {
                for (int k1 = 0; k1 <= roll[1]; k1++) {
                for (int k2 = 0; k2 <= roll[2]; k2++) {
                for (int k3 = 0; k3 <= roll[3]; k3++) {
                for (int k4 = 0; k4 <= roll[4]; k4++) {
                for (int k5 = 0; k5 <= roll[5]; k5++) {
                    Dice keep = {k0, k1, k2, k3, k4, k5};
                    keep_list.push_back({keep, DiceUtil::encode(keep)});
                }}}}}}
            }
        }
    }
    
    const vector<KeepPattern>& get(int roll_code) const {
        return patterns.at(roll_code);
    }
};

// =============================================================================
// DP計算エンジン（高速化版）
// =============================================================================

class YachtDPSolver {
private:
    const DicePatternTable& dice_table_;
    const KeepPatternTable& keep_table_;
    
    vector<vector<double>> dp_;
    HashMap<int, double> memo_;
    
    int current_x_;
    int current_hands_;
    
public:
    YachtDPSolver(const DicePatternTable& dice_table, const KeepPatternTable& keep_table)
        : dice_table_(dice_table), keep_table_(keep_table)
    {
        dp_.assign(Constants::UPPER_SUM_MAX,
                   vector<double>(Constants::USED_HANDS_MAX, 0.0));
    }
    
    void solve() {
        int total = Constants::USED_HANDS_MAX;
        int processed = 0;
        auto start = chrono::steady_clock::now();
        
        for (int hands = total - 2; hands >= 0; hands--) {
            for (int x = 0; x < Constants::UPPER_SUM_MAX; x++) {
                dp_[x][hands] = compute_turn(x, hands);
            }
            
            if (++processed % 100 == 0) {
                print_progress(processed, total, start);
            }
        }
    }
    
    const vector<vector<double>>& get_dp() const { return dp_; }
    double get_initial_score() const { return dp_[0][0]; }
    
private:
    double compute_turn(int x, int hands) {
        current_x_ = x;
        current_hands_ = hands;
        memo_.clear();
        return compute_stage(0, DiceUtil::encode(DiceUtil::empty()));
    }
    
    /// stage: 0=1回目振り後, 1=2回目振り後, 2=3回目振り後, 3=役選択
    double compute_stage(int stage, int state_code) {
        int key = (stage << 20) | state_code;
        auto it = memo_.find(key);
        if (it != memo_.end()) return it->second;
        
        int n_kept = DiceUtil::count_encoded(state_code);
        int n_roll = Constants::NUM_DICE - n_kept;
        
        double result;
        
        if (stage == 3) {
            result = compute_best_hand(state_code);
        }
        else if (stage == 2) {
            result = compute_final_roll(state_code, n_roll);
        }
        else {
            result = compute_roll_with_keep(stage, state_code, n_roll);
        }
        
        return memo_[key] = result;
    }
    
    double compute_best_hand(int dice_code) {
        Dice d = DiceUtil::decode(dice_code);
        double best = -1e18;
        
        for (int h = 0; h < Constants::NUM_HANDS; h++) {
            if ((current_hands_ >> h) & 1) continue;
            
            int pts = Scoring::calculate(h, d);
            int new_hands = current_hands_ | (1 << h);
            
            double val;
            if (Hand::is_upper(h)) {
                int new_x = min(Constants::UPPER_BONUS_THRESHOLD, current_x_ + pts);
                double bonus = (current_x_ < Constants::UPPER_BONUS_THRESHOLD &&
                               new_x >= Constants::UPPER_BONUS_THRESHOLD)
                               ? Constants::UPPER_BONUS_POINTS : 0.0;
                val = dp_[new_x][new_hands] + pts + bonus;
            } else {
                val = dp_[current_x_][new_hands] + pts;
            }
            best = max(best, val);
        }
        return best;
    }
    
    /// 最終振り（確率加重平均）
    double compute_final_roll(int state_code, int n_roll) {
        if (n_roll == 0) {
            return compute_stage(3, state_code);
        }
        
        Dice state = DiceUtil::decode(state_code);
        double total = 0.0;
        
        // ユニークパターンを確率付きで列挙
        for (const auto& pattern : dice_table_.get(n_roll)) {
            Dice final_d = DiceUtil::add(state, pattern.dice);
            total += pattern.probability * compute_stage(3, DiceUtil::encode(final_d));
        }
        
        return total;
    }
    
    /// キープ選択あり（確率加重平均）
    double compute_roll_with_keep(int stage, int state_code, int n_roll) {
        if (n_roll == 0) {
            return compute_stage(stage + 1, state_code);
        }
        
        Dice state = DiceUtil::decode(state_code);
        double total = 0.0;
        
        // ユニークパターンを確率付きで列挙
        for (const auto& roll_pattern : dice_table_.get(n_roll)) {
            double best = -1e18;
            
            // 全キープパターンを試す
            for (const auto& keep : keep_table_.get(roll_pattern.code)) {
                Dice new_state = DiceUtil::add(state, keep.keep);
                double val = compute_stage(stage + 1, DiceUtil::encode(new_state));
                best = max(best, val);
            }
            
            total += roll_pattern.probability * best;
        }
        
        return total;
    }
    
    void print_progress(int done, int total, chrono::steady_clock::time_point start) {
        auto now = chrono::steady_clock::now();
        double elapsed = chrono::duration<double>(now - start).count();
        double eta = (elapsed / done) * (total - done);
        cerr << "Progress: " << done << "/" << total
             << " (" << fixed << setprecision(1) << (100.0 * done / total) << "%), "
             << "ETA: " << int(eta/60) << "m " << int(fmod(eta,60)) << "s" << endl;
    }
};

// =============================================================================
// ファイル出力
// =============================================================================

void write_output(const string& filename, const vector<vector<double>>& dp) {
    ofstream ofs(filename);
    ofs << fixed << setprecision(6);
    
    ofs << R"(/**
 * ヨット DP テーブル（自動生成）
 * 
 * dp_table[upper_sum][used_hands] = その状態からの期待得点
 */

#pragma once
#include <vector>

namespace yacht {

const std::vector<std::vector<double>> dp_table = {
)";
    
    for (size_t x = 0; x < dp.size(); x++) {
        ofs << "    {";
        for (size_t h = 0; h < dp[x].size(); h++) {
            if (h > 0) ofs << ", ";
            ofs << dp[x][h];
        }
        ofs << (x < dp.size() - 1 ? "},\n" : "}\n");
    }
    
    ofs << R"(};

inline double get_expected_score(int upper_sum, int used_hands) {
    return dp_table[upper_sum][used_hands];
}

inline double get_initial_expected_score() {
    return dp_table[0][0];
}

} // namespace yacht
)";
}

// =============================================================================
// メイン
// =============================================================================

int main() {
    ios::sync_with_stdio(false);
    cin.tie(nullptr);
    
    cerr << "=== ヨット期待値計算（高速化版） ===" << endl;
    
    cerr << "出目パターンテーブルを構築中..." << endl;
    DicePatternTable dice_table;
    for (int n = 0; n <= 5; n++) {
        cerr << "  " << n << "個: " << dice_table.get(n).size() << " パターン" << endl;
    }
    
    cerr << "キープパターンテーブルを構築中..." << endl;
    KeepPatternTable keep_table;
    keep_table.compute(dice_table);
    cerr << "  総エントリ数: " << keep_table.patterns.size() << endl;
    
    cerr << "DP計算を開始..." << endl;
    auto start = chrono::steady_clock::now();
    
    YachtDPSolver solver(dice_table, keep_table);
    solver.solve();
    
    auto end = chrono::steady_clock::now();
    double elapsed = chrono::duration<double>(end - start).count();
    
    cerr << "\n=== 計算完了 ===" << endl;
    cerr << "計算時間: " << elapsed << " 秒" << endl;
    cerr << "期待得点: " << solver.get_initial_score() << endl;
    
    write_output("yacht_dp_table.hpp", solver.get_dp());
    cerr << "出力ファイル: yacht_dp_table.hpp" << endl;
    
    cout << fixed << setprecision(15);
    cout << solver.get_initial_score() << endl;
    
    return 0;
}