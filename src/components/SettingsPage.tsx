import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { ArrowLeft, Sun, Moon, Monitor, Check } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import { useTheme } from "@/components/theme-provider";
import {
  isContextMenuRegistered,
  registerContextMenu,
  unregisterContextMenu,
} from "@/lib/tauri-bridge";

const LANGUAGES = [
  { code: "en", label: "English" },
  { code: "fr", label: "Français" },
  { code: "zh", label: "中文" },
  { code: "es", label: "Español" },
  { code: "ar", label: "العربية" },
  { code: "hi", label: "हिन्दी" },
  { code: "pt", label: "Português" },
  { code: "ru", label: "Русский" },
  { code: "ja", label: "日本語" },
  { code: "ko", label: "한국어" },
  { code: "de", label: "Deutsch" },
  { code: "it", label: "Italiano" },
  { code: "tr", label: "Türkçe" },
] as const;

const THEMES = [
  { value: "light" as const, icon: Sun, labelKey: "theme.light" },
  { value: "dark" as const, icon: Moon, labelKey: "theme.dark" },
  { value: "system" as const, icon: Monitor, labelKey: "theme.system" },
];

interface SettingsPageProps {
  onBack: () => void;
}

export default function SettingsPage({ onBack }: SettingsPageProps) {
  const { t, i18n } = useTranslation();
  const { theme, setTheme } = useTheme();

  const [contextMenuEnabled, setContextMenuEnabled] = useState(false);
  const [contextMenuLoading, setContextMenuLoading] = useState(false);

  useEffect(() => {
    isContextMenuRegistered()
      .then(setContextMenuEnabled)
      .catch(() => {});
  }, []);

  const toggleContextMenu = useCallback(async (checked: boolean) => {
    setContextMenuLoading(true);
    try {
      if (checked) {
        await registerContextMenu();
      } else {
        await unregisterContextMenu();
      }
      setContextMenuEnabled(checked);
    } catch (err) {
      console.error("Context menu toggle failed:", err);
      setContextMenuEnabled(!checked);
    } finally {
      setContextMenuLoading(false);
    }
  }, []);

  return (
    <div className="flex-1 flex flex-col overflow-hidden min-h-0">
      <div className="px-5 pt-4 pb-3 flex items-center gap-3">
        <Button variant="ghost" size="icon-xs" onClick={onBack}>
          <ArrowLeft className="size-4" />
        </Button>
        <h2 className="text-sm font-semibold tracking-wide text-foreground/90">
          {t("settings.title")}
        </h2>
      </div>

      <div className="flex-1 overflow-y-auto px-5 pb-5 scrollbar-thin">
        <div className="max-w-md mx-auto space-y-5">

          {/* Language */}
          <section className="rounded-xl bg-card/60 border border-border/50 p-4">
            <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground/60 mb-3">
              {t("settings.language")}
            </h3>
            <div className="grid grid-cols-3 gap-1.5">
              {LANGUAGES.map((lang) => {
                const active = i18n.resolvedLanguage === lang.code;
                return (
                  <button
                    key={lang.code}
                    onClick={() => i18n.changeLanguage(lang.code)}
                    className={`relative flex items-center justify-center gap-1.5 rounded-lg px-2 py-1.5 text-xs font-medium transition-colors cursor-pointer ${
                      active
                        ? "bg-primary text-primary-foreground"
                        : "bg-muted/40 text-foreground/70 hover:bg-muted/80"
                    }`}
                  >
                    {lang.label}
                    {active && <Check className="size-3" />}
                  </button>
                );
              })}
            </div>
          </section>

          {/* Theme */}
          <section className="rounded-xl bg-card/60 border border-border/50 p-4">
            <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground/60 mb-3">
              {t("settings.theme")}
            </h3>
            <div className="flex gap-2">
              {THEMES.map(({ value, icon: Icon, labelKey }) => {
                const active = theme === value;
                return (
                  <button
                    key={value}
                    onClick={() => setTheme(value)}
                    className={`flex-1 flex items-center justify-center gap-2 rounded-lg px-3 py-2 text-xs font-medium transition-colors cursor-pointer ${
                      active
                        ? "bg-primary text-primary-foreground"
                        : "bg-muted/40 text-foreground/70 hover:bg-muted/80"
                    }`}
                  >
                    <Icon className="size-3.5" />
                    {t(labelKey)}
                  </button>
                );
              })}
            </div>
          </section>

          {/* Context Menu */}
          <section className="rounded-xl bg-card/60 border border-border/50 p-4">
            <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground/60 mb-3">
              {t("settings.integration")}
            </h3>
            <label className="flex items-center gap-3 cursor-pointer select-none">
              <div className="flex-1 space-y-0.5">
                <span className="text-xs font-medium text-foreground/90">
                  {t("contextMenu.label")}
                </span>
                <p className="text-[10px] leading-tight text-muted-foreground/60">
                  {contextMenuLoading
                    ? contextMenuEnabled
                      ? t("contextMenu.uninstalling")
                      : t("contextMenu.installing")
                    : t("contextMenu.description")}
                </p>
              </div>
              <Switch
                checked={contextMenuEnabled}
                onCheckedChange={toggleContextMenu}
                disabled={contextMenuLoading}
              />
            </label>
          </section>

        </div>
      </div>
    </div>
  );
}
