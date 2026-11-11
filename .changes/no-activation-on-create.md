---
"wry": patch
---

On macOS, creating a webview no longer activates the application. The window that holds the webview decides the activation when it is shown, so a webview that a background application creates leaves the active application in front.
