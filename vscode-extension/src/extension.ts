import * as vscode from "vscode";
import * as http from "http";

const COMMITCAT_PORT = 39547;
const CODING_INTERVAL_MS = 60_000; // 60 seconds

let codingTimer: NodeJS.Timeout | undefined;
let statusBarItem: vscode.StatusBarItem;

let sessionSaveCount = 0;
let sessionCodingMinutes = 0;
const SESSION_COMMAND_ID = "commitcat.showSessionStats";
const RESET_COMMAND_ID = "commitcat.resetSessionStats";
const OPEN_PROFILE_COMMAND_ID = "commitcat.openProfile";
const PROFILE_URL_BASE = "https://commitcat-api.fly.dev/profile/";

function updateStatusBar(): void {
  const parts: string[] = ["$(heart) CommitCat"];
  if (sessionCodingMinutes > 0) {
    parts.push(`${sessionCodingMinutes}m`);
  }
  if (sessionSaveCount > 0) {
    parts.push(`${sessionSaveCount} saves`);
  }
  statusBarItem.text = parts.join(" · ");
}

/** POST JSON to CommitCat's local HTTP server */
function postActivity(body: Record<string, unknown>): void {
  const data = JSON.stringify(body);
  const req = http.request(
    {
      hostname: "127.0.0.1",
      port: COMMITCAT_PORT,
      path: "/activity",
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "Content-Length": Buffer.byteLength(data),
      },
      timeout: 3000,
    },
    (res) => {
      // drain response
      res.resume();
    }
  );
  req.on("error", () => {
    // CommitCat not running — silently ignore
  });
  req.write(data);
  req.end();
}

export function activate(context: vscode.ExtensionContext): void {
  // ── Status bar ──
  statusBarItem = vscode.window.createStatusBarItem(
    vscode.StatusBarAlignment.Right,
    100
  );
  statusBarItem.text = "$(heart) CommitCat";
  statusBarItem.tooltip = "CommitCat is tracking your activity";
  statusBarItem.command = SESSION_COMMAND_ID;
  statusBarItem.show();
  context.subscriptions.push(statusBarItem);

  // ── Session stats command ──
  context.subscriptions.push(
    vscode.commands.registerCommand(SESSION_COMMAND_ID, () => {
      vscode.window.showInformationMessage(
        `CommitCat session: ${sessionCodingMinutes} min coded, ${sessionSaveCount} saves`,
        "Reset"
      ).then((choice) => {
        if (choice === "Reset") {
          sessionSaveCount = 0;
          sessionCodingMinutes = 0;
          updateStatusBar();
        }
      });
    })
  );

  // ── Reset session stats command ──
  context.subscriptions.push(
    vscode.commands.registerCommand(RESET_COMMAND_ID, () => {
      sessionSaveCount = 0;
      sessionCodingMinutes = 0;
      updateStatusBar();
      vscode.window.showInformationMessage("CommitCat: Session stats reset");
    })
  );

  // ── Open public profile command ──
  context.subscriptions.push(
    vscode.commands.registerCommand(OPEN_PROFILE_COMMAND_ID, async () => {
      const username = await vscode.window.showInputBox({
        prompt: "Enter a GitHub username",
        placeHolder: "eunseo9311",
      });
      if (username) {
        vscode.env.openExternal(
          vscode.Uri.parse(PROFILE_URL_BASE + encodeURIComponent(username))
        );
      }
    })
  );

  // ── 1. Coding time: send heartbeat every 60s while editor is focused ──
  codingTimer = setInterval(() => {
    if (vscode.window.state.focused) {
      postActivity({ type: "coding_time", seconds: 60 });
      sessionCodingMinutes += 1;
      updateStatusBar();
    }
  }, CODING_INTERVAL_MS);

  // ── 2. File change: when active editor changes ──
  context.subscriptions.push(
    vscode.window.onDidChangeActiveTextEditor((editor) => {
      if (editor) {
        const doc = editor.document;
        postActivity({
          type: "file_change",
          filename: doc.fileName,
          language: doc.languageId,
        });
      }
    })
  );

  // ── 3. File save ──
  context.subscriptions.push(
    vscode.workspace.onDidSaveTextDocument(() => {
      postActivity({ type: "save" });
      sessionSaveCount += 1;
      updateStatusBar();
    })
  );

  // ── 4. Build task success/fail ──
  context.subscriptions.push(
    vscode.tasks.onDidEndTaskProcess((event) => {
      const task = event.execution.task;
      // Only track build-group tasks or tasks with "build" in their name
      const isBuild =
        task.group === vscode.TaskGroup.Build ||
        task.name.toLowerCase().includes("build");

      if (isBuild) {
        if (event.exitCode === 0) {
          postActivity({ type: "build_success" });
        } else {
          postActivity({ type: "build_fail" });
        }
      }
    })
  );

  // ── Send initial file info if an editor is already open ──
  const activeEditor = vscode.window.activeTextEditor;
  if (activeEditor) {
    postActivity({
      type: "file_change",
      filename: activeEditor.document.fileName,
      language: activeEditor.document.languageId,
    });
  }
}

export function deactivate(): void {
  if (codingTimer) {
    clearInterval(codingTimer);
    codingTimer = undefined;
  }
}
