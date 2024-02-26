import { lazy } from 'solid-js';
import './index.scss';

export const LazyButton = lazy(() => import('~/components/button'));
export const LazyRow = lazy(() => import('~/components/row'));
export const LazyCol = lazy(() => import('~/components/col'));
export const LazyProgress = lazy(() => import('~/components/progress'));
export const LazyDropdown = lazy(() => import('~/components/dropdown'));
export const LazyTooltip = lazy(() => import('~/components/tooltip'));
export const LazyLink = lazy(() => import('~/components/link'));
export const LazyFlex = lazy(() => import('~/components/flex'));
export const LazyEmpty = lazy(() => import('~/components/empty'));
export const LazyList = lazy(() => import('~/components/list'));
export const LazyListItem = lazy(() => import('~/components/list/item'));
export const LazyFloatButton = lazy(
  () => import('~/components/floatButton/button'),
);
export const LazyFloatButtonGroup = lazy(
  () => import('~/components/floatButton/group'),
);
export const LazySwitch = lazy(() => import('~/components/switch'));
export const LazySpace = lazy(() => import('~/components/space'));
export const LazySpaceCompact = lazy(
  () => import('~/components/space/compact'),
);

export const LazyInput = lazy(() => import('~/components/input'));
export const LazyInputNumber = lazy(() => import('~/components/input-number'));

export const LazySpinner = lazy(() => import('~/components/spinner'));
export const LazyAlert = lazy(() => import('~/components/alert'));
