#!/usr/bin/env python3
import json
import os
import sys
import time
import base64
import hashlib
import re
from urllib.request import Request, urlopen
from cryptography.hazmat.primitives import hashes, serialization
from cryptography.hazmat.primitives.asymmetric import padding
from email.utils import formatdate

PROJECT_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
TFVARS_FILE = os.path.join(PROJECT_DIR, "terraform", "terraform.tfvars")

tfvars = {}
if os.path.exists(TFVARS_FILE):
    with open(TFVARS_FILE, "r") as f:
        for line in f:
            m = re.match(r'^\s*([a-zA-Z0-9_]+)\s*=\s*"([^"]+)"', line)
            if m:
                tfvars[m.group(1)] = m.group(2)

TENANCY_OCID = tfvars.get("tenancy_ocid")
USER_OCID = tfvars.get("user_ocid")
FINGERPRINT = tfvars.get("fingerprint")
OCI_KEY_FILE = os.path.expanduser(tfvars.get("private_key_path", "~/.ssh/huynhvannhancntt_private_key_pair.pem"))
REGION = tfvars.get("region", "ap-singapore-1")
SSH_PUB_KEY_FILE = os.path.expanduser(tfvars.get("ssh_public_key_path", "~/.ssh/id_ed25519.pub"))
SSH_PRIV_KEY_FILE = os.path.expanduser("~/.ssh/id_ed25519")
BASTION_ID = "ocid1.bastion.oc1.ap-singapore-1.amaaaaaasyx6r3qahivs4paq6b5hjwjljfutx4ompjrxybtvsulokjhbv6qa"
TARGET_IP = "10.0.1.60"

def get_auth_headers(method, path, body=""):
    with open(OCI_KEY_FILE, "rb") as f:
        private_key = serialization.load_pem_private_key(f.read(), password=None)

    key_id = f"{TENANCY_OCID}/{USER_OCID}/{FINGERPRINT}"
    host = f"bastion.{REGION}.oci.oraclecloud.com"
    date = formatdate(timeval=None, localtime=False, usegmt=True)

    headers_to_sign = ["(request-target)", "date", "host"]
    signing_string = f"(request-target): {method.lower()} {path}\ndate: {date}\nhost: {host}"

    if body:
        digest = base64.b64encode(hashlib.sha256(body.encode('utf-8')).digest()).decode('utf-8')
        signing_string += f"\nx-content-sha256: {digest}\ncontent-type: application/json\ncontent-length: {len(body)}"
        headers_to_sign.extend(["x-content-sha256", "content-type", "content-length"])

    signature = private_key.sign(
        signing_string.encode('utf-8'),
        padding.PKCS1v15(),
        hashes.SHA256()
    )
    sig_b64 = base64.b64encode(signature).decode('utf-8')

    auth_header = (
        f'Signature version="1",keyId="{key_id}",algorithm="rsa-sha256",'
        f'headers="{" ".join(headers_to_sign)}",signature="{sig_b64}"'
    )

    req_headers = {
        "Date": date,
        "Host": host,
        "Authorization": auth_header,
    }
    if body:
        req_headers["x-content-sha256"] = digest
        req_headers["Content-Type"] = "application/json"
        req_headers["Content-Length"] = str(len(body))

    return req_headers

def oci_request(method, path, body=""):
    headers = get_auth_headers(method, path, body)
    url = f"https://bastion.{REGION}.oci.oraclecloud.com{path}"
    data = body.encode('utf-8') if body else None
    req = Request(url, data=data, headers=headers, method=method)
    try:
        with urlopen(req) as resp:
            return resp.read().decode('utf-8')
    except Exception as e:
        if hasattr(e, 'read'):
            print(f"Error response: {e.read().decode('utf-8')}")
        raise e

def create_bastion_session():
    with open(SSH_PUB_KEY_FILE, "r") as f:
        pub_key = f.read().strip()

    body = json.dumps({
        "bastionId": BASTION_ID,
        "displayName": f"deploy-session-{int(time.time())}",
        "keyDetails": {
            "publicKeyContent": pub_key
        },
        "targetResourceDetails": {
            "sessionType": "PORT_FORWARDING",
            "targetResourceId": None,
            "targetResourcePrivateIpAddress": TARGET_IP,
            "targetResourcePort": 22
        },
        "sessionTtlInSeconds": 3600
    })

    print("🚀 Đang yêu cầu tạo OCI Bastion Port-Forwarding Session...")
    res = oci_request("POST", "/20210331/sessions", body)
    session = json.loads(res)
    session_id = session.get("id")
    print(f"✅ Session ID: {session_id}")
    
    # Wait for session to be ACTIVE
    for _ in range(20):
        time.sleep(3)
        check_res = oci_request("GET", f"/20210331/sessions/{session_id}")
        s_data = json.loads(check_res)
        state = s_data.get("lifecycleState")
        print(f"Trạng thái session: {state}")
        if state == "ACTIVE":
            print("⏳ Đợi 8 giây để OCI Bastion phân bổ SSH Key vào daemon...")
            time.sleep(8)
            ssh_cmd = s_data.get("sshMetadata", {}).get("command", "")
            print("\n================== COMMAND TUNNEL ==================")
            print(ssh_cmd)
            print("====================================================")
            # Replace placeholder <localPort> with 2222 and absolute private key path
            tunnel_cmd = ssh_cmd.replace("<localPort>", "2222").replace("<privateKey>", SSH_PRIV_KEY_FILE)
            with open("/tmp/bastion_tunnel_cmd.txt", "w") as f_out:
                f_out.write(tunnel_cmd)
            sys.exit(0)

    print("⚠️ Timeout waiting for Bastion session")
    sys.exit(1)

if __name__ == "__main__":
    create_bastion_session()
