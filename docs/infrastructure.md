# 🌐 Kiến Trúc Hạ Tầng (Infrastructure Architecture)

## 📌 Chuẩn Luồng Truy Cập (Traffic Flow)
```
[ 🌍 Internet: Người Chơi / Web Browsers ]
                     │  (HTTPS / WSS - Port 443)
                     ▼
[ 🛡️ Cloudflare Edge Network ]
  ├── Tự động cấp phát chứng chỉ SSL/TLS (HTTPS)
  ├── Bảo vệ chống tấn công DDoS & Web Application Firewall (WAF)
  ├── Caching CDN cho các tệp tĩnh (WASM, GLB 3D, Textures, Audio)
  └── Đóng gói lưu lượng vào Cloudflare Tunnel (Protocol: QUIC/HTTP2)
                     │
                     ▼ (Outbound-only Encrypted Tunnel)
[ ☁️ Oracle Cloud Infrastructure (OCI) - ap-singapore-1 ]
  ├── 🚪 NAT Gateway (Outbound Only)
  │     └── Cho phép VM gửi dữ liệu ra Cloudflare Edge, KHÔNG CHO PHÉP Inbound từ Internet
  │
  └── 🏰 VCN Riêng Tư (10.0.0.0/16) - KHÔNG CÓ INTERNET GATEWAY
        │
        └── 🔒 Private Subnet (10.0.1.0/24)
              │  ├── VNIC Public IP: NONE (Đã xoá hoàn toàn)
              │  ├── VNIC Private IP: 10.0.1.60
              │  └── Security List: Chỉ cho phép lưu lượng nội bộ VCN (10.0.0.0/16)
              │
              └── 🖥️ Compute Instance: VM.Standard.A1.Flex (ARM Ampere)
                    │   2 OCPU, 12 GB RAM, 50 GB Boot Disk (Ubuntu 22.04 LTS)
                    │
                    └── 🐳 Docker Environment (game-free_default Bridge Network)
                          │
                          ├── 🛡️ [cloudflared-tunnel] Container
                          │     ├── Kết nối ra Cloudflare Edge qua Tunnel Token
                          │     └── Chuyển tiếp request nội bộ tới: http://game-server:8080
                          │
                          └── ⚔️ [tactical-arena-pvp] Container
                                ├── Cổng lắng nghe nội bộ: 8080 (expose only, không map host port)
                                ├── Web Client & Asset Server (Axum)
                                └── Real-time PvP Engine (WebSocket)
```

---

## 🔐 Bảng So Sánh An Toàn & Bảo Mật

| Tiêu Chí | Mô Hình Truyền Thống (Có Public IP) | Mô Hình Hiện Tại (Cloudflare Tunnel + OCI Private) |
| :--- | :--- | :--- |
| **Public IP máy chủ** | Lộ Public IP trên DNS toàn cầu (`161.118.x.x`) | ❌ **KHÔNG CÓ (NONE)** |
| **Cổng Inbound mở** | Mở Port 80, 443, 8080 ra `0.0.0.0/0` | 🔒 **ZERO INBOUND PORTS** |
| **Nguy cơ tấn công mạng** | Dễ bị port scan, DDoS trực tiếp vào IP máy chủ | 🛡️ Hacker không thể tìm thấy hay scan IP của bạn |
| **Internet Gateway** | Bắt buộc phải có để nhận request | ❌ **ĐÃ HUỶ BỎ (Không dùng IGW)** |
| **Đường ra Internet** | Đi qua Internet Gateway | 🟢 **OCI NAT Gateway** (Chỉ đi ra, không cho đi vào) |
| **Bảo vệ DDoS & SSL** | Cần tự cài Nginx, Certbot SSL, cấu hình iptables | 🟢 **Cloudflare Edge tự động xử lý 100%** |
