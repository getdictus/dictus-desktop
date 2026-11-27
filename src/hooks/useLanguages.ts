import { useMemo } from "react";
import { useTranslation } from "react-i18next";
import {
  LANGUAGES,
  capitalizeFirst,
  getLanguageName,
  type Language,
} from "@/lib/constants/languages";

export const useLanguages = (): Language[] => {
  const { t, i18n } = useTranslation();
  const locale = i18n.language;
  return useMemo(
    () =>
      LANGUAGES.map(({ value }) => ({
        value,
        label:
          value === "auto"
            ? t("settings.general.language.auto")
            : capitalizeFirst(getLanguageName(value, locale), locale),
      })),
    [locale, t],
  );
};
