/*
 * kineti.h — C-ABI for the Kineti agent harness (v0.2.0, Phase 9).
 * v0.2: product is gateway + receipt (evidence/ship-check/verify); kineti_run()
 * below is the frozen 13-stage pipeline (legacy, use `kineti run --legacy`).
 *
 * Link against libkineti (cdylib). All calls MUST be made with the target
 * project directory as the process working directory.
 *
 * Memory contract:
 *   - Input strings: NUL-terminated UTF-8. Owned by the CALLER; Kineti
 *     never frees or retains them.
 *   - Output payloads (`KinetiResult.payload`): owned by the CALLER once
 *     returned. Release each one exactly once with kineti_free_string().
 *     NULL payload means out-of-memory.
 *   - kineti_version() points at static storage — do NOT free.
 *   - Panics never escape: failures arrive as ok=false with an error string.
 *
 * Thread safety: functions are independent; do not run two kineti_run()
 * calls concurrently in one process (the pipeline owns process-wide state).
 */

#ifndef KINETI_H
#define KINETI_H

#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Opaque owned UTF-8 string (NUL-terminated). */
typedef struct KinetiResult {
    bool ok;            /* false => payload holds the error text          */
    char *payload;      /* JSON on success, error text on failure         */
} KinetiResult;

/* Static library version string ("0.2.0"). Do not free. */
const char *kineti_version(void);

/* Release a payload obtained from any KinetiResult. NULL is a no-op. */
void kineti_free_string(char *ptr);

/*
 * Run the governed 13-stage pipeline (legacy, frozen at v0.1.0 — use `kineti run --legacy`)
 * against the current working directory.
 *
 * args_json — JSON object:
 *   {
 *     "goal":              str   (required, non-empty)
 *     "provider":          str   (optional, default "gemini")
 *     "model":             str   (optional)
 *     "cap":               number (optional global spend cap override)
 *     "mode":              "single" | "swarm"   (optional)
 *     "auto_approve_spec": bool   (optional; caller takes §10.2
 *                                  responsibility; audit logs
 *                                  "ffi auto-approval")
 *   }
 *
 * Payload on success: {"exit", "stage_reached", "spec_approved",
 *                      "shipped_at"}
 */
KinetiResult kineti_run(const char *args_json);

/*
 * Verify journal history in the current working directory.
 *
 * args_json — JSON object: {"all": bool}  (all=true → full DAG check;
 *            may also be "{}")
 *
 * all=false payload: {"ok", "records", "head"}
 * all=true  payload: {"ok", "main_records", "main_head",
 *                     "branches":[{branch,records,head}],
 *                     "orphans":[...], "errors":[...]}
 * ok=false with ok=false payload = tampering detected (error text).
 */
KinetiResult kineti_verify(const char *args_json);

/*
 * Full receipt summary as JSON for the current working directory.
 * args_json may be NULL or "{}".
 *
 * Payload: {"goal","records","chain_head","causal_edges","last_run",
 *           "spend":{coordinator_usd,workers_usd,total_usd,workers:[..]},
 *           "gates":[{at,kind,detail}],
 *           "dag":{main_records,main_head,branches_merged,orphans,errors},
 *           "egress":[{tag,records}],
 *           "clean_files_violations": null | number,
 *           "history_clean": bool}
 */
KinetiResult kineti_receipt(const char *args_json);

#ifdef __cplusplus
} /* extern "C" */
#endif

#endif /* KINETI_H */
