import { useState, useEffect, useCallback } from 'react'
import init, { GameState, YachtAI, get_category_name_ja } from './wasm/yacht_core'
import './App.css'

type GamePhase = 'loading' | 'ready' | 'playing' | 'ai_turn' | 'game_over'

interface CategoryRecommendation {
  category: number
  score: number
  expected: number
}

interface HoldRecommendation {
  holds: number[]
  expected: number
}

const sleep = (ms: number) => new Promise(resolve => setTimeout(resolve, ms))

function App() {
  const [wasmLoaded, setWasmLoaded] = useState(false)
  const [game, setGame] = useState<GameState | null>(null)
  const [ai, setAi] = useState<YachtAI | null>(null)
  const [phase, setPhase] = useState<GamePhase>('loading')
  const [diceValues, setDiceValues] = useState<number[]>([1, 1, 1, 1, 1])
  const [diceHolds, setDiceHolds] = useState<boolean[]>([false, false, false, false, false])
  const [rollsLeft, setRollsLeft] = useState(3)
  const [currentPlayer, setCurrentPlayer] = useState(0)
  const [playerScores, setPlayerScores] = useState<(number | null)[]>(Array(12).fill(null))
  const [aiScores, setAiScores] = useState<(number | null)[]>(Array(12).fill(null))
  const [playerTotal, setPlayerTotal] = useState(0)
  const [aiTotal, setAiTotal] = useState(0)
  const [playerUpperTotal, setPlayerUpperTotal] = useState(0)
  const [aiUpperTotal, setAiUpperTotal] = useState(0)
  const [playerBonus, setPlayerBonus] = useState(0)
  const [aiBonus, setAiBonus] = useState(0)
  const [message, setMessage] = useState('')
  const [rolling, setRolling] = useState(false)
  const [highlightCategory, setHighlightCategory] = useState<number | null>(null)
  const [recommendedCategories, setRecommendedCategories] = useState<CategoryRecommendation[]>([])
  const [recommendedHolds, setRecommendedHolds] = useState<HoldRecommendation[]>([])
  const [showRecommendations, setShowRecommendations] = useState(true)

  // WASM初期化
  useEffect(() => {
    init().then(() => {
      setWasmLoaded(true)
      setPhase('ready')
    })
  }, [])

  // 推奨を更新
  const updateRecommendations = useCallback((g: GameState, aiPlayer: YachtAI | null) => {
    if (!aiPlayer || g.get_current_player() !== 0) {
      setRecommendedCategories([])
      setRecommendedHolds([])
      return
    }

    // ロール後のみ推奨を表示
    if (g.get_rolls_left() === 3) {
      setRecommendedCategories([])
      setRecommendedHolds([])
      return
    }

    try {
      // カテゴリ推奨
      const catJson = aiPlayer.get_top_category_choices(g)
      const categories: CategoryRecommendation[] = JSON.parse(catJson)
      setRecommendedCategories(categories)

      // キープ推奨（まだ振れる場合のみ）
      if (g.get_rolls_left() > 0) {
        const holdJson = aiPlayer.get_top_hold_choices(g)
        const holds: HoldRecommendation[] = JSON.parse(holdJson)
        setRecommendedHolds(holds)
      } else {
        setRecommendedHolds([])
      }
    } catch (e) {
      console.error('Failed to get recommendations:', e)
    }
  }, [])

  // ゲーム状態の同期
  const syncGameState = useCallback((g: GameState) => {
    const values = Array.from(g.get_dice_values())
    const holds = Array.from(g.get_dice_holds()).map(h => h === 1)
    setDiceValues(values)
    setDiceHolds(holds)
    setRollsLeft(g.get_rolls_left())
    setCurrentPlayer(g.get_current_player())

    const pScores: (number | null)[] = []
    const aScores: (number | null)[] = []
    for (let i = 0; i < 12; i++) {
      const ps = g.get_player_score(i)
      const as = g.get_ai_score(i)
      pScores.push(ps >= 0 ? ps : null)
      aScores.push(as >= 0 ? as : null)
    }
    setPlayerScores(pScores)
    setAiScores(aScores)
    setPlayerTotal(g.get_player_total())
    setAiTotal(g.get_ai_total())
    setPlayerUpperTotal(g.get_player_upper_total())
    setAiUpperTotal(g.get_ai_upper_total())
    setPlayerBonus(g.get_player_upper_bonus())
    setAiBonus(g.get_ai_upper_bonus())

    if (g.is_game_over()) {
      setPhase('game_over')
      const pt = g.get_player_total()
      const at = g.get_ai_total()
      if (pt > at) {
        setMessage('あなたの勝ちです！')
      } else if (at > pt) {
        setMessage('AIの勝ちです')
      } else {
        setMessage('引き分けです')
      }
    }
  }, [])

  // 新規ゲーム開始
  const startGame = useCallback(() => {
    const newGame = new GameState()
    const newAi = new YachtAI()
    setGame(newGame)
    setAi(newAi)
    setPhase('playing')
    setMessage('サイコロを振ってください')
    syncGameState(newGame)
  }, [syncGameState])

  // サイコロを振る
  const rollDice = useCallback(() => {
    if (!game || !ai || rollsLeft === 0 || phase !== 'playing') return

    setRolling(true)

    // アニメーション用に複数回更新
    let animCount = 0
    const animInterval = setInterval(() => {
      setDiceValues(prev => prev.map((v, i) =>
        diceHolds[i] ? v : Math.floor(Math.random() * 6) + 1
      ))
      animCount++
      if (animCount >= 6) {
        clearInterval(animInterval)
        game.roll_dice()
        syncGameState(game)
        updateRecommendations(game, ai)
        setRolling(false)

        if (game.get_rolls_left() === 0) {
          setMessage('カテゴリを選択してください')
        } else {
          setMessage(`残り${game.get_rolls_left()}回振れます`)
        }
      }
    }, 35)
  }, [game, ai, rollsLeft, phase, diceHolds, syncGameState, updateRecommendations])

  // サイコロをホールド（ロールで確定したキープは解除不可）
  const toggleHold = useCallback((index: number) => {
    if (!game || rollsLeft === 3 || rollsLeft === 0 || phase !== 'playing') return
    game.toggle_hold(index)
    syncGameState(game)
  }, [game, rollsLeft, phase, syncGameState])

  // AIのサイコロアニメーション
  const animateAiRoll = useCallback(async (currentHolds: boolean[]) => {
    setRolling(true)
    for (let i = 0; i < 6; i++) {
      setDiceValues(prev => prev.map((v, idx) =>
        currentHolds[idx] ? v : Math.floor(Math.random() * 6) + 1
      ))
      await sleep(35)
    }
    setRolling(false)
  }, [])

  // AIのターンを実行（アニメーション付き）
  const executeAiTurn = useCallback(async (g: GameState, aiPlayer: YachtAI) => {
    setPhase('ai_turn')
    // 推奨表示をクリア
    setRecommendedCategories([])
    setRecommendedHolds([])

    // 1回目のロール
    setMessage('AIがサイコロを振っています...')
    await animateAiRoll([false, false, false, false, false])
    g.roll_dice()
    syncGameState(g)
    await sleep(500)

    // 2回目のロール判断
    let skipThirdRoll = false
    if (g.get_rolls_left() > 0) {
      setMessage('AIが考えています...')
      await sleep(400)

      const holds1 = Array.from(aiPlayer.get_holds_decision(g)).map(h => h === 1)

      // ホールドするサイコロを設定
      for (let i = 0; i < 5; i++) {
        if (holds1[i]) {
          g.toggle_hold(i)
        }
      }
      // 実際のゲーム状態を反映
      syncGameState(g)

      const actualHolds1 = Array.from(g.get_dice_holds()).map(h => h === 1)
      if (actualHolds1.some(h => h)) {
        setMessage('AIがサイコロをキープしています...')
        await sleep(500)
      }

      // 全てキープなら振らない→カテゴリ選択へ
      if (actualHolds1.every(h => h)) {
        setMessage('AIは振り直しません')
        await sleep(400)
        skipThirdRoll = true
      } else {
        setMessage('AIがサイコロを振り直しています...')
        await animateAiRoll(actualHolds1)
        g.roll_dice()
        syncGameState(g)
        await sleep(500)
      }
    }

    // 3回目のロール判断（2回目で全てキープした場合はスキップ）
    if (!skipThirdRoll && g.get_rolls_left() > 0) {
      setMessage('AIが考えています...')
      await sleep(400)

      // AIの決定を取得
      const holds2 = Array.from(aiPlayer.get_holds_decision(g)).map(h => h === 1)

      // 新しくキープするダイスのみtoggle（ロック済みは自動的に拒否される）
      for (let i = 0; i < 5; i++) {
        if (holds2[i]) {
          g.toggle_hold(i)
        }
      }
      // 実際のゲーム状態を反映
      syncGameState(g)

      const actualHolds2 = Array.from(g.get_dice_holds()).map(h => h === 1)
      if (actualHolds2.some(h => h)) {
        setMessage('AIがサイコロをキープしています...')
        await sleep(500)
      }

      // 全てキープなら振らない
      if (actualHolds2.every(h => h)) {
        setMessage('AIは振り直しません')
        await sleep(400)
      } else {
        setMessage('AIがサイコロを振り直しています...')
        await animateAiRoll(actualHolds2)
        g.roll_dice()
        syncGameState(g)
        await sleep(500)
      }
    }

    // カテゴリ選択
    setMessage('AIが役を選んでいます...')
    await sleep(400)

    const categoryChoice = aiPlayer.get_category_decision(g)
    setHighlightCategory(categoryChoice)
    setMessage(`AIが「${get_category_name_ja(categoryChoice)}」を選択しました`)
    await sleep(800)

    g.select_category(categoryChoice)
    setHighlightCategory(null)
    syncGameState(g)

    if (!g.is_game_over()) {
      setPhase('playing')
      setMessage('サイコロを振ってください')
    }
  }, [animateAiRoll, syncGameState])

  // カテゴリ選択
  const selectCategory = useCallback((categoryIndex: number) => {
    if (!game || !ai || rollsLeft === 3 || phase !== 'playing') return
    if (playerScores[categoryIndex] !== null) return

    const success = game.select_category(categoryIndex)
    if (success) {
      syncGameState(game)

      if (!game.is_game_over()) {
        executeAiTurn(game, ai)
      }
    }
  }, [game, ai, rollsLeft, phase, playerScores, syncGameState, executeAiTurn])

  // 現在の出目での各カテゴリの得点を計算
  const getPotentialScore = useCallback((categoryIndex: number): number => {
    if (!game) return 0
    return game.get_potential_score(categoryIndex)
  }, [game])

  if (!wasmLoaded) {
    return <div className="loading">読み込み中...</div>
  }

  return (
    <div className="app">
      <h1>ヨット</h1>

      {phase === 'ready' && (
        <div className="start-screen">
          <p>5つのサイコロを使って役を作るゲームです</p>
          <button className="start-button" onClick={startGame}>
            ゲーム開始
          </button>
          <div className="recommendations-toggle-area">
            <button
              className={`toggle-recommendations ${showRecommendations ? 'active' : ''}`}
              onClick={() => setShowRecommendations(!showRecommendations)}
            >
              {showRecommendations ? '推奨を非表示' : '推奨を表示'}
            </button>
          </div>
        </div>
      )}

      {(phase === 'playing' || phase === 'ai_turn' || phase === 'game_over') && (
        <>
          <div className="message">{message}</div>

          <div className="game-area">
            {/* サイコロエリア */}
            <div className="dice-area">
              <div className="dice-container">
                {diceValues.map((value, index) => (
                  <div
                    key={index}
                    className={`die ${diceHolds[index] ? (phase === 'ai_turn' ? 'ai-held' : 'held') : ''} ${rolling && !diceHolds[index] ? 'rolling' : ''}`}
                    onClick={() => toggleHold(index)}
                  >
                    <DiceFace value={value} />
                  </div>
                ))}
              </div>

              <div className="roll-info">
                残り {rollsLeft} 回
              </div>

              <button
                className="roll-button"
                onClick={rollDice}
                disabled={rollsLeft === 0 || phase !== 'playing' || rolling}
              >
                {rollsLeft === 3 ? 'サイコロを振る' : '振り直す'}
              </button>

              {/* 推奨表示トグルボタン（ゲーム中は常に表示） */}
              <div className="recommendations-toggle-area">
                <button
                  className={`toggle-recommendations ${showRecommendations ? 'active' : ''}`}
                  onClick={() => setShowRecommendations(!showRecommendations)}
                >
                  {showRecommendations ? '推奨を非表示' : '推奨を表示'}
                </button>
              </div>

              {/* 推奨表示エリア */}
              {showRecommendations && phase === 'playing' && (recommendedHolds.length > 0 || recommendedCategories.length > 0) && (
                <div className="recommendations">
                  <div className="recommendations-header">
                    <span>{rollsLeft > 0 ? '推奨キープ' : '推奨カテゴリ'}</span>
                  </div>
                  {/* 推奨キープ（まだ振れる場合） */}
                  {rollsLeft > 0 && recommendedHolds.map((rec, idx) => (
                    <div key={idx} className={`recommendation-item ${idx === 0 ? 'best' : ''}`}>
                      <span className="rank">{idx + 1}.</span>
                      <span className="hold-pattern">
                        {rec.holds.map((h, i) => (
                          <span key={i} className={`hold-indicator ${h === 1 ? 'keep' : 'reroll'}`}>
                            {diceValues[i]}
                          </span>
                        ))}
                      </span>
                      <span className="expected">期待値: {rec.expected.toFixed(1)}</span>
                    </div>
                  ))}
                  {/* 推奨カテゴリ（振り終わった場合） */}
                  {rollsLeft === 0 && recommendedCategories.map((rec, idx) => (
                    <div key={idx} className={`recommendation-item ${idx === 0 ? 'best' : ''}`}>
                      <span className="rank">{idx + 1}.</span>
                      <span className="category-name">{get_category_name_ja(rec.category)}</span>
                      <span className="score-info">({rec.score}点)</span>
                      <span className="expected">期待値: {rec.expected.toFixed(1)}</span>
                    </div>
                  ))}
                </div>
              )}
            </div>

            {/* スコアボード */}
            <div className="scoreboard">
              <table>
                <thead>
                  <tr>
                    <th>役</th>
                    <th>あなた</th>
                    <th>AI</th>
                  </tr>
                </thead>
                <tbody>
                  {/* 上段 */}
                  {[0, 1, 2, 3, 4, 5].map(i => {
                    const recIndex = recommendedCategories.findIndex(r => r.category === i)
                    const isRecommended = showRecommendations && recIndex !== -1
                    const canSelect = playerScores[i] === null && currentPlayer === 0 && rollsLeft < 3
                    return (
                      <tr
                        key={i}
                        className={`${canSelect ? 'selectable' : ''} ${isRecommended ? `recommended rank-${recIndex + 1}` : ''}`}
                        onClick={() => canSelect && selectCategory(i)}
                      >
                        <td className="category-name">
                          {get_category_name_ja(i)}
                          {isRecommended && <span className={`rec-badge rank-${recIndex + 1}`}>{recIndex + 1}</span>}
                        </td>
                        <td className={`score player-score ${playerScores[i] === null ? 'empty' : ''}`}>
                          {playerScores[i] !== null
                            ? playerScores[i]
                            : (rollsLeft < 3 && currentPlayer === 0 ? <span className="potential">{getPotentialScore(i)}</span> : '-')}
                        </td>
                        <td className={`score ai-score ${aiScores[i] === null ? 'empty' : ''} ${highlightCategory === i ? 'highlight' : ''}`}>
                          {aiScores[i] !== null ? aiScores[i] : '-'}
                        </td>
                      </tr>
                    )
                  })}
                  <tr className="subtotal">
                    <td>上段小計</td>
                    <td>{playerUpperTotal}</td>
                    <td>{aiUpperTotal}</td>
                  </tr>
                  <tr className="bonus">
                    <td>ボーナス (63点以上で+35)</td>
                    <td>{playerBonus > 0 ? `+${playerBonus}` : '-'}</td>
                    <td>{aiBonus > 0 ? `+${aiBonus}` : '-'}</td>
                  </tr>

                  {/* 下段 */}
                  {[6, 7, 8, 9, 10, 11].map(i => {
                    const recIndex = recommendedCategories.findIndex(r => r.category === i)
                    const isRecommended = showRecommendations && recIndex !== -1
                    const canSelect = playerScores[i] === null && currentPlayer === 0 && rollsLeft < 3
                    return (
                      <tr
                        key={i}
                        className={`${canSelect ? 'selectable' : ''} ${isRecommended ? `recommended rank-${recIndex + 1}` : ''}`}
                        onClick={() => canSelect && selectCategory(i)}
                      >
                        <td className="category-name">
                          {get_category_name_ja(i)}
                          {isRecommended && <span className={`rec-badge rank-${recIndex + 1}`}>{recIndex + 1}</span>}
                        </td>
                        <td className={`score player-score ${playerScores[i] === null ? 'empty' : ''}`}>
                          {playerScores[i] !== null
                            ? playerScores[i]
                            : (rollsLeft < 3 && currentPlayer === 0 ? <span className="potential">{getPotentialScore(i)}</span> : '-')}
                        </td>
                        <td className={`score ai-score ${aiScores[i] === null ? 'empty' : ''} ${highlightCategory === i ? 'highlight' : ''}`}>
                          {aiScores[i] !== null ? aiScores[i] : '-'}
                        </td>
                      </tr>
                    )
                  })}
                  <tr className="total">
                    <td>合計</td>
                    <td className="player-total">{playerTotal}</td>
                    <td className="ai-total">{aiTotal}</td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>

          {phase === 'game_over' && (
            <button className="restart-button" onClick={startGame}>
              もう一度プレイ
            </button>
          )}
        </>
      )}

      {/* ルール説明 */}
      <div className="rules-section">
        <h2>遊び方</h2>
        <div className="rules-content">
          <div className="rule-block">
            <h3>基本ルール</h3>
            <ul>
              <li>5つのサイコロを振って役を作るゲームです</li>
              <li>1ターンに最大3回まで振ることができます</li>
              <li>振った後、キープしたいサイコロをクリックして選択できます</li>
              <li>12個の役を埋めたらゲーム終了です</li>
            </ul>
          </div>

          <div className="rule-block">
            <h3>上段の役（数字の目）</h3>
            <ul>
              <li><strong>1の目〜6の目</strong>：該当する目の合計点（例：4が3つなら12点）</li>
              <li><strong>ボーナス</strong>：上段の合計が63点以上なら+35点</li>
            </ul>
          </div>

          <div className="rule-block">
            <h3>下段の役</h3>
            <ul>
              <li><strong>フルハウス</strong>：同じ目3つ＋同じ目2つ → 全ての目の合計点</li>
              <li><strong>フォーオブアカインド</strong>：同じ目4つ以上 → 全ての目の合計点</li>
              <li><strong>スモールストレート</strong>：4つ連続（1-2-3-4, 2-3-4-5, 3-4-5-6）→ 15点</li>
              <li><strong>ビッグストレート</strong>：5つ連続（1-2-3-4-5 または 2-3-4-5-6）→ 30点</li>
              <li><strong>チョイス</strong>：どんな出目でもOK → 全ての目の合計点</li>
              <li><strong>ヨット</strong>：5つ全て同じ目 → 50点</li>
            </ul>
          </div>

          <div className="rule-block">
            <h3>ヒント</h3>
            <ul>
              <li>上段ボーナス（+35点）を狙うなら、各目で平均3つ以上を目指しましょう</li>
              <li>「推奨を表示」で最適なプレイの参考にできます</li>
              <li>役が作れない場合は、0点でも埋める必要があります</li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  )
}

// サイコロの目を描画するコンポーネント
function DiceFace({ value }: { value: number }) {
  const dotPositions: Record<number, string[]> = {
    1: ['center'],
    2: ['top-right', 'bottom-left'],
    3: ['top-right', 'center', 'bottom-left'],
    4: ['top-left', 'top-right', 'bottom-left', 'bottom-right'],
    5: ['top-left', 'top-right', 'center', 'bottom-left', 'bottom-right'],
    6: ['top-left', 'top-right', 'middle-left', 'middle-right', 'bottom-left', 'bottom-right'],
  }

  return (
    <div className={`dice-face ${value === 1 ? 'one' : ''}`}>
      {dotPositions[value]?.map((pos, i) => (
        <div key={i} className={`dot ${pos}`} />
      ))}
    </div>
  )
}

export default App
