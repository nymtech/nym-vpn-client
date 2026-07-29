import { useState } from 'react';
import clsx from 'clsx';
import { Button as HuButton } from '@headlessui/react';
import useEmblaCarousel from 'embla-carousel-react';
import { motion } from 'motion/react';
import { useTranslation } from 'react-i18next';
import { useAnimatedNavigate } from '../../hooks/useAnimatedNavigate';
import { Button, ButtonIconNew, MsIcon } from '../../ui';
import { NymVpnTextLogo } from '../../assets';
import { routes } from '../../router';
import { InteractiveCard } from '../home/InteractiveCard';
import { Around, Network, Welcome } from './slides';
import { DotButton, useDotButton } from './CarouselDotButton';
import { NetworkVariant, VariantToggle } from './VariantToggle';

const ArrowButton = ({
  icon,
  label,
  onClick,
  hidden,
}: {
  icon: string;
  label: string;
  onClick: () => void;
  hidden: boolean;
}) => {
  return (
    <HuButton
      onClick={onClick}
      aria-label={label}
      className={clsx(
        'bg-surface-elev flex h-10 w-10 shrink-0 items-center justify-center rounded-full',
        hidden ? 'invisible' : 'hover:bg-surface-hair',
      )}
    >
      <MsIcon icon={icon} className="text-text-primary leading-none" />
    </HuButton>
  );
};

function Onboarding() {
  const { t } = useTranslation('onboarding');

  const navigate = useAnimatedNavigate();
  const [emblaRef, emblaApi] = useEmblaCarousel({ duration: 20 });

  const { selectedIndex, scrollSnaps, onDotButtonClick } =
    useDotButton(emblaApi);

  const [variant, setVariant] = useState<NetworkVariant>('dvpn');

  const slides = [
    { id: 'welcome', content: <Welcome /> },
    { id: 'network', content: <Network variant={variant} /> },
    { id: 'around', content: <Around /> },
  ];

  const onNetworkSlide = slides[selectedIndex]?.id === 'network';

  const tagline = onNetworkSlide ? t(`network.${variant}.tagline`) : '';

  return (
    <motion.div
      initial={{ opacity: 0, x: '-1rem' }}
      animate={{ opacity: 1, x: 0 }}
      transition={{ duration: 0.2, ease: 'easeOut' }}
      className="flex h-full flex-col"
    >
      <div className="flex h-10 shrink-0 justify-end">
        <ButtonIconNew
          initialAnimation={true}
          icon="close"
          aria-label={t('controls.close')}
          onClick={() => navigate(routes.root)}
          className="text-text-tertiary hover:text-text-primary transition-noborder cursor-default dark:hover:text-white"
        />
      </div>

      <div className="flex min-h-0 grow flex-col">
        <div
          className="-mx-4 my-auto w-[calc(100%+2rem)] shrink-0 overflow-hidden"
          ref={emblaRef}
        >
          <div className="flex touch-pinch-zoom">
            {slides.map((slide) => (
              <div
                key={slide.id}
                className="flex min-w-0 flex-none basis-full translate-3d transform flex-col justify-center"
              >
                {slide.content}
              </div>
            ))}
          </div>
        </div>
      </div>

      <div className="flex shrink-0 flex-col gap-6">
        {onNetworkSlide && (
          <VariantToggle value={variant} onChange={setVariant} />
        )}

        <div className="flex w-full flex-row items-center justify-between">
          <ArrowButton
            icon="arrow_left"
            label={t('controls.previous')}
            onClick={() => emblaApi?.scrollPrev()}
            hidden={selectedIndex === 0}
          />
          <div className="bg-surface-elev flex flex-row gap-2 rounded-2xl px-3 py-2">
            {scrollSnaps.map((_, index) => (
              <DotButton
                key={index}
                onClick={() => onDotButtonClick(index)}
                aria-label={`${index + 1}`}
                aria-current={index === selectedIndex}
                className={clsx(
                  'bg-surface-hair tap-highlight-transparent m-0 flex h-2 w-2 cursor-pointer touch-manipulation appearance-none items-center justify-center rounded-full border-0 p-0 no-underline',
                  index === selectedIndex ? 'bg-white' : '',
                )}
              />
            ))}
          </div>
          <ArrowButton
            icon="arrow_right"
            label={t('controls.next')}
            onClick={() => emblaApi?.scrollNext()}
            hidden={selectedIndex === slides.length - 1}
          />
        </div>

        <InteractiveCard>
          <div className="flex flex-col items-center gap-2.5">
            <NymVpnTextLogo className="fill-text-primary h-[27px] w-[100px]" />
            <p className="text-text-secondary h-5 text-center text-sm leading-5">
              {tagline}
            </p>
            <Button onClick={() => navigate(routes.welcome)}>
              {t('controls.get-started')}
            </Button>
          </div>
        </InteractiveCard>
      </div>
    </motion.div>
  );
}

export default Onboarding;
