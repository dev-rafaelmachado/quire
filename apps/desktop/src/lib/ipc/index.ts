// IPC layer: the only way the UI talks to the core (AGENTS.md). Everything here is typed by
// the bindings that tauri-specta generates from the Rust commands (TD-07); this file only
// gives the received shapes short names.
import type {
  CmdError,
  ConfigView_Serialize,
  FileContent_Serialize,
  FileEntry_Serialize,
  GateRun_Serialize,
  GatesView_Serialize,
  HeaderField_Serialize,
  InstanceSummary_Serialize,
  Lesson_Serialize,
  LessonsView_Serialize,
  PhaseView_Serialize,
  ProjectSummary_Serialize,
  ScanProgress_Serialize,
  SpecDetail_Serialize,
  SpecSummary_Serialize,
  TaskDetail,
  Task_Serialize,
  TasksView_Serialize,
} from "./bindings";

export { commands, events } from "./bindings";
export type { CmdError, FileKind, InstanceKind, ListFilter, PhaseState, ScanPhase, WatchStatus } from "./bindings";

export type ConfigView = ConfigView_Serialize;
export type ProjectSummary = ProjectSummary_Serialize;
export type SpecSummary = SpecSummary_Serialize;
export type InstanceSummary = InstanceSummary_Serialize;
export type SpecDetail = SpecDetail_Serialize;
export type FileEntry = FileEntry_Serialize;
export type FileContent = FileContent_Serialize;
export type HeaderField = HeaderField_Serialize;
export type TasksView = TasksView_Serialize;
export type PhaseView = PhaseView_Serialize;
export type Task = Task_Serialize;
export type { TaskDetail };
export type GatesView = GatesView_Serialize;
export type GateRun = GateRun_Serialize;
export type LessonsView = LessonsView_Serialize;
export type Lesson = Lesson_Serialize;
export type ScanProgress = ScanProgress_Serialize;

/** Narrows a rejected command promise to the typed error the Rust side returns. */
export function asCmdError(e: unknown): CmdError {
  if (typeof e === "object" && e !== null && "code" in e && "message" in e) {
    const { code, message } = e as Record<"code" | "message", unknown>;
    if (typeof code === "string" && typeof message === "string") {
      return { code, message };
    }
  }
  return { code: "unknown", message: String(e) };
}
