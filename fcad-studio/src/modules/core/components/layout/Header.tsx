import { Settings, Play, Save } from "lucide-react";
import { Button } from "@/modules/core/components/ui/button";
import { useTranslation } from "react-i18next";
import i18n from "@/modules/core/lib/i18n"; // Optional if we want a lang switcher

export function Header() {
  const { t } = useTranslation();

  return (
    <header className="bg-card flex h-12 items-center justify-between border-b px-4">
      <div className="flex items-center gap-4">
        <span className="text-sm font-bold tracking-widest uppercase">
          {t("ui.app.title")}
        </span>
        <div className="bg-border mx-2 h-4 w-px" />
        <div className="flex gap-1">
          <Button variant="ghost" size="icon" className="h-8 w-8">
            <Play className="h-4 w-4" />
          </Button>
          <Button variant="ghost" size="icon" className="h-8 w-8">
            <Save className="h-4 w-4" />
          </Button>
        </div>
      </div>
      <div className="flex items-center gap-2">
        <Button variant="outline" size="sm" className="h-8">
          {t("ui.header.export_block")}
        </Button>

        {/* Toggle Language Temporary Button */}
        <Button
          variant="ghost"
          size="sm"
          className="h-8 uppercase"
          onClick={() =>
            i18n.changeLanguage(i18n.language === "en" ? "es" : "en")
          }
        >
          {i18n.language}
        </Button>
        <Button variant="ghost" size="icon" className="h-8 w-8">
          <Settings className="h-4 w-4" />
        </Button>
      </div>
    </header>
  );
}
