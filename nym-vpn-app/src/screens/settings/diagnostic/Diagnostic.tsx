import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import clsx from 'clsx';
import { Button, PageAnim } from '../../../ui';

function Diagnostic() {
  const [diagnosticRunning, setDiagnosticRunning] = useState(false);
  const [diagnosticResult, setDiagnosticResult] = useState<string | null>(null);

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
  return (
    <PageAnim className="h-full flex flex-col mt-2 gap-6">
      <Button
        onClick={handleRunDiagnostic}
        disabled={diagnosticRunning}
        spinner={diagnosticRunning}
      >
        Run Diagnostic
      </Button>
      {diagnosticResult && (
        <Button
          outline
          color="gray"
          onClick={() => {
            invoke('share_diagnostic', {
              report: JSON.parse(diagnosticResult),
            });
          }}
        >
          Share report
        </Button>
      )}
      {diagnosticResult && (
        <div className="mt-2 p-3 rounded-lg text-xs font-mono h-full overflow-auto bg-green-900/20 text-green-300 border border-green-800">
          <div className="flex justify-between items-center mb-1">
            <span className="font-semibold text-sm">Diagnostic report</span>
          </div>
          <pre className="whitespace-pre-wrap wrap-break-word">
            {diagnosticResult}
          </pre>
        </div>
      )}
    </PageAnim>
  );
}

export default Diagnostic;
