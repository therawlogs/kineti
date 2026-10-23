#!/usr/bin/env bun
/** Read-only model selection using the same classification, access and policy checks as execution. */
import {
  classify,
  hostName,
  loadConfig,
  planRoute,
  type Host,
  type TaskClass,
} from "../src/routing/index.ts";
export type TaskType = TaskClass;
export interface ModelPick {
  task: TaskClass | null;
  host: Host | null;
  model: string | null;
  status: "selected" | "blocked";
  reason: string;
}
export function classifyTask(text: string): TaskClass | null {
  return classify(text).task_class;
}
export function recommend(
  text: string,
  host?: Host,
  root = process.cwd(),
): ModelPick {
  let task: TaskClass | null = null;
  try {
    task = classify(text).task_class;
    if (!host)
      throw new Error(
        "choose OpenCode or Codex and configure its model access and policy",
      );
    const { manifest, policy } = loadConfig(root, host);
    const plan = planRoute(root, host, text, manifest, policy);
    return {
      task: plan.classification.task_class,
      host,
      model: plan.requested_model,
      status: "selected",
      reason: "first eligible model in the owner's permitted order",
    };
  } catch (error) {
    return {
      task,
      host: host ?? null,
      model: null,
      status: "blocked",
      reason: (error as Error).message,
    };
  }
}
if (import.meta.main) {
  const args = process.argv.slice(2);
  if (
    args[0] !== "recommend" ||
    args.length < 2 ||
    args.slice(1).some((arg) => arg.startsWith("-"))
  ) {
    console.error(
      "usage: kineti-models recommend <task words>; use kineti route for configured planning",
    );
    process.exitCode = 2;
  } else {
    const text = args.slice(1).join(" "),
      match = text.match(/\b(opencode|codex)\b/i);
    const result = recommend(
      text,
      match ? hostName(match[1].toLowerCase()) : undefined,
    );
    console.log(JSON.stringify(result));
    if (result.status === "blocked") process.exitCode = 2;
  }
}
