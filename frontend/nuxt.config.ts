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
      keycloakUrl: process.env.NUXT_PUBLIC_KEYCLOAK_URL || 'http://localhost:8080',
      keycloakRealm: process.env.NUXT_PUBLIC_KEYCLOAK_REALM || 'opennvr',
      keycloakClientId: process.env.NUXT_PUBLIC_KEYCLOAK_CLIENT_ID || 'opennvr-frontend',
    },
  },

  app: {
    head: {
      title: 'Open-NVR',
      meta: [
        { name: 'description', content: 'Open Source Network Video Recorder' },
      ],
    },
  },
})
