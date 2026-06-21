import {
  assertNever,
  type ConnectionStatus,
  type Diagnostic,
  type WorkspaceSnapshot,
} from "../expected/complex-api";

export function renderConnectionStatus(status: ConnectionStatus): string {
  switch (status) {
    case "connected":
      return "connected";
    case "idle":
      return "idle";
    case "error":
      return "error";
    case "sshTunnelDown":
      return "ssh tunnel down";
    default:
      return assertNever(status);
  }
}

export function collectFatalDiagnostics(snapshot: WorkspaceSnapshot): Diagnostic[] {
  return snapshot.connections.flatMap((connection) =>
    connection.warnings.filter((diagnostic) => diagnostic.severity === "fatal"),
  );
}

// This is intentionally narrow: JsonValue is unknown, so the consumer must narrow.
export function safeReadMessage(value: unknown): string | undefined {
  if (typeof value === "object" && value !== null && "message" in value) {
    const message = (value as { message?: unknown }).message;
    return typeof message === "string" ? message : undefined;
  }
  return undefined;
}
