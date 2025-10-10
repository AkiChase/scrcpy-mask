import { useTranslation } from "react-i18next";

export default function Welcome() {
  const { t } = useTranslation();
  return (
    <div className="flex flex-col items-center justify-center h-full">
      <h1 className="text-4xl font-bold">{t("welcome.main")}</h1>
    </div>
  );
}
