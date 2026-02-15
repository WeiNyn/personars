var APP_VERSION = 'v0.1.0';
var cacheName = 'personars-' + APP_VERSION;
var filesToCache = [
  './',
  './index.html',
  './personars.js',
  './personars_bg.wasm',
];

/* Install: cache all files under the new version key */
self.addEventListener('install', function (e) {
  self.skipWaiting();
  e.waitUntil(
    caches.open(cacheName).then(function (cache) {
      return cache.addAll(filesToCache);
    })
  );
});

/* Activate: purge any caches from older versions */
self.addEventListener('activate', function (e) {
  e.waitUntil(
    caches.keys().then(function (keyList) {
      return Promise.all(
        keyList.map(function (key) {
          if (key !== cacheName) {
            console.log('[SW] Removing old cache:', key);
            return caches.delete(key);
          }
        })
      );
    })
  );
  return self.clients.claim();
});

/* Cache-first: instant loads, cache invalidation handled by version bump */
self.addEventListener('fetch', function (e) {
  e.respondWith(
    caches.match(e.request).then(function (response) {
      return response || fetch(e.request);
    })
  );
});
