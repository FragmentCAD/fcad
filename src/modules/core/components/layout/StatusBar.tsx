import { useTranslation } from "react-i18next";

export function StatusBar() {
  const { t } = useTranslation();

  return (
    <footer className="bg-muted text-muted-foreground flex h-6 items-center justify-between border-t px-3 text-[10px]">
      <div className="flex gap-4">
        <span>{t("ui.statusbar.ready")}</span>
        <span>{t("ui.statusbar.fps")}: 60</span>
      </div>
      <div>
        <span>{t("ui.statusbar.tech_stack")}</span>
      </div>
    </footer>
  );
}
