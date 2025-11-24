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

"""Publisher example for testing Amadeus iceoryx2 dispatcher.

This script sends messages to the Amadeus iceoryx2 service that the Rust
dispatcher should receive and process.
"""

import iceoryx2 as iox2
from amadeus_message_data import AmadeusMessageData

# Service name used by the Rust Amadeus dispatcher
SERVICE_NAME = "Amadeus/Message/Service"

# Cycle time between messages
CYCLE_TIME = iox2.Duration.from_millis(1000)

def main():
    """Main publisher function - sends test messages to Amadeus service."""
    print("=== Amadeus Iceoryx2 Publisher Test ===")
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
    publisher = service.publisher_builder().create()

    print(f"✅ Publisher connected to service '{SERVICE_NAME}'")
    print("🚀 Starting to send messages...\n")

    counter = 0
    try:
        while True:
            counter += 1

            # Rotate through different message types for testing
            if counter % 3 == 1:
                # Create notification message
                message_data = AmadeusMessageData.from_dict(
                    "notification",
                    {
                        "title": f"Test Notification #{counter}",
                        "message": f"This is a test notification from Python publisher (#{counter})",
                        "level": "info"
                    },
                    priority=1  # Normal priority
                )
            elif counter % 3 == 2:
                # Create alert message with higher priority
                message_data = AmadeusMessageData.from_dict(
                    "alert",
                    {
                        "type": "system",
                        "severity": "warning",
                        "description": f"System alert from Python publisher (#{counter})",
                    },
                    priority=2  # High priority
                )
            else:
                # Create custom event with metadata
                message_data = AmadeusMessageData.from_dict(
                    "custom_event",
                    {
                        "event_id": counter,
                        "source": "python_test_publisher",
                        "data": {
                            "counter": counter,
                            "status": "active",
                            "metadata": {
                                "version": "1.0",
                                "publisher": "python"
                            }
                        }
                    },
                    priority=0  # Low priority
                )

            # Send message using zero-copy loan pattern
            sample = publisher.loan_uninit()
            sample = sample.write_payload(message_data)
            sample.send()

            print(f"📤 Sent message #{counter}: {message_data}")

            # Rate limiting: wait before next message
            node.wait(CYCLE_TIME)

    except KeyboardInterrupt:
        print("\n🛑 Publisher stopped by user")
    except iox2.NodeWaitFailure:
        print("\n❌ Node wait failure - exiting")
    except Exception as e:
        print(f"\n❌ Error: {e}")
        return 1

    print("\n✅ Publisher finished")
    return 0

if __name__ == "__main__":
    exit(main())
