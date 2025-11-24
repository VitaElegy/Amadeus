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

"""Integration test for Amadeus iceoryx2 dispatcher communication.

This script tests the interaction between Python iceoryx2 clients and the
Rust Amadeus dispatcher by running both publisher and subscriber in the
same process.
"""

import threading
import time
import iceoryx2 as iox2
from amadeus_message_data import AmadeusMessageData

# Service name used by the Rust Amadeus dispatcher
SERVICE_NAME = "Amadeus/Message/Service"

def run_publisher(test_duration: int = 10):
    """Run publisher thread that sends test messages for specified duration."""
    print("🚀 Starting Python Publisher...")

    # Set up iceoryx2 with reduced logging for cleaner output
    iox2.set_log_level_from_env_or(iox2.LogLevel.Warn)
    node = iox2.NodeBuilder.new().create(iox2.ServiceType.Ipc)

    # Connect to shared service
    service = (
        node.service_builder(iox2.ServiceName.new(SERVICE_NAME))
        .publish_subscribe(AmadeusMessageData)
        .open_or_create()
    )
    publisher = service.publisher_builder().create()

    print("✅ Python Publisher connected")

    start_time = time.time()
    counter = 0

    try:
        while time.time() - start_time < test_duration:
            counter += 1

            # Create structured test message with metadata
            message_data = AmadeusMessageData.from_dict(
                "python_test",
                {
                    "message_id": counter,
                    "source": "python_integration_test",
                    "content": f"Test message #{counter} from Python publisher",
                    "timestamp": int(time.time() * 1000)
                },
                priority=1  # Normal priority
            )

            # Send via zero-copy loan pattern
            sample = publisher.loan_uninit()
            sample = sample.write_payload(message_data)
            sample.send()

            print(f"📤 Python sent: #{counter}")
            time.sleep(0.5)  # Rate limit: 2 messages per second

    except Exception as e:
        print(f"❌ Publisher error: {e}")

    print("🛑 Python Publisher finished")

def run_subscriber(test_duration: int = 10):
    """Run subscriber thread that receives and displays messages for specified duration."""
    print("👂 Starting Python Subscriber...")

    # Set up iceoryx2 with reduced logging
    iox2.set_log_level_from_env_or(iox2.LogLevel.Warn)
    node = iox2.NodeBuilder.new().create(iox2.ServiceType.Ipc)

    # Connect to shared service
    service = (
        node.service_builder(iox2.ServiceName.new(SERVICE_NAME))
        .publish_subscribe(AmadeusMessageData)
        .open_or_create()
    )
    subscriber = service.subscriber_builder().create()

    print("✅ Python Subscriber connected")

    start_time = time.time()
    message_count = 0

    try:
        while time.time() - start_time < test_duration:
            time.sleep(0.1)  # Poll interval: check every 100ms

            # Drain all available messages from queue
            while True:
                sample = subscriber.receive()
                if sample is None:
                    break  # Queue empty

                message_count += 1
                message_data = sample.payload

                # Extract and display key message details
                msg_type = message_data.get_message_type()
                priority = message_data.get_priority_name()

                print(f"📥 Python received #{message_count}: {msg_type} ({priority})")

                # Parse and display message content
                try:
                    payload_dict = message_data.to_dict()
                    content = payload_dict.get('content', 'N/A')
                    source = payload_dict.get('source', 'unknown')
                    print(f"   From: {source}")
                    print(f"   Content: {content}")
                except:
                    print("   (Could not parse content)")
    except Exception as e:
        print(f"❌ Subscriber error: {e}")

    print(f"🛑 Python Subscriber finished (received {message_count} messages)")

def main():
    print("=== Amadeus Iceoryx2 Integration Test ===")
    print("Testing Python ↔ Rust iceoryx2 communication")
    print()

    # Test duration in seconds
    TEST_DURATION = 15

    print(f"Test will run for {TEST_DURATION} seconds...")
    print("Make sure to start the Rust Amadeus dispatcher first!")
    print("Example: cargo run --example messaging")
    print()

    # Start subscriber thread
    subscriber_thread = threading.Thread(target=run_subscriber, args=(TEST_DURATION,))
    subscriber_thread.start()

    # Wait a bit for subscriber to be ready
    time.sleep(1)

    # Start publisher thread
    publisher_thread = threading.Thread(target=run_publisher, args=(TEST_DURATION,))
    publisher_thread.start()

    # Wait for both threads to complete
    publisher_thread.join()
    subscriber_thread.join()

    print()
    print("✅ Integration test completed!")
    print()
    print("Expected behavior:")
    print("- Python publisher sends messages every 500ms")
    print("- Python subscriber receives messages from both Python publisher and Rust dispatcher")
    print("- Rust dispatcher should also receive messages from Python publisher")
    print()
    print("If you see messages being sent and received, the integration is working!")

if __name__ == "__main__":
    main()
