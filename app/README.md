# opticon-app

The PWA — HTML/CSS/JS, runs in each phone's browser and installs to the home
screen. The server serves these files; they execute on the phone.

**Does:** live view + two-way talk (WebRTC), recent-events/clip playback, login
form, service worker (receives push), PWA install (manifest + add-to-home-screen).

**Required:** served over **HTTPS** (or `localhost` in dev) — service workers,
web-push, and camera/mic access all need a secure context.

The service worker is what makes push work when the app is closed — it's what
turns this from "a website" into "an app that buzzes your phone."
