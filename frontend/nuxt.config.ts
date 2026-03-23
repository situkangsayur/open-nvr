export default defineNuxtConfig({
  compatibilityDate: '2025-03-22',
  devtools: { enabled: true },

  modules: [
    '@pinia/nuxt',
    '@nuxtjs/tailwindcss',
  ],

  runtimeConfig: {
    public: {
      apiUrl: process.env.NUXT_PUBLIC_API_URL || 'http://localhost:8888',
      keycloakUrl: process.env.NUXT_PUBLIC_KEYCLOAK_URL || 'http://localhost:8180',
      keycloakRealm: process.env.NUXT_PUBLIC_KEYCLOAK_REALM || 'opennvr',
      keycloakClientId: process.env.NUXT_PUBLIC_KEYCLOAK_CLIENT_ID || 'opennvr-frontend',
    },
  },

  app: {
    head: {
      title: 'Open-NVR',
      link: [
        { rel: 'manifest', href: '/manifest.json' },
        { rel: 'icon', type: 'image/svg+xml', href: '/favicon.svg' },
        { rel: 'icon', type: 'image/png', href: '/icon-192.png' },
        { rel: 'apple-touch-icon', href: '/icon-192.png' },
      ],
      meta: [
        { name: 'description', content: 'Open Source Network Video Recorder' },
        { name: 'theme-color', content: '#1e40af' },
        { name: 'apple-mobile-web-app-capable', content: 'yes' },
        { name: 'apple-mobile-web-app-status-bar-style', content: 'black-translucent' },
        { name: 'viewport', content: 'width=device-width, initial-scale=1, maximum-scale=1' },
      ],
    },
  },
})
