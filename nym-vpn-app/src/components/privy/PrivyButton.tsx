// import { openUrl } from "@tauri-apps/plugin-opener";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "../../ui";
import { useInAppNotify } from "../../contexts";
import { useDeepLink } from "../../hooks/useDeepLink";

function PrivyButton() {
  const { t } = useTranslation('login');
  const { push } = useInAppNotify();
  const { startListening } = useDeepLink();

  const [loading, setLoading] = useState(false);

  const handlePrivy = async () => {
    setLoading(true);

    // TODO: Open nym.com login page. Most probably the url will come from backend
    // openUrl('https://nym.com/account/login');

    try {
      const deeplinkurl = await Promise.race([
        startListening(),
        new Promise<never>((_, reject) => setTimeout(() => reject(new Error('Login timeout')), 300000))
      ]);

      console.log("Received deep link: ", deeplinkurl);
      // TODO: Validate deep link and invoke('add_account')
    } catch (error) {
      console.error("Login timeout: ", error);
      push({
        message: "Login timeout",
        type: "error",
        duration: 3000,
        close: true,
      });
    } finally {
      setLoading(false);
    }
  }
  return (
    <Button
      outline
      color="gray"
      onClick={handlePrivy}
      className="group border border-iron dark:border-bombay hover:ring-0! dark:hover:ring-0!"
      spinner={loading}
    >
      <span className="text-black dark:text-white group-hover:text-black/50 dark:group-hover:text-white/80">
        {t('privy.login-button')}
      </span>
    </Button>
  )
}

export default PrivyButton;
