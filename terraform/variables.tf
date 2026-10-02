# ==========================================
# Oracle Cloud Infrastructure (OCI) Config
# ==========================================

variable "tenancy_ocid" {
  type        = string
  description = "OCID của Tenancy trên Oracle Cloud"
}

variable "user_ocid" {
  type        = string
  description = "OCID của User API trên Oracle Cloud"
}

variable "fingerprint" {
  type        = string
  description = "Fingerprint của OCI API Signing Key (vd: 12:34:56:...)"
}

variable "private_key_path" {
  type        = string
  default     = "~/.oci/oci_api_key.pem"
  description = "Đường dẫn file Private Key (.pem) của OCI API"
}

variable "region" {
  type        = string
  default     = "ap-singapore-1"
  description = "OCI Region (vd: ap-singapore-1, ap-tokyo-1, us-ashburn-1)"
}

variable "compartment_ocid" {
  type        = string
  default     = null
  description = "OCID của Compartment (nếu để trống sẽ dùng Tenancy OCID)"
}

# ==========================================
# OCI Compute & Free Tier Sizing
# ==========================================

variable "instance_shape" {
  type        = string
  default     = "VM.Standard.A1.Flex"
  description = "Hình thái máy ảo: VM.Standard.A1.Flex (ARM Ampere Free Tier) hoặc VM.Standard.E2.1.Micro (AMD Free Tier)"
}

variable "instance_ocpus" {
  type        = number
  default     = 2
  description = "Số OCPU cho Flex shape (Free Tier tối đa 4 OCPU)"
}

variable "instance_memory_in_gbs" {
  type        = number
  default     = 12
  description = "Dung lượng RAM (GB) cho Flex shape (Free Tier tối đa 24 GB)"
}

variable "ssh_public_key_path" {
  type        = string
  default     = "~/.ssh/id_rsa.pub"
  description = "Đường dẫn file SSH Public Key để kết nối vào máy chủ OCI"
}

# ==========================================
# Cloudflare Configuration
# ==========================================

variable "cloudflare_api_token" {
  type        = string
  description = "Cloudflare API Token có quyền Zone.DNS (Edit)"
  sensitive   = true
}

variable "cloudflare_zone_id" {
  type        = string
  description = "Zone ID của tên miền trong Cloudflare Dashboard"
}

variable "subdomain" {
  type        = string
  default     = "game"
  description = "Subdomain để trỏ về máy chủ game (vd: 'game' -> game.yourdomain.com, hoặc '@' cho root domain)"
}

variable "enable_cloudflare_proxy" {
  type        = bool
  default     = true
  description = "Bật Cloudflare CDN/Proxy (Hỗ trợ SSL miễn phí, CDN & chống DDoS)"
}
