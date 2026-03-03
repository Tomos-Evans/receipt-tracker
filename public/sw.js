const CACHE_NAME = 'receipt-tracker-v2';

// Derived from the SW registration scope — correct for any subpath deployment
// e.g. "https://user.github.io/receipt-tracker/" or "http://localhost:8080/"
const SCOPE = self.registration.scope;

const APP_SHELL = [
  SCOPE,
  SCOPE + 'index.html',
  SCOPE + 'receipt_tracker.js',
  SCOPE + 'receipt_tracker_bg.wasm',
  SCOPE + 'manifest.json',
  SCOPE + 'icons/icon-192.png',
  SCOPE + 'icons/icon-512.png',
];

// External resources to pre-cache at install time using no-cors (opaque) mode,
// matching how the browser loads them (<link stylesheet>, <script defer>).
const EXTERNAL_PRECACHE = [
  'https://fonts.googleapis.com/icon?family=Material+Icons',
  'https://cdnjs.cloudflare.com/ajax/libs/jspdf/2.5.1/jspdf.umd.min.js',
];

self.addEventListener('install', (event) => {
  event.waitUntil(
    caches.open(CACHE_NAME).then(async (cache) => {
      await cache.addAll(APP_SHELL);
      // Pre-cache CDN resources; don't block install if network is unavailable
      await Promise.allSettled(
        EXTERNAL_PRECACHE.map((url) =>
          fetch(url, { mode: 'no-cors' })
            .then((res) => cache.put(url, res))
            .catch(() => {})
        )
      );
    }).then(() => self.skipWaiting())
  );
});

self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches.keys().then((keys) =>
      Promise.all(
        keys.filter((key) => key !== CACHE_NAME).map((key) => caches.delete(key))
      )
    ).then(() => self.clients.claim())
  );
});

self.addEventListener('fetch', (event) => {
  if (event.request.method !== 'GET') return;

  event.respondWith(
    caches.match(event.request).then((cached) => {
      if (cached) return cached;

      return fetch(event.request).then((response) => {
        // Cache successful same-origin responses and opaque CDN responses
        if (response.ok || response.type === 'opaque') {
          caches.open(CACHE_NAME).then((cache) => cache.put(event.request, response.clone()));
        }
        return response;
      }).catch(() => {
        // Offline fallback: serve the app shell for navigation requests
        if (event.request.mode === 'navigate') {
          return caches.match(SCOPE + 'index.html');
        }
      });
    })
  );
});
