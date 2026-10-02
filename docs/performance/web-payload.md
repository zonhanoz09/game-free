# Web payload và chiến lược tải asset

## Kết quả đo hiện tại

| Nhóm | Trước | Sau | Ghi chú |
|---|---:|---:|---|
| WASM client | ~74 MB | ~17 MB | `opt-level=z`, LTO, strip symbol |
| Runtime assets | ~36 MB | ~14 MB | Chỉ copy asset client thực sự load |
| Tổng `dist/wasm` | ~107 MB | ~30 MB | Chưa tính gzip/Brotli ở CDN |

Kích thước được đo bằng:

```bash
du -ah dist/wasm | sort -h
```

## Phân tách payload

`dist/wasm` hiện gồm:

- `game-free_bg.wasm`: binary gameplay bắt buộc.
- `game-free.js` và các file bindgen: loader WASM.
- `index.html`: shell lobby/UI.
- `assets/audio`, `assets/fonts`, `assets/textures`: runtime assets tải ở phiên
  đầu.
- `assets/manifest.json`: danh sách initial/deferred asset.

Các file `.glb` và `.jpg` vẫn được giữ trong `assets/` làm source/deferred
assets, nhưng không được copy vào initial web payload vì client hiện chỉ load
PNG texture, font và audio. Khi có màn hình 3D hoặc preview cần dùng model,
hãy thêm route/loader lazy riêng thay vì đưa toàn bộ vào bundle đầu.

## Quy tắc build

`scripts/build-wasm.sh` là nguồn chuẩn để tạo artifact. Script:

1. Build client ở release với LTO và tối ưu kích thước.
2. Chạy `wasm-bindgen`.
3. Xóa runtime asset cũ để tránh file mồ côi.
4. Chỉ copy allowlist asset được client sử dụng.
5. Ghi manifest để các màn hình mới có thể lazy-load asset deferred.

Không commit file nén thủ công hoặc copy toàn bộ `assets/` vào `dist/wasm`;
CDN/Cloudflare nên đảm nhiệm Brotli/gzip cho WASM, JS, JSON, CSS và HTML.
