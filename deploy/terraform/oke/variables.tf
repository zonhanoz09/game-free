# ==============================================================================
# Oracle Cloud Infrastructure (OCI) Authentication
# ==============================================================================

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
  description = "Fingerprint của OCI API Key"
}

variable "private_key_path" {
  type        = string
  default     = "~/.oci/oci_api_key.pem"
  description = "Đường dẫn file Private Key (.pem)"
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

# ==============================================================================
# OKE (Oracle Kubernetes Engine) Free Tier Settings
# ==============================================================================

variable "cluster_name" {
  type        = string
  default     = "tactical-arena-oke-free"
  description = "Tên của Kubernetes Cluster"
}

variable "k8s_version" {
  type        = string
  default     = "v1.30.1"
  description = "Phiên bản Kubernetes (vd: v1.30.1, v1.29.1)"
}

variable "node_shape" {
  type        = string
  default     = "VM.Standard.A1.Flex"
  description = "Hình thái máy chủ Worker Node: VM.Standard.A1.Flex (ARM Ampere Free Tier)"
}

variable "node_count" {
  type        = number
  default     = 2
  description = "Số lượng Worker Nodes (vd: 2 nodes)"
}

variable "node_ocpus" {
  type        = number
  default     = 1
  description = "Số OCPU cho mỗi node (2 nodes x 1 OCPU = 2 OCPUs, nằm trọn trong hạn mức 4 OCPU miễn phí)"
}

variable "node_memory_in_gbs" {
  type        = number
  default     = 6
  description = "Dung lượng RAM (GB) cho mỗi node (2 nodes x 6 GB = 12 GB, nằm trọn trong hạn mức 24 GB miễn phí)"
}

variable "ssh_public_key_path" {
  type        = string
  default     = "~/.ssh/id_rsa.pub"
  description = "Đường dẫn SSH Public Key để kết nối Worker Node"
}

# ==============================================================================
# Cloudflare & Cloudflare Tunnel Settings (No 10 Mbps Limit)
# ==============================================================================

variable "cloudflare_api_token" {
  type        = string
  description = "Cloudflare API Token có quyền Zone.DNS (Edit) và Cloudflare Tunnel"
  sensitive   = true
}

variable "cloudflare_account_id" {
  type        = string
  description = "Cloudflare Account ID (Lấy từ URL hoặc Sidebar của Cloudflare Dashboard)"
}

variable "cloudflare_zone_id" {
  type        = string
  description = "Zone ID của tên miền trong Cloudflare Dashboard"
}

variable "domain_name" {
  type        = string
  description = "Tên miền chính (vd: yourdomain.com)"
}

variable "subdomain" {
  type        = string
  default     = "game"
  description = "Subdomain để trỏ về game (vd: 'game' -> game.yourdomain.com)"
}
