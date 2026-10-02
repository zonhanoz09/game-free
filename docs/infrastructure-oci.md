# 🚀 Hướng Dẫn Triển Khai Game Lên Oracle Cloud (OCI) Free Tier & Cloudflare Bằng Terraform

Hệ thống triển khai **Tactical Arena CCG Auto-Battler** lên OCI Always Free và
Cloudflare. Runtime hiện tại là Rust/Axum server, WebAssembly client và WebSocket PvP.

---

## 📌 Tổng Quan Kiến Trúc Hạ Tầng
- **Oracle Cloud (Always Free)**:
  - Máy ảo **Ampere A1 Flex** (ARM 64-bit): 2 OCPU, 12 GB RAM, 50 GB Boot Volume (Nằm trong hạn mức miễn phí 4 OCPU, 24 GB RAM, 200 GB Storage).
  - Virtual Cloud Network, subnet và security list phục vụ Bastion/Cloudflare Tunnel.
  - Tự động chạy `cloud-init` cài đặt Docker, mở Firewall nội bộ của Ubuntu (`iptables`) và thiết lập dịch vụ systemd cho container.
- **Cloudflare**:
  - Tự động tạo bản ghi DNS (Type A) trỏ về Public IP của máy ảo OCI.
  - Kích hoạt Proxy (`proxied = true`) để cung cấp chứng chỉ SSL/TLS miễn phí (`https://...` và `wss://...`).

---

## 📝 Bước 1: Chuẩn Bị Thông Tin Oracle Cloud (OCI)

### 1. Lấy Tenancy OCID và User OCID
1. Đăng nhập vào [Oracle Cloud Console](https://cloud.oracle.com/).
2. Click vào **Avatar tài khoản** ở góc trên cùng bên phải:
   - Mục **Tenancy**: Click vào tên Tenancy và bấm **Copy** ở dòng `OCID`.
   - Mục **User Settings / My profile**: Bấm **Copy** ở dòng `OCID` của User.

### 2. Tạo API Key trên OCI
1. Trong trang **User Settings**, kéo xuống phần **Resources** bên trái ➔ Chọn **API Keys**.
2. Click **Add API Key**:
   - Chọn **Generate API Key Pair**.
   - Bấm **Download Private Key** (Lưu file thành `~/.oci/oci_api_key.pem`).
   - Bấm **Add**.
3. Sau khi bấm Add, màn hình hiện ra bảng **Configuration File Preview**. Copy các giá trị:
   - `fingerprint` (Ví dụ: `12:34:56:78:90:...`)
   - `region` (Ví dụ: `ap-singapore-1`, `ap-tokyo-1`, `us-ashburn-1`)

### 3. Phân quyền cho file Private Key
Chạy lệnh sau trên máy của bạn:
```bash
mkdir -p ~/.oci
# Đặt file vừa tải về vào ~/.oci/oci_api_key.pem
chmod 600 ~/.oci/oci_api_key.pem
```

---

## 🌐 Bước 2: Chuẩn Bị Thông Tin Cloudflare

### 1. Lấy Cloudflare API Token
1. Đăng nhập vào [Cloudflare Dashboard](https://dash.cloudflare.com/).
2. Click Avatar góc trên phải ➔ **My Profile** ➔ **API Tokens**.
3. Bấm **Create Token** ➔ Chọn template **Edit zone DNS** (Use template).
4. Ở mục **Zone Resources**: Chọn `Include` ➔ `Specific zone` ➔ Chọn tên miền của bạn.
5. Bấm **Continue to summary** ➔ **Create Token** và copy mã Token.

### 2. Lấy Cloudflare Zone ID
1. Trên Cloudflare Dashboard, click chọn tên miền của bạn.
2. Tại trang **Overview**, kéo xuống dưới cùng cột bên phải ➔ Mục **API** ➔ Copy **Zone ID**.

---

## ⚙️ Bước 3: Cấu Hình Terraform

1. Di chuyển vào thư mục `deploy/terraform`:
   ```bash
   cd deploy/terraform
   ```
2. Copy file mẫu để tạo file cấu hình thực tế:
   ```bash
   cp terraform.tfvars.example terraform.tfvars
   ```
3. Mở file `terraform.tfvars` và điền các thông tin đã chuẩn bị ở Bước 1 & Bước 2:
   ```hcl
   tenancy_ocid     = "ocid1.tenancy.oc1..aaaaaaaaxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
   user_ocid        = "ocid1.user.oc1..aaaaaaaayyyyyyyyyyyyyyyyyyyyyyyyyyyy"
   fingerprint      = "12:34:56:78:90:ab:cd:ef:12:34:56:78:90:ab:cd:ef"
   private_key_path = "~/.oci/oci_api_key.pem"
   region           = "ap-singapore-1"

   instance_shape         = "VM.Standard.A1.Flex"
   instance_ocpus         = 2
   instance_memory_in_gbs = 12

   ssh_public_key_path = "~/.ssh/id_rsa.pub"

   cloudflare_api_token = "mã_token_cloudflare_của_bạn"
   cloudflare_zone_id   = "zone_id_của_domain_trên_cloudflare"
   subdomain            = "game" # Game sẽ chạy tại: https://game.ten-mien-cua-ban.com
   enable_cloudflare_proxy = true
   ```

---

## 🚀 Bước 4: Khởi Tạo & Triển Khai Hạ Tầng (Terraform Apply)

1. Khởi tạo Terraform:
   ```bash
   terraform init
   ```
2. Xem trước các tài nguyên sẽ được tạo:
   ```bash
   terraform plan
   ```
3. Tạo máy ảo và cấu hình DNS tự động:
   ```bash
   terraform apply
   ```
   *(Nhập `yes` khi được hỏi xác nhận)*

Sau khi hoàn tất, Terraform sẽ hiển thị kết quả:
```
Outputs:
server_public_ip       = "150.136.xxx.xxx"
ssh_command            = "ssh ubuntu@150.136.xxx.xxx"
game_domain            = "game.yourdomain.com"
game_url               = "https://game.yourdomain.com"
direct_ip_url          = "http://150.136.xxx.xxx:8080"
```

---

## 🚢 Bước 5: Deploy Mã Nguồn Lên Máy Chủ

Quay lại thư mục gốc dự án và chạy script tự động deploy:
```bash
cd ..
./scripts/deploy-to-oci.sh
```

Script sẽ tự động:
1. Tạo Bastion port-forwarding session tới private instance.
2. Biên dịch WebAssembly (`wasm32-unknown-unknown`) bản tối ưu Release.
3. Đồng bộ toàn bộ `apps/server/`, `dist/wasm/`, `assets/` lên máy chủ OCI.
4. Build image Rust server và khởi chạy Docker container.
5. In ra đường dẫn truy cập game!

Truy cập ngay: **`https://game.yourdomain.com`** để trải nghiệm game online mượt mà với bạn bè!
