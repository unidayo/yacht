import { useState, useEffect, useCallback } from 'react'
import init, { GameState, YachtAI, get_category_name_ja } from './wasm/yacht_core'
import './App.css'

type GamePhase = 'loading' | 'ready' | 'playing' | 'ai_turn' | 'game_over'

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

  // WASM初期化
  useEffect(() => {
    init().then(() => {
      setWasmLoaded(true)
      setPhase('ready')
    })
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
    if (!game || rollsLeft === 0 || phase !== 'playing') return

    setRolling(true)

    // アニメーション用に複数回更新
    let animCount = 0
    const animInterval = setInterval(() => {
      setDiceValues(prev => prev.map((v, i) =>
        diceHolds[i] ? v : Math.floor(Math.random() * 6) + 1
      ))
      animCount++
      if (animCount >= 8) {
        clearInterval(animInterval)
        game.roll_dice()
        syncGameState(game)
        setRolling(false)

        if (game.get_rolls_left() === 0) {
          setMessage('カテゴリを選択してください')
        } else {
          setMessage(`残り${game.get_rolls_left()}回振れます`)
        }
      }
    }, 50)
  }, [game, rollsLeft, phase, diceHolds, syncGameState])

  // サイコロをホールド/アンホールド
  const toggleHold = useCallback((index: number) => {
    if (!game || rollsLeft === 3 || rollsLeft === 0 || phase !== 'playing') return
    game.toggle_hold(index)
    syncGameState(game)
  }, [game, rollsLeft, phase, syncGameState])

  // AIのサイコロアニメーション
  const animateAiRoll = useCallback(async (currentHolds: boolean[]) => {
    setRolling(true)
    for (let i = 0; i < 8; i++) {
      setDiceValues(prev => prev.map((v, idx) =>
        currentHolds[idx] ? v : Math.floor(Math.random() * 6) + 1
      ))
      await sleep(50)
    }
    setRolling(false)
  }, [])

  // AIのターンを実行（アニメーション付き）
  const executeAiTurn = useCallback(async (g: GameState, aiPlayer: YachtAI) => {
    setPhase('ai_turn')

    // 1回目のロール
    setMessage('AIがサイコロを振っています...')
    await animateAiRoll([false, false, false, false, false])
    g.roll_dice()
    syncGameState(g)
    await sleep(800)

    // 2回目のロール判断
    if (g.get_rolls_left() > 0) {
      setMessage('AIが考えています...')
      await sleep(600)

      const holds1 = Array.from(aiPlayer.get_holds_decision(g)).map(h => h === 1)

      // ホールドするサイコロを表示
      if (holds1.some(h => h)) {
        setMessage('AIがサイコロをキープしています...')
        setDiceHolds(holds1)
        for (let i = 0; i < 5; i++) {
          if (holds1[i]) {
            g.toggle_hold(i)
          }
        }
        await sleep(800)
      }

      // 全てキープなら振らない
      if (holds1.every(h => h)) {
        setMessage('AIは振り直しません')
        await sleep(600)
      } else {
        setMessage('AIがサイコロを振り直しています...')
        await animateAiRoll(holds1)
        g.roll_dice()
        syncGameState(g)
        await sleep(800)
      }
    }

    // 3回目のロール判断
    if (g.get_rolls_left() > 0) {
      setMessage('AIが考えています...')
      await sleep(600)

      // 現在のホールド状態を取得
      const currentHolds = Array.from(g.get_dice_holds()).map(h => h === 1)
      const holds2 = Array.from(aiPlayer.get_holds_decision(g)).map(h => h === 1)

      // ホールド状態が変わる場合のみ更新
      let holdsChanged = false
      for (let i = 0; i < 5; i++) {
        if (currentHolds[i] !== holds2[i]) {
          g.toggle_hold(i)
          holdsChanged = true
        }
      }

      if (holdsChanged || holds2.some(h => h)) {
        setMessage('AIがサイコロをキープしています...')
        setDiceHolds(holds2)
        await sleep(800)
      }

      // 全てキープなら振らない
      if (holds2.every(h => h)) {
        setMessage('AIは振り直しません')
        await sleep(600)
      } else {
        setMessage('AIがサイコロを振り直しています...')
        await animateAiRoll(holds2)
        g.roll_dice()
        syncGameState(g)
        await sleep(800)
      }
    }

    // カテゴリ選択
    setMessage('AIが役を選んでいます...')
    await sleep(600)

    const categoryChoice = aiPlayer.get_category_decision(g)
    setHighlightCategory(categoryChoice)
    setMessage(`AIが「${get_category_name_ja(categoryChoice)}」を選択しました`)
    await sleep(1200)

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
                  {[0, 1, 2, 3, 4, 5].map(i => (
                    <tr key={i} className={playerScores[i] === null && currentPlayer === 0 && rollsLeft < 3 ? 'selectable' : ''}>
                      <td className="category-name">{get_category_name_ja(i)}</td>
                      <td
                        className={`score player-score ${playerScores[i] === null ? 'empty' : ''}`}
                        onClick={() => selectCategory(i)}
                      >
                        {playerScores[i] !== null
                          ? playerScores[i]
                          : (rollsLeft < 3 && currentPlayer === 0 ? <span className="potential">{getPotentialScore(i)}</span> : '-')}
                      </td>
                      <td className={`score ai-score ${aiScores[i] === null ? 'empty' : ''} ${highlightCategory === i ? 'highlight' : ''}`}>
                        {aiScores[i] !== null ? aiScores[i] : '-'}
                      </td>
                    </tr>
                  ))}
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
                  {[6, 7, 8, 9, 10, 11].map(i => (
                    <tr key={i} className={playerScores[i] === null && currentPlayer === 0 && rollsLeft < 3 ? 'selectable' : ''}>
                      <td className="category-name">{get_category_name_ja(i)}</td>
                      <td
                        className={`score player-score ${playerScores[i] === null ? 'empty' : ''}`}
                        onClick={() => selectCategory(i)}
                      >
                        {playerScores[i] !== null
                          ? playerScores[i]
                          : (rollsLeft < 3 && currentPlayer === 0 ? <span className="potential">{getPotentialScore(i)}</span> : '-')}
                      </td>
                      <td className={`score ai-score ${aiScores[i] === null ? 'empty' : ''} ${highlightCategory === i ? 'highlight' : ''}`}>
                        {aiScores[i] !== null ? aiScores[i] : '-'}
                      </td>
                    </tr>
                  ))}
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
    <div className="dice-face">
      {dotPositions[value]?.map((pos, i) => (
        <div key={i} className={`dot ${pos}`} />
      ))}
    </div>
  )
}

export default App
