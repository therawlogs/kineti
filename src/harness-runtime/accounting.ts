import path from "node:path";
import { appendJsonl, loadLimits } from "../../bin/lib.ts";
import { readSpending, saveSpending, spendingStop, withAccountingLock } from "../../bin/spending-state.ts";
export function assertRunHeadroom(root: string, runId: string, sessionId: string) {
    withAccountingLock(root, () => {
        const s = readSpending(root), pending = s.routing_reservations ?? {}, own = pending[runId], limits = loadLimits(root);
        if (!own || own.session_id !== sessionId)
            throw new Error("run reservation missing or foreign");
        if (Object.entries(pending).some(([id, r]) => id !== runId && r.status === "unknown"))
            throw new Error("another run cost is unresolved");
        const stop = spendingStop({ ...s, routing_reservations: undefined }, limits);
        if (stop)
            throw new Error(stop);
        const rows = Object.values(pending), total = rows.reduce((n, r) => n + r.microcents, 0), stage = rows.filter(r => r.stage === own.stage).reduce((n, r) => n + r.microcents, 0);
        if (s.total_microcents + total >= Math.round(limits.globalUsd * limits.safetyFactor * 1e6) || (s.by_stage_microcents[own.stage] ?? 0) + stage >= Math.round((limits.perStage[own.stage] ?? limits.perStageDefaultUsd) * limits.safetyFactor * 1e6))
            throw new Error("reserved work exhausted total/stage headroom");
    });
}
/** Only for this controller's definite pre-inference failure, never historical unknown cost. */
export function cancelUnlaunched(root: string, runId: string, sessionId: string) {
    withAccountingLock(root, () => {
        const s = readSpending(root), pending = s.routing_reservations ?? {}, r = pending[runId];
        if (!r || r.session_id !== sessionId)
            throw new Error("cannot cancel foreign or missing reservation");
        delete pending[runId];
        saveSpending(root, { ...s, routing_reservations: pending });
        appendJsonl(path.join(root, ".kineti/spend.log.jsonl"), { at: new Date().toISOString(), kind: "reservation-cancelled", run_id: runId, session_id: sessionId, stage: r.stage, reason: "controller ended before attempting turn/start; no inference request submitted" });
    });
}
