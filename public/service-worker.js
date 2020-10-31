importScripts('https://storage.googleapis.com/workbox-cdn/releases/5.1.2/workbox-sw.js');

// workbox.core - Provides core workbox functionality. Ths will be used for service worker updating.
const core = workbox.core;

// workbox.precaching - Helps to simplify the caching process
const precaching = workbox.precaching;

// We want to publish a new service worker and immediately update and control the page.
// // - https://developers.google.com/web/tools/workbox/modules/workbox-core#skip_waiting_and_clients_claim
core.skipWaiting();
core.clientsClaim();

precaching.precacheAndRoute(
  [
    { "revision": "1", "url": "index.html" },
    { "revision": "1", "url": "/" },
    { "revision": "1", "url": "/pwa" },
    { "revision": "1", "url": "public/index.css" },
    { "revision": "1", "url": "pkg/package_bg.wasm" },
    { "revision": "1", "url": "pkg/package.js" },
  ]
);
