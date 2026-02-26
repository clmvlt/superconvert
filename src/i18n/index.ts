import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import LanguageDetector from "i18next-browser-languagedetector";
import en from "./locales/en/translation.json";
import fr from "./locales/fr/translation.json";
import zh from "./locales/zh/translation.json";
import es from "./locales/es/translation.json";
import ar from "./locales/ar/translation.json";
import hi from "./locales/hi/translation.json";
import pt from "./locales/pt/translation.json";
import ru from "./locales/ru/translation.json";
import ja from "./locales/ja/translation.json";
import ko from "./locales/ko/translation.json";
import de from "./locales/de/translation.json";
import it from "./locales/it/translation.json";
import tr from "./locales/tr/translation.json";

i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    resources: {
      en: { translation: en },
      fr: { translation: fr },
      zh: { translation: zh },
      es: { translation: es },
      ar: { translation: ar },
      hi: { translation: hi },
      pt: { translation: pt },
      ru: { translation: ru },
      ja: { translation: ja },
      ko: { translation: ko },
      de: { translation: de },
      it: { translation: it },
      tr: { translation: tr },
    },
    fallbackLng: "en",
    interpolation: {
      escapeValue: false,
    },
    detection: {
      order: ["localStorage", "navigator"],
      caches: ["localStorage"],
    },
  });

export default i18n;
