#!/usr/bin/env python3
"""
Helper script: Unassign & delete ephemeral public IP from OCI compute instance.
Reads configuration automatically from terraform/terraform.tfvars.
"""
import subprocess, os, sys, time, base64, urllib.request, ssl, json, hashlib, re

PROJECT_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
TFVARS_FILE = os.path.join(PROJECT_DIR, "terraform", "terraform.tfvars")

if not os.path.exists(TFVARS_FILE):
    print(f"❌ Error: {TFVARS_FILE} not found.")
    sys.exit(1)

# Parse tfvars
tfvars = {}
with open(TFVARS_FILE, "r") as f:
    for line in f:
        m = re.match(r'^\s*([a-zA-Z0-9_]+)\s*=\s*"([^"]+)"', line)
        if m:
            tfvars[m.group(1)] = m.group(2)

tenancy_id = tfvars.get("tenancy_ocid")
user_id = tfvars.get("user_ocid")
fingerprint = tfvars.get("fingerprint")
key_path = os.path.expanduser(tfvars.get("private_key_path", ""))
region = tfvars.get("region", "ap-singapore-1")

def oci_request(method, path, body=None):
    host = f"iaas.{region}.oraclecloud.com"
    date_str = time.strftime("%a, %d %b %Y %H:%M:%S GMT", time.gmtime())
    headers_list = "(request-target) host date"
    signing_string = f"(request-target): {method.lower()} {path}\nhost: {host}\ndate: {date_str}"
    
    body_bytes = None
    sha_b64 = None
    if body is not None:
        body_bytes = json.dumps(body).encode("utf-8")
        sha = hashlib.sha256(body_bytes).digest()
        sha_b64 = base64.b64encode(sha).decode("utf-8")
        headers_list += " x-content-sha256 content-type content-length"
        signing_string += f"\nx-content-sha256: {sha_b64}\ncontent-type: application/json\ncontent-length: {len(body_bytes)}"

    proc = subprocess.Popen(["openssl", "dgst", "-sha256", "-sign", key_path],
                            stdin=subprocess.PIPE, stdout=subprocess.PIPE)
    sig_bytes, _ = proc.communicate(signing_string.encode("utf-8"))
    sig_b64 = base64.b64encode(sig_bytes).decode("utf-8")
    
    key_id = f"{tenancy_id}/{user_id}/{fingerprint}"
    auth_header = (f'Signature version="1",keyId="{key_id}",algorithm="rsa-sha256",'
                   f'headers="{headers_list}",signature="{sig_b64}"')
    
    url = f"https://{host}{path}"
    req = urllib.request.Request(url, data=body_bytes, method=method)
    req.add_header("host", host)
    req.add_header("date", date_str)
    req.add_header("authorization", auth_header)
    if body_bytes is not None:
        req.add_header("content-type", "application/json")
        req.add_header("x-content-sha256", sha_b64)
    
    ctx = ssl.create_default_context()
    with urllib.request.urlopen(req, context=ctx) as resp:
        return resp.read().decode("utf-8")

if __name__ == "__main__":
    print(f"🔍 Checking public IPs in compartment: {tenancy_id[:30]}...")
    res = oci_request("GET", f"/20160918/publicIps?compartmentId={tenancy_id}&scope=REGION")
    public_ips = json.loads(res)
    print(f"Found {len(public_ips)} region-level public IPs.")
