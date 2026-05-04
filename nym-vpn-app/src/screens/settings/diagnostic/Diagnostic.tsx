import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useTranslation } from 'react-i18next';
import { Separator } from '@base-ui-components/react';
import {
  Button,
  ButtonIcon,
  ButtonIconNew,
  ButtonNew,
  PageAnim,
} from '../../../ui';
import { useClipboard } from '../../../hooks';

function Diagnostic() {
  const { t } = useTranslation('settings');
  const { copy } = useClipboard();

  const [diagnosticRunning, setDiagnosticRunning] = useState(false);
  const [diagnosticResult, setDiagnosticResult] = useState<string | null>(null);
  const [shareLoading, setShareLoading] = useState(false);

  const handleRunDiagnostic = async () => {
    setDiagnosticRunning(true);
    try {
      const report = await invoke('run_diagnostic', {
        params: { gateway: null, skipDns: false, skipHttp: false },
      });
      setDiagnosticResult(JSON.stringify(report, null, 2));
    } finally {
      setDiagnosticRunning(false);
    }
  };

  const handleShareReport = async () => {
    if (!diagnosticResult) return;

    setShareLoading(true);
    try {
      await invoke('share_diagnostic', {
        report: JSON.parse(diagnosticResult),
      });
    } catch (error) {
      console.error('failed to share diagnostic report', error);
    } finally {
      setShareLoading(false);
    }
  };

  return (
    <PageAnim className="h-full flex flex-col">
      <div className="flex gap-6 flex-col min-h-0 flex-1">
        <ButtonNew
          onClick={handleRunDiagnostic}
          disabled={diagnosticRunning}
          loading={diagnosticRunning}
        >
          {t('diagnostic.run')}
        </ButtonNew>
        {diagnosticResult && (
          <>
            <ButtonNew
              variant="outlined"
              onClick={handleShareReport}
              disabled={shareLoading}
              loading={shareLoading}
            >
              {t('diagnostic.share')}
            </ButtonNew>
            <div className="space-y-4 p-6 rounded-lg min-h-0 flex flex-col flex-1 text-xs font-mono dark:bg-charcoal bg-white">
              <div className="flex justify-between items-center">
                <span className="font-semibold text-sm">
                  {t('diagnostic.report-title')}
                </span>
                <ButtonIconNew
                  className="self-start"
                  icon="content_copy"
                  onClick={() => copy(diagnosticResult, false)}
                  clickFeedback
                  noDefaultSize
                />
              </div>
              <Separator
                orientation="horizontal"
                className="w-full h-px bg-bombay dark:bg-iron"
              />
              <div className="overflow-y-auto flex-1 min-h-0">
                <pre className="whitespace-pre-wrap wrap-break-word text-iron dark:text-bombay">
                  {diagnosticResult}
                </pre>
              </div>
            </div>
          </>
        )}
      </div>
    </PageAnim>
  );
}

export default Diagnostic;
