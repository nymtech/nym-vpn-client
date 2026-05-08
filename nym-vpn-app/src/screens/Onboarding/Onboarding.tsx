import clsx from 'clsx';
import { Button as HuButton } from '@headlessui/react';
import useEmblaCarousel from 'embla-carousel-react';
import { useAnimatedNavigate } from '../../hooks/useAnimatedNavigate';
import { ButtonIconNew, ButtonNew, MsIcon } from '../../ui';
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
        'bg-mercury dark:bg-mine-shaft my-2 mr-2 flex h-11 w-11 items-center justify-center rounded-full',
        !disabled && 'hover:bg-bombay dark:hover:bg-charcoal',
      )}
    >
      <MsIcon
        icon={icon}
        className={clsx(
          'leading-none',
          !disabled ? 'text-text-primary' : 'text-bombay',
        )}
      />
    </HuButton>
  );
};

function Onboarding() {
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
              uiTheme === 'dark' ? 'fill-white' : 'fill-ash',
            )}
          />
          <ButtonIconNew
            initialAnimation={true}
            icon="close"
            onClick={() => navigate(routes.root)}
            className="text-bombay hover:text-baltic-sea transition-noborder absolute right-0 cursor-default dark:hover:text-white"
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
            <div className="bg-mercury dark:bg-mine-shaft flex flex-row gap-2 rounded-2xl px-3 py-2">
              {scrollSnaps.map((_, index) => (
                <DotButton
                  key={index}
                  onClick={() => onDotButtonClick(index)}
                  className={clsx(
                    'bg-bombay dark:bg-charcoal tap-highlight-transparent m-0 flex h-2 w-2 cursor-pointer touch-manipulation appearance-none items-center justify-center rounded-full border-0 p-0 no-underline',
                    index === selectedIndex ? 'bg-white dark:bg-white' : '',
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
          <ButtonNew onClick={() => handleNavigate(routes.welcome)}>
            Get Started
          </ButtonNew>
        </div>
      </section>
    </InteractiveCard>
  );
}

export default Onboarding;
