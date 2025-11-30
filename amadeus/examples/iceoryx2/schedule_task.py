#!/usr/bin/env python3
# Copyright (c) 2025 Amadeus Contributors
#
# SPDX-License-Identifier: MIT

"""Schedule task example for Amadeus iceoryx2 dispatcher.

This script demonstrates how an external program can schedule a task in Amadeus
by sending a specific message over Iceoryx2.
"""

import sys
import os
import glob

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
import json
import time

# Service name used by the Rust Amadeus dispatcher
SERVICE_NAME = "Amadeus/Message/Service"

def main():
    print("=== Amadeus Scheduler Test Client ===")
    print(f"Connecting to service: {SERVICE_NAME}")

    # Set up iceoryx2 node and service
    node = iox2.NodeBuilder.new().create(iox2.ServiceType.Ipc)

    # Open or create publish-subscribe service
    service = (
        node.service_builder(iox2.ServiceName.new(SERVICE_NAME))
        .publish_subscribe(AmadeusMessageData)
        .open_or_create()
    )
    publisher = service.publisher_builder().create()
    
    print(f"✅ Publisher connected to service '{SERVICE_NAME}'")

    # 1. Schedule a repeating task
    # This message tells Amadeus to:
    # - Run every 2 seconds
    # - Send a "custom.notification" message when triggered
    cron_schedule = "1/2 * * * * *"  # Every 2 seconds
    
    schedule_payload = {
        "cron": cron_schedule,
        "message": {
            "message_type": "custom.notification",
            "payload": {
                "source": "python_scheduler",
                "content": "This is a scheduled message from Python!",
                "timestamp": int(time.time())
            }
        }
    }

    print("\n📤 Sending schedule request...")
    message_data = AmadeusMessageData.from_dict(
        "system.schedule.add",  # Topic for adding schedules
        schedule_payload,
        priority=1
    )

    sample = publisher.loan_uninit()
    sample = sample.write_payload(message_data)
    sample.send()
    
    print(f"✅ Schedule request sent: {schedule_payload}")
    
    print("\nNow creating a TODO item via CoreSystem...")
    
    # 2. Create a TODO item
    todo_payload = {
        "content": "Review Python Integration",
        "tags": ["python", "urgent"],
        "priority": 1
    }
    
    message_data = AmadeusMessageData.from_dict(
        "system.memo.create",
        todo_payload,
        priority=1
    )
    
    sample = publisher.loan_uninit()
    sample = sample.write_payload(message_data)
    sample.send()
    print(f"✅ TODO creation request sent: {todo_payload}")

    print("\nTasks submitted. Check Amadeus logs for execution.")

if __name__ == "__main__":
    main()

