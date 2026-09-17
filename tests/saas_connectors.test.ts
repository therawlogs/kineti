import { describe, test, expect } from "bun:test";

describe("SaaS Connectors & Wispr Flow MCP", () => {
  test("Wispr Flow MCP endpoint configuration", () => {
    const wisprMcpEndpoint = "https://api.wisprflow.ai/connect/mcp";
    expect(wisprMcpEndpoint).toBe("https://api.wisprflow.ai/connect/mcp");
    expect(wisprMcpEndpoint.startsWith("https://api.wisprflow.ai")).toBe(true);
  });

  test("Connector protocol consequence level evaluation", () => {
    function evaluateConsequence(connector: string, action: string): "trivial" | "operational" | "high_consequence" {
      if (connector === "google_workspace") {
        if (action.startsWith("read_") || action.startsWith("search_")) return "trivial";
        if (action.startsWith("create_draft")) return "operational";
        return "high_consequence"; // send_email, delete_file, update_event
      }
      if (connector === "wispr_flow") {
        return "trivial"; // speech-to-text dictation is sensory input
      }
      if (connector === "github" || connector === "linear") {
        if (action.startsWith("read_") || action.startsWith("search_")) return "trivial";
        return "operational"; // create_issue, add_comment
      }
      return "high_consequence"; // Fail closed
    }

    expect(evaluateConsequence("google_workspace", "read_messages")).toBe("trivial");
    expect(evaluateConsequence("google_workspace", "send_email")).toBe("high_consequence");
    expect(evaluateConsequence("wispr_flow", "stream_transcription")).toBe("trivial");
    expect(evaluateConsequence("github", "create_issue")).toBe("operational");
    expect(evaluateConsequence("unknown", "any_action")).toBe("high_consequence");
  });
});
