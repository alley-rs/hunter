import { lazy } from 'solid-js';

export const LazyTable = lazy(() => import('~/app/table'));

export const LazyRow = lazy(
  () => import('alley-components/lib/components/row'),
);
export const LazyButton = lazy(
  () => import('alley-components/lib/components/button'),
);
export const LazyFlex = lazy(
  () => import('alley-components/lib/components/flex'),
);
export const LazyInput = lazy(
  () => import('alley-components/lib/components/input'),
);
export const LazyInputNumber = lazy(
  () => import('alley-components/lib/components/input-number'),
);
export const LazySwitch = lazy(
  () => import('alley-components/lib/components/switch'),
);
export const LazyTooltip = lazy(
  () => import('alley-components/lib/components/tooltip'),
);
export const LazySpace = lazy(
  () => import('alley-components/lib/components/space'),
);
export const LazyEmpty = lazy(
  () => import('alley-components/lib/components/empty'),
);
export const LazyCol = lazy(
  () => import('alley-components/lib/components/col'),
);
export const LazyCircleProgress = lazy(
  () => import('alley-components/lib/components/progress/circle'),
);
export const LazySpaceCompact = lazy(
  () => import('alley-components/lib/components/space/compact'),
);
export const LazySpinner = lazy(
  () => import('alley-components/lib/components/spinner'),
);
export const LazyToast = lazy(
  () => import('alley-components/lib/components/toast'),
);
