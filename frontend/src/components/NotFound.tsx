import { Button, Flex, Result } from "antd";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";

export default function NotFound() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  return (
    <Flex justify="center" align="center" className="h-full w-full">
      <Result
        status="404"
        title="404"
        subTitle={t("404.title")}
        extra={
          <Button
            type="primary"
            onClick={() => navigate("/", { replace: true })}
          >
            {t("404.back")}
          </Button>
        }
      />
    </Flex>
  );
}
