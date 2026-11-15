import { EmptyView } from "./app/EmptyView";
import { Shell } from "./app/Shell";
import { TopBar } from "./app/TopBar";
import { t } from "./lib/i18n";
import { StatusDot } from "./ui";

export function App() {
  return (
    <Shell
      top={<TopBar />}
      side={null}
      main={<EmptyView />}
      status={
        <div className="flex items-center gap-2">
          <StatusDot tone="faint" />
          <span>{t("statusbar.watcherOff")}</span>
        </div>
      }
    />
  );
}
