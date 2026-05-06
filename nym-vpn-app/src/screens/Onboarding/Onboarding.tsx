import clsx from 'clsx';
import { useAnimatedNavigate } from '../../hooks/useAnimatedNavigate';
import { useTranslation } from 'react-i18next';
import { Button as HuButton } from '@headlessui/react';
import useEmblaCarousel from 'embla-carousel-react';
import { Button, ButtonIconNew, ButtonNew, MsIcon, PageAnim } from '../../ui';
import { NymVpnTextLogo } from '../../assets';
import { routes } from '../../router';
import { Speed, Tracking, Welcome, ZeroKnowledge } from './slides';
import { DotButton, useDotButton } from './CarouselDotButton';
import { InteractiveCard } from '../home/InteractiveCard';
import { useAppStore } from '../../store/index';

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
        'h-11 w-11 my-2 mr-2 flex items-center justify-center rounded-full bg-mercury dark:bg-mine-shaft',
        !disabled && 'hover:bg-bombay dark:hover:bg-charcoal',
      )}
    >
      <MsIcon
        icon={icon}
        className={clsx(
          'leading-none',
          !disabled ? 'text-baltic-sea dark:text-white' : 'text-bombay',
        )}
      />
    </HuButton>
  );
};

function Onboarding() {
  const uiTheme = useAppStore((s) => s.uiTheme);

  const navigate = useAnimatedNavigate();
  const { t } = useTranslation('onboarding');
  const [emblaRef, emblaApi] = useEmblaCarousel({
    duration: 20,
  });

  const { selectedIndex, scrollSnaps, onDotButtonClick } =
    useDotButton(emblaApi);

  const handleNavigate = (route: string) => navigate(route);

  return (
    <InteractiveCard className="h-full">
      <div className="mb-12">
        <div className="flex items-center justify-center relative h-[27px]">
          <NymVpnTextLogo
            className={clsx(
              'w-[100px] h-[27px]',
              uiTheme === 'dark' ? 'fill-white' : 'fill-ash',
            )}
          />
          <ButtonIconNew
            initialAnimation={true}
            icon="close"
            onClick={() => navigate(routes.root)}
            className="absolute right-0 text-bombay hover:text-baltic-sea dark:hover:text-white transition-noborder cursor-default"
          />
        </div>
      </div>
      <section className="embla w-full h-full flex flex-col justify-between">
        <div className="flex flex-1 justify-center flex-col gap-6 items-center">
          {/* <NymVpnTextLogo className="w-32" /> */}
          <div className="overflow-hidden w-full" ref={emblaRef}>
            <div className="flex touch-pinch-zoom">
              {slides.map((Slide, index) => (
                <div
                  className="transform translate-3d flex-none basis-full min-w-0 pl-4"
                  key={index}
                >
                  <Slide />
                </div>
              ))}
            </div>
          </div>
          <div className="w-full flex flex-row justify-between items-center">
            <ArrowButton
              icon="arrow_left"
              onClick={() => emblaApi?.scrollPrev()}
              disabled={!emblaApi?.canScrollPrev()}
            />
            <div className="flex flex-row gap-2 bg-mercury dark:bg-mine-shaft px-3 py-2 rounded-2xl">
              {scrollSnaps.map((_, index) => (
                <DotButton
                  key={index}
                  onClick={() => onDotButtonClick(index)}
                  className={clsx(
                    'flex items-center justify-center rounded-full cursor-pointer bg-bombay dark:bg-charcoal border-0 p-0 m-0 w-2 h-2 appearance-none tap-highlight-transparent no-underline touch-manipulation',
                    index === selectedIndex ? ' bg-white dark:bg-white' : '',
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

        <div className="flex flex-col items-center gap-4 w-full">
          <ButtonNew onClick={() => handleNavigate(routes.loginScreen2)}>
            Get Started
          </ButtonNew>
        </div>
      </section>
    </InteractiveCard>
  );
}

export default Onboarding;
