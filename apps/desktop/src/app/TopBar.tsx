import { Layers, Moon, Sun } from "lucide-react";
import type { ReactNode } from "react";
import { t } from "../lib/i18n";
import { IconButton } from "../ui";
import { useTheme } from "./theme";

interface TopBarProps {
  /** Project and spec of the open view, if any. */
  crumbs?: { project: string; spec: string } | undefined;
  /** Instance badge (T20) and other actions. */
  actions?: ReactNode;
}

export function TopBar({ crumbs, actions }: TopBarProps) {
  const theme = useTheme((s) => s.theme);
  const toggle = useTheme((s) => s.toggle);
  return (
    <>
      <div className="flex w-60 flex-none items-center gap-2">
        <Layers aria-hidden="true" className="size-[18px] text-acc" />
        <span className="text-sm font-semibold tracking-tight">{t("app.name")}</span>
      </div>
      {crumbs ? (
        <nav aria-label={t("topbar.breadcrumb")} className="flex min-w-0 items-center gap-2 whitespace-nowrap">
          <span className="truncate text-muted">{crumbs.project}</span>
          <span aria-hidden="true" className="text-faint">
            /
          </span>
          <span className="truncate font-medium">{crumbs.spec}</span>
        </nav>
      ) : null}
      <div className="flex-1" />
      {actions}
      <IconButton
        label={theme === "dark" ? t("topbar.themeToLight") : t("topbar.themeToDark")}
        icon={theme === "dark" ? <Sun className="size-4" /> : <Moon className="size-4" />}
        onClick={toggle}
      />
    </>
  );
}
