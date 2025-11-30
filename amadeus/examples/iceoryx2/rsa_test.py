#!/usr/bin/env python3
# Copyright (c) 2025 Amadeus Contributors
# SPDX-License-Identifier: MIT

"""RSA Encryption Test for Amadeus iceoryx2 dispatcher.

This script demonstrates how an external program can receive encrypted messages
from Amadeus by providing an RSA public key.
"""

import sys
import os
import glob
import time
import base64
import json

# Try to import iceoryx2, if failing, try to find it in the project venv
try:
    import iceoryx2 as iox2
except ImportError:
    # Try to find the venv created by run_test.sh
    script_dir = os.path.dirname(os.path.abspath(__file__))
    project_root = os.path.abspath(os.path.join(script_dir, "../../.."))
    venv_path = os.path.join(project_root, "iceoryx2/iceoryx2-ffi/python/venv/lib/python3.*/site-packages")
    
    found_paths = glob.glob(venv_path)
    if found_paths:
        import site
        site.addsitedir(found_paths[0])
        try:
            import iceoryx2 as iox2
        except ImportError:
            print("❌ Error: Could not import iceoryx2 even after adding venv to path.")
            print("Please run ./run_test.sh to build and install the bindings first.")
            sys.exit(1)
    else:
        print("❌ Error: iceoryx2 module not found.")
        print("Please run ./run_test.sh to build and install the bindings first.")
        sys.exit(1)

from amadeus_message_data import AmadeusMessageData
from cryptography.hazmat.primitives import serialization, hashes
from cryptography.hazmat.primitives.asymmetric import rsa, padding
from cryptography.hazmat.primitives.ciphers.aead import AESGCM
from cryptography.hazmat.backends import default_backend

SERVICE_NAME = "Amadeus/Message/Service"
CYCLE_TIME = iox2.Duration.from_millis(100)


def generate_keys():
    """Generate RSA key pair and save public key to file."""
    print("🔑 Generating RSA Key Pair...")
    private_key = rsa.generate_private_key(
        public_exponent=65537,
        key_size=2048,
        backend=default_backend()
    )
    
    public_key = private_key.public_key()
    
    pem = public_key.public_bytes(
        encoding=serialization.Encoding.PEM,
        format=serialization.PublicFormat.SubjectPublicKeyInfo
    )
    
    with open("public_key.pem", "wb") as f:
        f.write(pem)
        
    print("✅ Public key saved to 'public_key.pem'")
    return private_key

def main():
    print("=== Amadeus RSA Encryption Test ===")
    
    # 1. Generate Keys
    private_key = generate_keys()
    
    print(f"\n🚀 Connecting to service: {SERVICE_NAME}")
    
    iox2.set_log_level_from_env_or(iox2.LogLevel.Warn)
    node = iox2.NodeBuilder.new().create(iox2.ServiceType.Ipc)
    
    service = (
        node.service_builder(iox2.ServiceName.new(SERVICE_NAME))
        .publish_subscribe(AmadeusMessageData)
        .open_or_create()
    )
    subscriber = service.subscriber_builder().create()
    
    print("👂 Listening for encrypted messages...")
    print("   (Ensure the Rust application is running with 'public_key.pem' configured)")
    
    while True:
        try:
            node.wait(CYCLE_TIME)
            while True:
                sample = subscriber.receive()
                if sample is None:
                    break
                
                message_data = sample.payload
                try:
                    payload_dict = message_data.to_dict()
                    
                    if "secure_key" in payload_dict and "secure_payload" in payload_dict:
                        print(f"\n🔐 Received Hybrid Encrypted Message!")
                        
                        try:
                            # 1. Decode fields
                            encrypted_key = base64.b64decode(payload_dict["secure_key"])
                            iv = base64.b64decode(payload_dict["iv"])
                            encrypted_payload = base64.b64decode(payload_dict["secure_payload"])
                            
                            # 2. Decrypt AES Key with RSA
                            aes_key = private_key.decrypt(
                                encrypted_key,
                                padding.PKCS1v15()
                            )
                            
                            # 3. Decrypt Payload with AES-GCM
                            aesgcm = AESGCM(aes_key)
                            decrypted_data = aesgcm.decrypt(iv, encrypted_payload, None)
                            
                            decrypted_json = json.loads(decrypted_data.decode('utf-8'))
                            print(f"🔓 Decrypted Content: {json.dumps(decrypted_json, indent=2, ensure_ascii=False)}")
                        except Exception as e:
                            print(f"❌ Decryption Failed: {e}")
                    elif "secure_payload" in payload_dict:
                         # Old RSA-only fallback (though Rust side changed, good to keep logic safe)
                        print(f"\n🔐 Received Legacy RSA Message!")
                        encrypted_b64 = payload_dict["secure_payload"]
                        encrypted_bytes = base64.b64decode(encrypted_b64)
                        
                        try:
                            decrypted_data = private_key.decrypt(
                                encrypted_bytes,
                                padding.PKCS1v15()
                            )
                            decrypted_json = json.loads(decrypted_data.decode('utf-8'))
                            print(f"🔓 Decrypted Content: {json.dumps(decrypted_json, indent=2, ensure_ascii=False)}")
                        except Exception as e:
                            print(f"❌ Decryption Failed: {e}")
                    else:
                        print(f"📨 Received Plaintext Message: {payload_dict}")
                        
                except Exception as e:
                    print(f"⚠️ Error processing message: {e}")
                    
        except KeyboardInterrupt:
            print("\n🛑 Stopped by user")
            break

if __name__ == "__main__":
    main()

