import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Settings } from "lucide-react";
import { useConversion } from "@/hooks/useConversion";
import DropZone from "@/components/DropZone";
import FileList from "@/components/FileList";
import FormatPicker from "@/components/FormatPicker";
import QualitySlider from "@/components/QualitySlider";
import OutputFolderPicker from "@/components/OutputFolderPicker";
import ProgressBar from "@/components/ProgressBar";
import ConvertButton from "@/components/ConvertButton";
import SettingsPage from "@/components/SettingsPage";
import logo from "@/assets/logo.png";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";

function App() {
  const {
    files,
    outputFormats,
    quality,
    outputFolder,
    deleteOriginals,
    isConverting,
    conversionDone,
    globalProgress,
    presentCategories,
    categoryFormats,
    hasLossyFormat,
    convertSingleFile,
    addFiles,
    addFolder,
    removeFile,
    clearFiles,
    setFormat,
    setQuality,
    setOutputFolder,
    setDeleteOriginals,
    startConversion,
    openFile,
    openFileFolder,
    openOutputFolder,
    resetConversion,
  } = useConversion();

  const { t } = useTranslation();
  const hasFiles = files.length > 0;
  const [showSettings, setShowSettings] = useState(false);

  return (
    <div className="h-screen bg-background text-foreground flex flex-col overflow-hidden relative">
      <header
        className="px-5 py-3 flex items-center gap-3 border-b border-border/40"
        data-tauri-drag-region
      >
        <img
          src={logo}
          alt="SuperConvert"
          className="h-7 w-7 opacity-80 drop-shadow-[0_0_6px_rgba(255,255,255,0.15)]"
          draggable={false}
        />
        <div className="flex items-baseline gap-1.5 flex-1" data-tauri-drag-region>
          <h1
            className="text-sm font-semibold tracking-wide text-foreground/90"
            data-tauri-drag-region
          >
            SuperConvert
          </h1>
          <span
            className="text-[9px] tracking-wider text-muted-foreground/40 font-medium"
            data-tauri-drag-region
          >
            by dimzou
          </span>
        </div>
        <Button
          variant="ghost"
          size="icon-xs"
          onClick={() => setShowSettings(!showSettings)}
        >
          <Settings className="size-3.5" />
          <span className="sr-only">{t("settings.title")}</span>
        </Button>
      </header>

      {showSettings ? (
        <SettingsPage onBack={() => setShowSettings(false)} />
      ) : (
        <main className="flex-1 pt-5 px-5 pb-0 flex gap-5 overflow-hidden min-h-0">
          <div className="flex-1 flex flex-col gap-3 min-w-0 min-h-0 pb-5">
            <DropZone
              onFilesAdded={addFiles}
              onFolderAdded={addFolder}
              hasFiles={hasFiles}
              disabled={isConverting}
              presentCategories={presentCategories}
            />
            <FileList
              files={files}
              outputFormats={outputFormats}
              onRemove={removeFile}
              onOpenFile={openFile}
              onOpenFolder={openFileFolder}
              onConvertSingle={convertSingleFile}
              disabled={isConverting}
            />
          </div>

          <div className="w-[280px] flex-shrink-0 flex flex-col min-h-0 relative">
            <div className="flex-1 space-y-3 min-h-0 pb-20 scrollbar-thin">
              {hasFiles && (
                <div className="rounded-xl bg-card/60 border border-border/50 p-4 space-y-4">
                  <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground/60">
                    {t("formatPicker.outputFormat")}
                  </h3>
                  {presentCategories.map((cat) => (
                    <FormatPicker
                      key={cat}
                      category={cat}
                      showLabel={presentCategories.length > 1}
                      formats={categoryFormats[cat] ?? []}
                      selected={outputFormats[cat] ?? ""}
                      onChange={(fmt) => setFormat(cat, fmt)}
                      disabled={isConverting || conversionDone}
                    />
                  ))}
                  {hasLossyFormat && (
                    <QualitySlider
                      quality={quality}
                      onChange={setQuality}
                      disabled={isConverting || conversionDone}
                    />
                  )}
                </div>
              )}

              <div className="rounded-xl bg-card/60 border border-border/50 p-4">
                <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground/60 mb-3">
                  {t("outputFolder.title")}
                </h3>
                <OutputFolderPicker
                  folder={outputFolder}
                  onChange={setOutputFolder}
                  disabled={isConverting || conversionDone}
                />
              </div>

              <label className="flex items-center gap-3 rounded-xl bg-card/60 border border-border/50 px-4 py-3 cursor-pointer select-none transition-colors hover:bg-card/80">
                <div className="flex-1 space-y-0.5">
                  <span className="text-xs font-medium text-foreground/90">{t("deleteOriginals.label")}</span>
                  <p className="text-[10px] leading-tight text-muted-foreground/60">
                    {t("deleteOriginals.description")}
                  </p>
                </div>
                <Switch
                  checked={deleteOriginals}
                  onCheckedChange={setDeleteOriginals}
                  disabled={isConverting || conversionDone}
                />
              </label>
            </div>

            <div className="absolute bottom-0 left-0 right-0 z-10 p-5 pt-8 flex flex-col gap-2.5 pointer-events-none [&_button]:pointer-events-auto">
              <ProgressBar
                progress={globalProgress}
                isConverting={isConverting}
              />
              <ConvertButton
                fileCount={files.length}
                isConverting={isConverting}
                conversionDone={conversionDone}
                onConvert={startConversion}
                onClear={clearFiles}
                onOpenFolder={openOutputFolder}
                onReset={resetConversion}
              />
            </div>
          </div>
        </main>
      )}
    </div>
  );
}

export default App;
