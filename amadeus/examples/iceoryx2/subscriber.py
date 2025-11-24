#!/usr/bin/env python3
# Copyright (c) 2025 Contributors to the Eclipse Foundation
#
# See the NOTICE file(s) distributed with this work for additional
# information regarding copyright ownership.
#
# This program and the accompanying materials are made available under the
# terms of the Apache Software License 2.0 which is available at
# https://www.apache.org/licenses/LICENSE-2.0, or the MIT license
# which is available at https://opensource.org/licenses/MIT.
#
# SPDX-License-Identifier: Apache-2.0 OR MIT

"""Subscriber example for testing Amadeus iceoryx2 dispatcher.

This script receives messages from the Amadeus iceoryx2 service that the Rust
dispatcher publishes.
"""

import iceoryx2 as iox2
from amadeus_message_data import AmadeusMessageData

# Service name used by the Rust Amadeus dispatcher
SERVICE_NAME = "Amadeus/Message/Service"

# Cycle time for receiving messages
CYCLE_TIME = iox2.Duration.from_millis(100)

def main():
    """Main subscriber function - receives and displays messages from Amadeus service."""
    print("=== Amadeus Iceoryx2 Subscriber Test ===")
    print(f"Connecting to service: {SERVICE_NAME}")
    print("Press Ctrl+C to stop\n")

    # Set up iceoryx2 node and service
    iox2.set_log_level_from_env_or(iox2.LogLevel.Info)
    node = iox2.NodeBuilder.new().create(iox2.ServiceType.Ipc)

    # Open or create publish-subscribe service
    service = (
        node.service_builder(iox2.ServiceName.new(SERVICE_NAME))
        .publish_subscribe(AmadeusMessageData)
        .open_or_create()
    )
    subscriber = service.subscriber_builder().create()

    print(f"✅ Subscriber connected to service '{SERVICE_NAME}'")
    print("👂 Listening for messages...\n")

    message_count = 0

    try:
        while True:
            # Poll for messages at regular intervals
            node.wait(CYCLE_TIME)

            # Process all available messages in queue
            while True:
                sample = subscriber.receive()
                if sample is None:
                    break  # No more messages available

                message_count += 1
                message_data = sample.payload

                print(f"📥 Received message #{message_count}:")
                print(f"   {message_data}")

                # Parse and pretty-print JSON payload
                try:
                    import json
                    payload_dict = message_data.to_dict()
                    print(f"   📋 Content: {json.dumps(payload_dict, indent=2, ensure_ascii=False)}")
                except Exception as e:
                    print(f"   ⚠️  Could not parse JSON: {e}")

                print()

    except KeyboardInterrupt:
        print(f"\n🛑 Subscriber stopped by user (received {message_count} messages)")
    except iox2.NodeWaitFailure:
        print(f"\n❌ Node wait failure - exiting (received {message_count} messages)")
    except Exception as e:
        print(f"\n❌ Error: {e}")
        return 1

    print(f"\n✅ Subscriber finished (received {message_count} messages)")
    return 0

if __name__ == "__main__":
    exit(main())
