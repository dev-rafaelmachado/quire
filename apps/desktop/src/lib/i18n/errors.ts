import type { CmdError } from "../ipc";
import { type MessageKey, t } from "./index";
import { ptBR } from "./pt-BR";

/** Translates a backend error by its stable code (DEB-02). The backend text, in English, is
 *  never the message; it is only offered as detail. */
export function errorText(error: Pick<CmdError, "code">): string {
  const key = `errors.${error.code}`;
  return error.code in ptBR.errors ? t(key as MessageKey) : t("errors.unknown");
}
