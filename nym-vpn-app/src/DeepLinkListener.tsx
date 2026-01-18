import { onOpenUrl } from "@tauri-apps/plugin-deep-link";
import { useEffect } from "react";

let initialized = false;

function DeepLinkListener() {
    useEffect(() => {
        if (initialized) {
            return;
        }
        
        initialized = true;

        (async () => {
            const unlisten = await onOpenUrl((urls) => {
                console.log('deep link:', urls);
            });
            return () => {
                unlisten();
            };
        })()
    }, [])
    return null;
}
export default DeepLinkListener