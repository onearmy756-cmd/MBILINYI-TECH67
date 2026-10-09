/* Mbilinyi Tech Solutions — Service Worker v1-mts
 * Simple safe strategy:
 *  - Pre-cache core static pages on install (clean URLs, no .php).
 *  - Cache-first for GETs with known file extensions (images, fonts, css, js) — fall back to network.
 *  - Network-first for HTML pages and API requests (always get fresh content, save to cache).
 */
const CACHE_NAME = 'mts-cache-v1';
/* Clean URLs — no .php anywhere. The server rewrites /pricing -> pricing.php. */
const PRECACHE_URLS = [
  '/',
  '/services',
  '/work',
  '/pricing',
  '/contact',
  '/stack',
  '/ai',
  '/process',
  '/track',
  '/auth',
  '/manifest.json',
  '/assets/app.js',
];

self.addEventListener('install', event => {
  event.waitUntil(
    caches.open(CACHE_NAME).then(cache => {
      return Promise.all(
        PRECACHE_URLS.map(url =>
          cache.add(url).catch(() => { /* ignore — some pages may need session */ })
        )
      );
    }).then(() => self.skipWaiting())
  );
});

self.addEventListener('activate', event => {
  event.waitUntil(
    caches.keys().then(keys => Promise.all(
      keys.filter(k => k !== CACHE_NAME).map(k => caches.delete(k))
    )).then(() => self.clients.claim())
  );
});

self.addEventListener('fetch', event => {
  const req = event.request;
  if (req.method !== 'GET') return;

  const url = new URL(req.url);
  const isApi = url.pathname.startsWith('/api/');
  const isHtml = req.headers.get('accept')?.includes('text/html');
  const isPhp = url.pathname.endsWith('.php');

  if (isApi || isHtml || isPhp) {
    // Network-first for pages and API
    event.respondWith(
      fetch(req)
        .then(res => {
          const copy = res.clone();
          caches.open(CACHE_NAME).then(c => c.put(req, copy)).catch(() => {});
          return res;
        })
        .catch(() => caches.match(req).then(r => r || caches.match('/')))
    );
    return;
  }

  // Cache-first for everything else (CSS, JS, images, fonts, manifest, CDN...)
  event.respondWith(
    caches.match(req).then(cached => {
      return cached || fetch(req).then(res => {
        const copy = res.clone();
        caches.open(CACHE_NAME).then(c => c.put(req, copy)).catch(() => {});
        return res;
      }).catch(() => cached);
    })
  );
});
