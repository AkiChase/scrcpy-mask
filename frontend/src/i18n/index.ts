import i18n from "i18next";
import { initReactI18next } from "react-i18next";

export const DEFAULT_LANGUAGE = "en-US";

type TranslationResource = Record<string, unknown> & {
  _meta?: {
    languageName?: unknown;
  };
};

const translationModules = import.meta.glob<TranslationResource>(
  "./*/translation.json",
  {
    eager: true,
    import: "default",
  }
);

export const resources = Object.fromEntries(
  Object.entries(translationModules)
    .map(([path, translation]) => {
      const language = path.match(/^\.\/([^/]+)\/translation\.json$/)?.[1];
      if (!language) {
        throw new Error(`Invalid translation path: ${path}`);
      }
      return [language, { translation }] as const;
    })
    .sort(([a], [b]) => a.localeCompare(b))
);

function languageLabel(language: string) {
  const configuredLabel = resources[language]?.translation._meta?.languageName;
  if (typeof configuredLabel === "string") {
    return configuredLabel;
  }

  const label =
    new Intl.DisplayNames([language], { type: "language" }).of(language) ??
    language;
  return label.slice(0, 1).toLocaleUpperCase(language) + label.slice(1);
}

export const languageOptions = Object.keys(resources).map((language) => ({
  label: languageLabel(language),
  value: language,
}));

i18n.use(initReactI18next).init({
  fallbackLng: DEFAULT_LANGUAGE,
  resources,
  interpolation: {
    escapeValue: false, // not needed for react as it escapes by default
  },
});

export default i18n;
