import { Toast } from '@base-ui/react';
import clsx from 'clsx';
import { MsIcon } from '../../ui';
import { ToastAddData } from '../../hooks';
import { ToastIcon } from './ToastIcon';

function ToastList() {
  const { toasts } = Toast.useToastManager();

  return (
    <Toast.Viewport className="fixed z-50 top-16 right-0 bottom-auto left-0 mx-auto flex w-full max-w-md">
      {toasts.map((toast) => (
        <Toast.Root
          key={toast.id}
          toast={toast}
          swipeDirection="right"
          className={clsx(
            [
              "[--gap:0.75rem] [--peek:0.75rem] [--scale:calc(max(0,1-(var(--toast-index)*0.1)))] [--shrink:calc(1-var(--scale))] [--height:var(--toast-frontmost-height,var(--toast-height))] [--offset-y:calc(var(--toast-offset-y)+(var(--toast-index)*var(--gap))+var(--toast-swipe-movement-y))] absolute right-0 top-0 left-0 z-[calc(1000-var(--toast-index))] mx-auto max-w-lg origin-top transform-[translateX(var(--toast-swipe-movement-x))_translateY(calc(var(--toast-swipe-movement-y)+(var(--toast-index)*var(--peek))+(var(--shrink)*var(--height))))_scale(var(--scale))] rounded-lg bg-clip-padding p-4 shadow-lg select-none after:absolute after:bottom-full after:left-0 after:h-[calc(var(--gap)+1px)] after:w-full after:content-[''] data-ending-style:opacity-0 data-expanded:transform-[translateX(var(--toast-swipe-movement-x))_translateY(calc(var(--offset-y)))] data-limited:opacity-0 data-starting-style:transform-[translateY(-150%)] [&[data-ending-style]:not([data-limited]):not([data-swipe-direction])]:transform-[translateY(-150%)] data-ending-style:data-[swipe-direction=down]:transform-[translateY(calc(var(--toast-swipe-movement-y)+150%))] data-expanded:data-ending-style:data-[swipe-direction=down]:transform-[translateY(calc(var(--toast-swipe-movement-y)+150%))] data-ending-style:data-[swipe-direction=left]:transform-[translateX(calc(var(--toast-swipe-movement-x)-150%))_translateY(var(--offset-y))] data-expanded:data-ending-style:data-[swipe-direction=left]:transform-[translateX(calc(var(--toast-swipe-movement-x)-150%))_translateY(var(--offset-y))] data-ending-style:data-[swipe-direction=right]:transform-[translateX(calc(var(--toast-swipe-movement-x)+150%))_translateY(var(--offset-y))] data-expanded:data-ending-style:data-[swipe-direction=right]:transform-[translateX(calc(var(--toast-swipe-movement-x)+150%))_translateY(var(--offset-y))] data-ending-style:data-[swipe-direction=up]:transform-[translateY(calc(var(--toast-swipe-movement-y)-150%))] data-expanded:data-ending-style:data-[swipe-direction=up]:transform-[translateY(calc(var(--toast-swipe-movement-y)-150%))] h-(--height) data-expanded:h-(--toast-height) [transition:transform_0.5s_cubic-bezier(0.22,1,0.36,1),opacity_0.5s,height_0.15s]",
              toast.updateKey &&
                (toast.updateKey % 2 === 0
                  ? 'animate-pulse-scale-even'
                  : 'animate-pulse-scale-odd'),
            ],
            {
              'bg-aphrodisiac text-white': toast.type === 'error',
              'bg-charcoal dark:bg-white text-white dark:text-baltic-sea':
                toast.type !== 'error',
            },
          )}
        >
          <Toast.Content className="relative flex flex-row gap-4 items-start justify-start overflow-hidden transition-opacity duration-250 data-behind:pointer-events-none data-behind:opacity-0 data-expanded:pointer-events-auto data-expanded:opacity-100">
            <ToastIcon type={toast.type as ToastAddData['type']} />

            <div className="w-full flex flex-col gap-1 items-start justify-center">
              <Toast.Title className="text-sm leading-5 font-medium" />
              <Toast.Description className="text-xs font-normal leading-5 text-gray-700" />
              <Toast.Action
                className={clsx([
                  'self-end border-[1.5px] font-bold bg-transparent rounded-2xl text-xs py-1.5 px-3',
                  ' hover:bg-white/10 dark:hover:bg-black/10',
                ])}
              />
            </div>
            <Toast.Close
              className={clsx([
                'relative flex items-center justify-center w-6 h-6 p-0 rounded-lg',
                'bg-transparent',
                'hover:bg-iron dark:hover:bg-mercury',
                'text-white dark:text-baltic-sea',
              ])}
              aria-label="Close"
            >
              <MsIcon
                icon="close"
                className="text-xl leading-none font-light"
              />
            </Toast.Close>
          </Toast.Content>
        </Toast.Root>
      ))}
    </Toast.Viewport>
  );
}
export default ToastList;
