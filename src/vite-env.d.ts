declare const __SENTRY_RELEASE__: string;
declare const __SENTRY_ENVIRONMENT__: string;

declare module '*.vue' {
  import type { DefineComponent } from 'vue'
  const component: DefineComponent<{}, {}, any>
  export default component
}
