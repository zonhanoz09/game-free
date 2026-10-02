# Architecture

`apps/client` chạy Bevy và giao tiếp với server qua WebSocket. `apps/server` phục vụ
HTTP/WebSocket và static web artifact từ `dist/wasm`. Các schema/rule dùng chung đặt
trong `crates/`.
