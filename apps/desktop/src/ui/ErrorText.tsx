import { errorText } from "../lib/i18n/errors";
import { t } from "../lib/i18n";
import type { CmdError } from "../lib/ipc";

/** The backend text of an error (English), folded away: it is detail, never the message. */
export function ErrorDetail({ error }: { error: CmdError }) {
  if (!error.message) return null;
  return (
    <details className="inline-block align-top">
      <summary className="ml-1 cursor-pointer text-faint underline decoration-dotted">{t("errors.details")}</summary>
      <span className="block font-code text-[11px] break-all text-faint">{error.message}</span>
    </details>
  );
}

/** A translated backend error with the backend detail folded away (DEB-02). */
export function ErrorText({ error, className }: { error: CmdError; className?: string }) {
  return (
    <span className={className}>
      {errorText(error)}
      <ErrorDetail error={error} />
    </span>
  );
}
