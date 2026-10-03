# Architecture

Tài liệu kiến trúc chính là [`game-architecture.md`](game-architecture.md).

Hệ thống phát triển theo một hướng: data schema → headless simulation → server
authority → client rendering → content expansion → release hardening. Mọi thay
đổi boundary hoặc protocol phải cập nhật tài liệu kiến trúc và API trong cùng
phase.
