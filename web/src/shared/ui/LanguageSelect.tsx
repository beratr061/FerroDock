
import { useTranslation } from "../i18n";
import { changeLanguage, languages } from "../i18n";
import { SelectField } from "./SelectField";

interface LanguageSelectProps {
  hideLabel?: boolean;
  className?: string;
}

export function LanguageSelect({ hideLabel = false, className }: LanguageSelectProps) {
  const { t, i18n } = useTranslation();
  return (
    <SelectField
      label={t("language")}
      hideLabel={hideLabel}
      className={className}
      value={i18n.language}
      options={languages.map((language) => ({ value: language.code, label: language.name }))}
      onChange={(event) => void changeLanguage(event.target.value)}
    />
  );
}
