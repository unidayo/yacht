/* tslint:disable */
/* eslint-disable */

export enum Category {
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

export class Dice {
  free(): void;
  [Symbol.dispose](): void;
  get_values(): Uint8Array;
  set_values(values: Uint8Array): void;
  reset_holds(): void;
  toggle_hold(index: number): void;
  constructor();
  roll(): void;
  is_held(index: number): boolean;
  set_hold(index: number, hold: boolean): void;
  get_holds(): Uint8Array;
  get_locks(): Uint8Array;
  get_value(index: number): number;
  is_locked(index: number): boolean;
}

export class GameState {
  free(): void;
  [Symbol.dispose](): void;
  reset_holds(): void;
  toggle_hold(index: number): void;
  get_ai_score(category_index: number): number;
  get_ai_total(): number;
  is_game_over(): boolean;
  get_dice_holds(): Uint8Array;
  get_dice_locks(): Uint8Array;
  get_rolls_left(): number;
  get_dice_values(): Uint8Array;
  select_category(category_index: number): boolean;
  get_player_score(category_index: number): number;
  get_player_total(): number;
  get_ai_upper_bonus(): number;
  get_ai_upper_total(): number;
  get_current_player(): number;
  get_potential_score(category_index: number): number;
  get_player_upper_bonus(): number;
  get_player_upper_total(): number;
  get_available_categories(): Uint8Array;
  constructor();
  to_json(): string;
  roll_dice(): boolean;
}

export class ScoreBoard {
  free(): void;
  [Symbol.dispose](): void;
  is_complete(): boolean;
  get_lower_total(): number;
  get_upper_bonus(): number;
  get_upper_total(): number;
  available_categories(): Uint8Array;
  constructor();
  is_used(category: Category): boolean;
  get_score(category: Category): number;
  get_total(): number;
  set_score(category: Category, score: number): boolean;
}

export class YachtAI {
  free(): void;
  [Symbol.dispose](): void;
  get_holds_decision(game: GameState): Uint8Array;
  get_category_decision(game: GameState): number;
  constructor();
  play_turn(game: GameState): string;
}

export function calculate_score_js(dice: Uint8Array, category_index: number): number;

export function get_category_name(category_index: number): string;

export function get_category_name_ja(category_index: number): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_dice_free: (a: number, b: number) => void;
  readonly __wbg_gamestate_free: (a: number, b: number) => void;
  readonly __wbg_scoreboard_free: (a: number, b: number) => void;
  readonly __wbg_yachtai_free: (a: number, b: number) => void;
  readonly calculate_score_js: (a: number, b: number, c: number) => number;
  readonly dice_get_holds: (a: number) => [number, number];
  readonly dice_get_locks: (a: number) => [number, number];
  readonly dice_get_value: (a: number, b: number) => number;
  readonly dice_get_values: (a: number) => [number, number];
  readonly dice_is_held: (a: number, b: number) => number;
  readonly dice_is_locked: (a: number, b: number) => number;
  readonly dice_new: () => number;
  readonly dice_reset_holds: (a: number) => void;
  readonly dice_roll: (a: number) => void;
  readonly dice_set_hold: (a: number, b: number, c: number) => void;
  readonly dice_set_values: (a: number, b: number, c: number) => void;
  readonly dice_toggle_hold: (a: number, b: number) => void;
  readonly gamestate_get_ai_score: (a: number, b: number) => number;
  readonly gamestate_get_ai_total: (a: number) => number;
  readonly gamestate_get_ai_upper_bonus: (a: number) => number;
  readonly gamestate_get_ai_upper_total: (a: number) => number;
  readonly gamestate_get_available_categories: (a: number) => [number, number];
  readonly gamestate_get_current_player: (a: number) => number;
  readonly gamestate_get_dice_holds: (a: number) => [number, number];
  readonly gamestate_get_dice_locks: (a: number) => [number, number];
  readonly gamestate_get_dice_values: (a: number) => [number, number];
  readonly gamestate_get_player_score: (a: number, b: number) => number;
  readonly gamestate_get_player_total: (a: number) => number;
  readonly gamestate_get_player_upper_bonus: (a: number) => number;
  readonly gamestate_get_player_upper_total: (a: number) => number;
  readonly gamestate_get_potential_score: (a: number, b: number) => number;
  readonly gamestate_get_rolls_left: (a: number) => number;
  readonly gamestate_is_game_over: (a: number) => number;
  readonly gamestate_new: () => number;
  readonly gamestate_roll_dice: (a: number) => number;
  readonly gamestate_select_category: (a: number, b: number) => number;
  readonly gamestate_to_json: (a: number) => [number, number];
  readonly gamestate_toggle_hold: (a: number, b: number) => void;
  readonly get_category_name: (a: number) => [number, number];
  readonly get_category_name_ja: (a: number) => [number, number];
  readonly scoreboard_available_categories: (a: number) => [number, number];
  readonly scoreboard_get_lower_total: (a: number) => number;
  readonly scoreboard_get_score: (a: number, b: number) => number;
  readonly scoreboard_get_total: (a: number) => number;
  readonly scoreboard_get_upper_bonus: (a: number) => number;
  readonly scoreboard_get_upper_total: (a: number) => number;
  readonly scoreboard_is_complete: (a: number) => number;
  readonly scoreboard_is_used: (a: number, b: number) => number;
  readonly scoreboard_new: () => number;
  readonly scoreboard_set_score: (a: number, b: number, c: number) => number;
  readonly yachtai_get_category_decision: (a: number, b: number) => number;
  readonly yachtai_get_holds_decision: (a: number, b: number) => [number, number];
  readonly yachtai_play_turn: (a: number, b: number) => [number, number];
  readonly gamestate_reset_holds: (a: number) => void;
  readonly yachtai_new: () => number;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_externrefs: WebAssembly.Table;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
