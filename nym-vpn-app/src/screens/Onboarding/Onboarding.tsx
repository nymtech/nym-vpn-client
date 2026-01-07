import clsx from 'clsx';
import { useNavigate } from 'react-router';
import { useTranslation } from 'react-i18next';
import useEmblaCarousel from 'embla-carousel-react';
import { Button, PageAnim } from '../../ui';
import { NymVpnTextLogo } from '../../assets';
import { routes } from '../../router';
import { Speed, Tracking, Welcome, ZeroKnowledge } from './slides';
import { DotButton, useDotButton } from './CarouselDotButton';

const slides = [Welcome, Tracking, ZeroKnowledge, Speed];

function Onboarding() {
  const navigate = useNavigate();
  const { t } = useTranslation('onboarding');
  const [emblaRef, emblaApi] = useEmblaCarousel({
    duration: 20,
  });

  const { selectedIndex, scrollSnaps, onDotButtonClick } =
    useDotButton(emblaApi);

  const handleNext = () => {
    if (emblaApi?.canScrollNext()) {
      emblaApi?.scrollNext();
    } else {
      handleSkip();
    }
  };

  const handleSkip = () => {
    navigate(routes.welcome);
  };

  return (
    <PageAnim className="h-full flex flex-col justify-end items-center gap-8 select-none cursor-default">
      <section className="embla w-full h-full flex flex-col gap-4 justify-between">
        <div className="flex flex-1 justify-center flex-col gap-10 items-center">
          <NymVpnTextLogo className="w-32" />
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
        </div>

        <div className="flex flex-col items-center gap-4 w-full">
          <Button onClick={handleNext}>{t('controls.next')}</Button>
          <Button
            outline
            color="gray"
            onClick={handleSkip}
            className="group border-none hover:ring-0! dark:hover:ring-0! w-fit!"
          >
            <span className="flex items-center gap-2 text-black dark:text-white group-hover:text-black/50 dark:group-hover:text-white/80">
              {t('controls.skip')}
            </span>
          </Button>
        </div>
      </section>
    </PageAnim>
  );
}

export default Onboarding;
