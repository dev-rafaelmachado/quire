import type { Lesson, LessonsView } from "../../lib/ipc";
import { t } from "../../lib/i18n";
import { Chip, type Tone } from "../../ui";
import { SkippedNotice } from "./GatesPanel";

function signalTone(signal: string | null): Tone {
  if (!signal) return "neutral";
  if (/fail/.test(signal)) return "warn";
  if (/gap/.test(signal) && signal.startsWith("ac")) return "acc";
  return "neutral";
}

function LessonCard({ lesson }: { lesson: Lesson }) {
  return (
    <article className="flex flex-col gap-2.5 rounded-[var(--radius-lg)] border border-line bg-panel p-3.5">
      <div className="flex flex-wrap items-center gap-2">
        {lesson.signal ? (
          <span aria-label={t("logs.signal", { signal: lesson.signal })}>
            <Chip mono tone={signalTone(lesson.signal)}>
              {lesson.signal}
            </Chip>
          </span>
        ) : null}
        {lesson.scope ? <span className="font-code text-[11px] text-faint">{lesson.scope}</span> : null}
      </div>
      {lesson.text ? <p className="m-0 text-[13px] leading-relaxed">{lesson.text}</p> : null}
      {lesson.source ? (
        <span className="font-code text-xs text-acc" title={t("logs.source", { source: lesson.source })}>
          {lesson.source}
        </span>
      ) : null}
    </article>
  );
}

/** Lessons as cards: signal, scope, text and source (LOG-03). */
export function LessonCards({ lessons }: { lessons: readonly Lesson[] }) {
  if (lessons.length === 0) return <p className="m-0 text-muted">{t("logs.noLessons")}</p>;
  return (
    <div className="flex flex-col gap-2.5">
      {lessons.map((l, i) => (
        <LessonCard key={i} lesson={l} />
      ))}
    </div>
  );
}

/** Center of the lessons tab. */
export function LessonsPanel({ lessons }: { lessons: LessonsView }) {
  return (
    <>
      <SkippedNotice file="lessons.jsonl" skipped={lessons.skipped} lines={lessons.skippedLines} />
      <div className="min-h-0 flex-1 overflow-y-auto">
        <div className="max-w-[736px] px-10 pt-6 pb-24">
          <LessonCards lessons={lessons.lessons} />
        </div>
      </div>
    </>
  );
}

/** Right column of the gates tab (wireframe 3). */
export function LessonsAside({ lessons, onOpen }: { lessons: LessonsView; onOpen: () => void }) {
  return (
    <section aria-label={t("logs.lessons")} className="flex min-h-0 flex-1 flex-col">
      <div className="flex flex-none items-center gap-2 px-4 pt-4 pb-3">
        <h2 className="m-0 text-[11px] font-semibold tracking-[.06em] text-faint uppercase">{t("logs.lessons")}</h2>
        <Chip>{lessons.lessons.length}</Chip>
        <div className="flex-1" />
        <button type="button" onClick={onOpen} aria-label={t("logs.openLessons")} className="font-code text-xs text-acc hover:underline">
          {t("logs.lessonsFile")}
        </button>
      </div>
      <div className="min-h-0 flex-1 overflow-y-auto px-4 pb-4">
        <LessonCards lessons={lessons.lessons} />
      </div>
    </section>
  );
}
