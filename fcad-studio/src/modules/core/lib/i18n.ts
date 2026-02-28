import i18n from "i18next";
import { initReactI18next } from "react-i18next";

import esES from "../../../../../fcad-assets/i18n/es-ES.json";
import enUS from "../../../../../fcad-assets/i18n/en-US.json";

const resources = {
  en: { translation: enUS },
  es: { translation: esES },
};

i18n.use(initReactI18next).init({
  resources,
  lng: "es", // Default language
  fallbackLng: "en",
  interpolation: {
    escapeValue: false, // React already escapes values
  },
});

export default i18n;
