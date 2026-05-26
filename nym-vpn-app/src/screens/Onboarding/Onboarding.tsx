import clsx from 'clsx';
import { Button as HuButton } from '@headlessui/react';
import useEmblaCarousel from 'embla-carousel-react';
import { useTranslation } from 'react-i18next';
import { useAnimatedNavigate } from '../../hooks/useAnimatedNavigate';
import { Button, ButtonIconNew, MsIcon } from '../../ui';
import { NymVpnTextLogo } from '../../assets';
import { routes } from '../../router';
import { InteractiveCard } from '../home/InteractiveCard';
import { useAppStore } from '../../store/index';
import { Speed, Tracking, Welcome, ZeroKnowledge } from './slides';
import { DotButton, useDotButton } from './CarouselDotButton';

const slides = [Welcome, Speed, Tracking, ZeroKnowledge];

const ArrowButton = ({
  icon,
  onClick,
  disabled,
}: {
  icon: string;
  onClick: () => void;
  disabled: boolean;
}) => {
  return (
    <HuButton
      onClick={onClick}
      className={clsx(
        'bg-surface-elev my-2 mr-2 flex h-11 w-11 items-center justify-center rounded-full',
        !disabled && 'hover:bg-surface-elev/70',
      )}
    >
      <MsIcon
        icon={icon}
        className={clsx(
          'leading-none',
          !disabled ? 'text-text-primary' : 'text-text-tertiary',
        )}
      />
    </HuButton>
  );
};

function Onboarding() {
  const { t } = useTranslation('onboarding');

  const uiTheme = useAppStore((s) => s.uiTheme);

  const navigate = useAnimatedNavigate();
  const [emblaRef, emblaApi] = useEmblaCarousel({
    duration: 20,
  });

  const { selectedIndex, scrollSnaps, onDotButtonClick } =
    useDotButton(emblaApi);

  const handleNavigate = (route: string) => navigate(route);

  return (
    <InteractiveCard className="h-full">
      <div className="mb-12">
        <div className="relative flex h-[27px] items-center justify-center">
          <NymVpnTextLogo
            className={clsx(
              'h-[27px] w-[100px]',
              uiTheme === 'dark' ? 'fill-white' : 'fill-surface-bg',
            )}
          />
          <ButtonIconNew
            initialAnimation={true}
            icon="close"
            onClick={() => navigate(routes.root)}
            className="text-text-tertiary hover:text-text-primary transition-noborder absolute right-0 cursor-default dark:hover:text-white"
          />
        </div>
      </div>
      <section className="embla flex h-full w-full flex-col justify-between">
        <div className="flex flex-1 flex-col items-center justify-center gap-6">
          <div className="w-full overflow-hidden" ref={emblaRef}>
            <div className="flex touch-pinch-zoom">
              {slides.map((Slide, index) => (
                <div
                  className="min-w-0 flex-none basis-full translate-3d transform pl-4"
                  key={index}
                >
                  <Slide />
                </div>
              ))}
            </div>
          </div>
          <div className="flex w-full flex-row items-center justify-between">
            <ArrowButton
              icon="arrow_left"
              onClick={() => emblaApi?.scrollPrev()}
              disabled={!emblaApi?.canScrollPrev()}
            />
            <div className="bg-surface-elev flex flex-row gap-2 rounded-2xl px-3 py-2">
              {scrollSnaps.map((_, index) => (
                <DotButton
                  key={index}
                  onClick={() => onDotButtonClick(index)}
                  className={clsx(
                    'bg-surface-hair tap-highlight-transparent m-0 flex h-2 w-2 cursor-pointer touch-manipulation appearance-none items-center justify-center rounded-full border-0 p-0 no-underline',
                    index === selectedIndex ? 'bg-white' : '',
                  )}
                />
              ))}
            </div>
            <ArrowButton
              icon="arrow_right"
              onClick={() => emblaApi?.scrollNext()}
              disabled={!emblaApi?.canScrollNext()}
            />
          </div>
        </div>

        <div className="flex w-full flex-col items-center gap-4">
          <Button onClick={() => handleNavigate(routes.welcome)}>
            {t('controls.get-started')}
          </Button>
        </div>
      </section>
    </InteractiveCard>
  );
}

export default Onboarding;
