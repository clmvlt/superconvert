import { useConversion } from "@/hooks/useConversion";
import DropZone from "@/components/DropZone";
import FileList from "@/components/FileList";
import FormatPicker from "@/components/FormatPicker";
import QualitySlider from "@/components/QualitySlider";
import OutputFolderPicker from "@/components/OutputFolderPicker";
import ProgressBar from "@/components/ProgressBar";
import ConvertButton from "@/components/ConvertButton";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { TooltipProvider } from "@/components/ui/tooltip";
import logo from "@/assets/logo.png";

function App() {
  const {
    files,
    outputFormat,
    quality,
    outputFolder,
    isConverting,
    conversionDone,
    globalProgress,
    detectedCategory,
    availableFormats,
    addFiles,
    addFolder,
    removeFile,
    clearFiles,
    setFormat,
    setQuality,
    setOutputFolder,
    startConversion,
    openFile,
    openOutputFolder,
    resetConversion,
  } = useConversion();

  return (
    <TooltipProvider>
      <div className="min-h-screen bg-background text-foreground flex flex-col">
        <header className="px-6 py-3 border-b border-border flex items-center gap-3">
          <img src={logo} alt="Convertor" className="w-8 h-8" />
          <div>
            <h1 className="text-lg font-bold tracking-tight">Convertor</h1>
            <p className="text-xs text-muted-foreground">Fast file converter</p>
          </div>
        </header>

        <main className="flex-1 p-6 flex gap-6 overflow-hidden">
          <div className="flex-1 flex flex-col gap-4 min-w-0">
            <DropZone
              onFilesAdded={addFiles}
              onFolderAdded={addFolder}
              hasFiles={files.length > 0}
              disabled={isConverting}
              detectedCategory={detectedCategory}
            />
            <FileList
              files={files}
              outputFormat={outputFormat}
              onRemove={removeFile}
              onOpenFile={openFile}
              disabled={isConverting}
            />
          </div>

          <div className="w-72 flex-shrink-0 flex flex-col gap-4">
            <Card className="py-4">
              <CardHeader className="px-4 py-0">
                <CardTitle className="text-sm">Output format</CardTitle>
              </CardHeader>
              <CardContent className="px-4 space-y-4">
                <FormatPicker
                  formats={availableFormats}
                  selected={outputFormat}
                  onChange={setFormat}
                  disabled={isConverting || conversionDone}
                />
                <QualitySlider
                  format={outputFormat}
                  quality={quality}
                  onChange={setQuality}
                  disabled={isConverting || conversionDone}
                />
              </CardContent>
            </Card>

            <Card className="py-4">
              <CardHeader className="px-4 py-0">
                <CardTitle className="text-sm">Output folder</CardTitle>
              </CardHeader>
              <CardContent className="px-4">
                <OutputFolderPicker
                  folder={outputFolder}
                  onChange={setOutputFolder}
                  disabled={isConverting || conversionDone}
                />
              </CardContent>
            </Card>

            <div className="mt-auto flex flex-col gap-3">
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
      </div>
    </TooltipProvider>
  );
}

export default App;
