#!/usr/bin/env python3
"""
OCI Bastion Helper: Creates a temporary Port-Forwarding SSH tunnel to private VM 10.0.1.60.
"""
import subprocess, os, sys, time, base64, urllib.request, ssl, json, hashlib, re

PROJECT_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
TFVARS_FILE = os.path.join(PROJECT_DIR, "terraform", "terraform.tfvars")

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

bastion_id = "ocid1.bastion.oc1.ap-singapore-1.amaaaaaasyx6r3qahivs4paq6b5hjwjljfutx4ompjrxybtvsulokjhbv6qa"
instance_id = "ocid1.instance.oc1.ap-singapore-1.anzwsljrsyx6r3qcpglovknffch5gg56fq5mr5gyaa6wqmdx5wvfvr75zwaq"

def oci_request(method, path, body=None):
    host = f"bastion.{region}.oci.oraclecloud.com"
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
    with open(os.path.expanduser("~/.ssh/id_ed25519.pub")) as f:
        pub_key = f.read().strip()

    body = {
        "bastionId": bastion_id,
        "displayName": f"deploy-{int(time.time())}",
        "keyType": "PUB",
        "targetResourceDetails": {
            "sessionType": "PORT_FORWARDING",
            "targetResourceId": instance_id,
            "targetResourcePrivateIpAddress": "10.0.1.60",
            "targetResourcePort": 22
        },
        "keyDetails": {
            "publicKeyContent": pub_key
        },
        "sessionTtlInSeconds": 3600
    }

    print("🚀 Đang yêu cầu tạo OCI Bastion Port-Forwarding Session...")
    res = oci_request("POST", "/20210331/sessions", body)
    session = json.loads(res)
    session_id = session.get("id")
    print(f"✅ Session ID: {session_id}")
    
    # Wait for session to be ACTIVE
    for _ in range(15):
        time.sleep(3)
        check_res = oci_request("GET", f"/20210331/sessions/{session_id}")
        s_data = json.loads(check_res)
        state = s_data.get("lifecycleState")
        print(f"Trạng thái session: {state}")
        if state == "ACTIVE":
            ssh_cmd = s_data.get("sshMetadata", {}).get("command", "")
            print("\n================== COMMAND TUNNEL ==================")
            print(ssh_cmd)
            print("====================================================")
            # Replace placeholder <localPort> with 2222
            tunnel_cmd = ssh_cmd.replace("<localPort>", "2222").replace("<privateKey>", "~/.ssh/id_ed25519")
            with open("/tmp/bastion_tunnel_cmd.txt", "w") as f_out:
                f_out.write(tunnel_cmd)
            sys.exit(0)

    print("⚠️ Timeout waiting for Bastion session")
    sys.exit(1)
