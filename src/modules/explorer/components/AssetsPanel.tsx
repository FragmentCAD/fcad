import { Box } from "lucide-react";
import { useTranslation } from "react-i18next";

export function AssetsPanel() {
  const { t } = useTranslation();

  return (
    <>
      <div className="text-muted-foreground flex items-center gap-2 border-t p-3 text-xs font-semibold tracking-wider uppercase">
        <Box className="h-3 w-3" /> {t("ui.panels.assets")}
      </div>
      <div className="text-muted-foreground h-32 p-2 text-center text-xs italic">
        {t("ui.panels.assets_empty")}
      </div>
    </>
  );
}
