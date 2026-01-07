import clsx from 'clsx';
import { useNavigate } from 'react-router';
import { useTranslation } from 'react-i18next';
import useEmblaCarousel from 'embla-carousel-react';
import { Button, PageAnim } from '../../ui';
import { NymVpnTextLogo } from '../../assets';
import { routes } from '../../router';
import { useMainDispatch } from '../../contexts/main';
import { StateDispatch } from '../../types';
import { kvSet } from '../../kvStore';
import { Speed, Tracking, Welcome, ZeroKnowledge } from './slides';
import { DotButton, useDotButton } from './CarouselDotButton';

const slides = [Welcome, Speed, Tracking, ZeroKnowledge];

function Onboarding() {
  const dispatch = useMainDispatch() as StateDispatch;
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
      dispatch({ type: 'set-onboarding-completed', completed: true });
      kvSet('onboarding-completed', true);
      navigate(routes.login, { state: { skip: true } });
    }
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
        </div>
      </section>
    </PageAnim>
  );
}

export default Onboarding;
