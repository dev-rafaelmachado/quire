import { CircleCheck, Plus, TriangleAlert, X } from "lucide-react";
import { type FormEvent, useState } from "react";
import { useTheme } from "../../app/theme";
import { isLocale, t } from "../../lib/i18n";
import { errorText } from "../../lib/i18n/errors";
import { asCmdError, type CmdError, commands, type ConfigView, type ProjectSummary } from "../../lib/ipc";
import { Button, Chip, ErrorDetail, ErrorText, IconButton, Segmented } from "../../ui";
import { useWorkspace } from "../workspace/store";

interface ProjectDraft {
  name: string;
  preset: string;
  alias: string;
  issueUrlBase: string;
}

function draftProjects(config: ConfigView, projects: readonly ProjectSummary[]): ProjectDraft[] {
  const names = [...new Set([...projects.map((p) => p.name), ...config.projects.map((p) => p.name)])].sort();
  return names.map((name) => {
    const c = config.projects.find((p) => p.name === name);
    return { name, preset: c?.preset ?? "", alias: c?.alias ?? "", issueUrlBase: c?.issueUrlBase ?? "" };
  });
}

function Section({ title, hint, children }: { title: string; hint?: string; children: React.ReactNode }) {
  return (
    <section className="flex flex-col gap-3">
      <div>
        <h2 className="m-0 text-[15px] font-semibold">{title}</h2>
        {hint ? <p className="m-0 mt-0.5 text-[12.5px] text-muted">{hint}</p> : null}
      </div>
      {children}
    </section>
  );
}

const input =
  "h-[30px] min-w-0 rounded-[var(--radius-md)] border border-line bg-bg px-2.5 text-[12.5px] text-text outline-none focus-visible:border-acc disabled:opacity-60";

/** Preferences (T23): scan paths with status, per-project preset/alias/tracker, polling and
 *  theme. Saving is the only action that writes config.toml. */
export function Preferences({ config }: { config: ConfigView }) {
  const projects = useWorkspace((s) => s.projects);
  const setConfig = useWorkspace((s) => s.setConfig);
  const close = useWorkspace((s) => s.setPrefsOpen);
  const setTheme = useTheme((s) => s.setTheme);
  const [paths, setPaths] = useState<string[]>(config.scanPaths);
  const [newPath, setNewPath] = useState("");
  const [interval, setIntervalSeconds] = useState(String(config.pollIntervalMs / 1000));
  const [theme, setThemeDraft] = useState<"dark" | "light">(config.theme === "light" ? "light" : "dark");
  const [language, setLanguage] = useState<string | null>(config.language);
  const [rows, setRows] = useState<ProjectDraft[]>(() => draftProjects(config, projects));
  const [state, setState] = useState<{ kind: "idle" | "saving" | "saved" } | { kind: "error"; error: CmdError }>({ kind: "idle" });
  const editable = config.editable;
  const reason = config.error ? errorText(config.error) : t("prefs.noConfigDir");

  const statusOf = (path: string) => config.scanPathStatus.find((s) => s.path === path);
  const update = (name: string, patch: Partial<ProjectDraft>) => setRows((r) => r.map((x) => (x.name === name ? { ...x, ...patch } : x)));

  const add = (e: FormEvent) => {
    e.preventDefault();
    const p = newPath.trim();
    if (p && !paths.includes(p)) setPaths([...paths, p]);
    setNewPath("");
  };

  const save = async () => {
    setState({ kind: "saving" });
    try {
      const next = await commands.setConfig({
        scanPaths: paths,
        pollIntervalMs: Math.round(Number(interval) * 1000),
        theme,
        language,
        projects: rows.map((r) => ({
          name: r.name,
          preset: r.preset || null,
          alias: r.alias || null,
          issueUrlBase: r.issueUrlBase || null,
        })),
      });
      setConfig(next);
      setPaths(next.scanPaths);
      setTheme(theme);
      setState({ kind: "saved" });
    } catch (e) {
      setState({ kind: "error", error: asCmdError(e) });
    }
  };

  return (
    <div className="min-h-0 flex-1 overflow-y-auto">
      <div className="flex max-w-[760px] flex-col gap-8 px-10 pt-8 pb-24">
        <div className="flex items-start gap-3">
          <div className="flex-1">
            <h1 className="m-0 text-[22px] font-semibold tracking-tight">{t("prefs.title")}</h1>
            {config.path ? <p className="m-0 mt-1 font-code text-xs text-faint">{t("prefs.file", { path: config.path })}</p> : null}
          </div>
          <IconButton label={t("prefs.close")} icon={<X className="size-4" />} onClick={() => close(false)} />
        </div>
        {!editable ? (
          <p role="alert" className="m-0 rounded-[var(--radius-md)] bg-warn-bg px-3 py-2 text-[12.5px] text-warn">
            {t("prefs.readOnly", { reason })}
            {config.error ? <ErrorDetail error={config.error} /> : null}
          </p>
        ) : null}

        <Section title={t("prefs.scanPaths")} hint={t("prefs.scanPathsHint")}>
          <ul className="m-0 flex list-none flex-col gap-1.5 p-0">
            {paths.map((p) => {
              const status = statusOf(p);
              return (
                <li key={p} className="flex items-center gap-2 rounded-[var(--radius-md)] border border-line bg-panel px-3 py-2">
                  <span className="min-w-0 flex-1 truncate font-code text-xs" title={p}>
                    {p}
                  </span>
                  {status === undefined ? (
                    <Chip>{t("prefs.pathUnsaved")}</Chip>
                  ) : status.error ? (
                    <Chip tone="warn" icon={<TriangleAlert aria-hidden="true" className="size-3" />}>
                      {errorText(status.error)}
                    </Chip>
                  ) : (
                    <Chip tone="ok" icon={<CircleCheck aria-hidden="true" className="size-3" />}>
                      {t("prefs.pathOk")}
                    </Chip>
                  )}
                  <IconButton label={t("prefs.removePath", { path: p })} icon={<X className="size-3.5" />} onClick={() => setPaths(paths.filter((x) => x !== p))} disabled={!editable} className="size-6" />
                </li>
              );
            })}
          </ul>
          <form onSubmit={add} className="flex gap-2">
            <input
              aria-label={t("prefs.addPathLabel")}
              placeholder={t("prefs.addPathPlaceholder")}
              value={newPath}
              onChange={(e) => setNewPath(e.target.value)}
              disabled={!editable}
              className={`${input} flex-1 font-code`}
            />
            <Button type="submit" icon={<Plus aria-hidden="true" className="size-3.5" />} disabled={!editable || newPath.trim() === ""}>
              {t("prefs.addPath")}
            </Button>
          </form>
        </Section>

        <Section title={t("prefs.projects")} hint={t("prefs.projectsHint")}>
          {rows.length === 0 ? <p className="m-0 text-[12.5px] text-muted">{t("prefs.noProjects")}</p> : null}
          {rows.length > 0 ? (
            <table className="w-full border-collapse text-[12.5px]">
              <thead>
                <tr className="text-left text-[11px] font-semibold tracking-[.06em] text-faint uppercase">
                  <th className="pb-2 font-semibold">{t("prefs.project")}</th>
                  <th className="pb-2 font-semibold">{t("prefs.preset")}</th>
                  <th className="pb-2 font-semibold">{t("prefs.alias")}</th>
                  <th className="pb-2 font-semibold">{t("prefs.trackerUrl")}</th>
                </tr>
              </thead>
              <tbody>
                {rows.map((r) => (
                  <tr key={r.name} className="border-t border-line">
                    <td className="py-2 pr-3 font-medium">{r.name}</td>
                    <td className="py-2 pr-3">
                      <select aria-label={t("prefs.presetFor", { name: r.name })} value={r.preset} onChange={(e) => update(r.name, { preset: e.target.value })} disabled={!editable} className={input}>
                        <option value="">{t("prefs.presetDefault")}</option>
                        {config.presets.map((p) => (
                          <option key={p} value={p}>
                            {p}
                          </option>
                        ))}
                      </select>
                      {(() => {
                        const suggested = projects.find((p) => p.name === r.name)?.suggestedPreset;
                        return suggested && r.preset === "" ? (
                          <button
                            type="button"
                            disabled={!editable}
                            onClick={() => update(r.name, { preset: suggested })}
                            className="mt-1 block text-[11.5px] text-acc underline decoration-dotted disabled:opacity-60"
                          >
                            {t("prefs.suggested", { preset: suggested })}
                          </button>
                        ) : null;
                      })()}
                    </td>
                    <td className="py-2 pr-3">
                      <input aria-label={t("prefs.aliasFor", { name: r.name })} value={r.alias} onChange={(e) => update(r.name, { alias: e.target.value })} disabled={!editable} className={`${input} w-28`} />
                    </td>
                    <td className="py-2">
                      <input
                        aria-label={t("prefs.trackerFor", { name: r.name })}
                        placeholder={t("prefs.trackerPlaceholder")}
                        value={r.issueUrlBase}
                        onChange={(e) => update(r.name, { issueUrlBase: e.target.value })}
                        disabled={!editable}
                        className={`${input} w-full font-code`}
                      />
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          ) : null}
        </Section>

        <Section title={t("prefs.watching")} hint={t("prefs.pollHint")}>
          <label className="flex items-center gap-3 text-[12.5px]">
            <span className="w-56">{t("prefs.pollInterval")}</span>
            <input type="number" min={0.5} max={60} step={0.5} value={interval} onChange={(e) => setIntervalSeconds(e.target.value)} disabled={!editable} className={`${input} w-24`} />
          </label>
        </Section>

        <Section title={t("prefs.appearance")}>
          <div className="flex items-center gap-3 text-[12.5px]">
            <span aria-hidden="true" className="w-56">
              {t("prefs.theme")}
            </span>
            <Segmented
              label={t("prefs.theme")}
              value={theme}
              onChange={setThemeDraft}
              options={[
                { value: "dark", label: t("prefs.themeDark") },
                { value: "light", label: t("prefs.themeLight") },
              ]}
            />
          </div>
          <div className="flex items-center gap-3 text-[12.5px]">
            <span aria-hidden="true" className="w-56">
              {t("prefs.language")}
            </span>
            <Segmented
              label={t("prefs.language")}
              value={isLocale(language) ? language : "system"}
              onChange={(v) => setLanguage(v === "system" ? null : v)}
              options={[
                { value: "system", label: t("prefs.languageSystem") },
                { value: "pt-BR", label: t("prefs.languagePt") },
                { value: "en", label: t("prefs.languageEn") },
              ]}
            />
          </div>
          {language !== null && !isLocale(language) ? (
            <p role="status" className="m-0 text-[12px] text-warn">
              {t("prefs.languageUnknown", { value: language })}
            </p>
          ) : null}
        </Section>

        <div className="flex items-center gap-3 border-t border-line pt-5">
          <Button onClick={() => void save()} disabled={!editable || state.kind === "saving"}>
            {state.kind === "saving" ? t("prefs.saving") : t("prefs.save")}
          </Button>
          <Button variant="ghost" onClick={() => close(false)}>
            {t("prefs.close")}
          </Button>
          {state.kind === "saved" ? (
            <span role="status" className="text-[12.5px] text-ok">
              {t("prefs.saved")}
            </span>
          ) : null}
          {state.kind === "error" ? (
            <span role="alert" className="text-[12.5px] text-warn">
              <ErrorText error={state.error} />
            </span>
          ) : null}
        </div>
      </div>
    </div>
  );
}
