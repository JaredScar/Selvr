/* tslint:disable */
/* eslint-disable */

/**
 * Call an exported Selvr function by name.
 *
 * `args_json` is a JSON array of arguments encoded as primitive JS values.
 * Returns a JSON-encoded result or an error string.
 */
export function SELVR_call(name: string, args_json: string): string;

/**
 * Return buffered console output (newline-separated) and clear the buffer.
 */
export function SELVR_drain_output(): string;

/**
 * Load a Selvr bytecode module from a `Uint8Array`.
 */
export function SELVR_load(bytes: Uint8Array): void;

/**
 * Resume a suspended async coroutine by its numeric ID.
 */
export function SELVR_resume(_id: number): void;

/**
 * Return the runtime version string.
 */
export function SELVR_version(): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly SELVR_call: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly SELVR_drain_output: () => [number, number];
    readonly SELVR_load: (a: number, b: number) => [number, number];
    readonly SELVR_resume: (a: number) => void;
    readonly SELVR_version: () => [number, number];
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
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
