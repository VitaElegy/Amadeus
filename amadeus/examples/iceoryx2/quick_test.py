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

"""Quick test to verify iceoryx2 Python binding installation."""

import iceoryx2
from amadeus_message_data import AmadeusMessageData

def main():
    print("🧪 Testing iceoryx2 Python binding...")

    # Test basic import
    print("✅ iceoryx2 imported successfully")

    # Test version info
    try:
        # Try to get version (might not be available)
        version = getattr(iceoryx2, '__version__', 'unknown')
        print(f"📦 iceoryx2 version: {version}")
    except:
        print("📦 iceoryx2 version: unknown")

    # Test node creation
    node = iceoryx2.NodeBuilder.new().create(iceoryx2.ServiceType.Ipc)
    print("✅ Node created successfully")

    # Test AmadeusMessageData
    message = AmadeusMessageData.from_dict(
        "test",
        {"message": "Hello from Python!", "test": True},
        priority=1
    )
    print("✅ AmadeusMessageData created successfully")
    print(f"📄 Sample message: {message}")

    # Test service creation (without connecting to Rust app)
    service = (
        node.service_builder(iceoryx2.ServiceName.new("Test/Service"))
        .publish_subscribe(AmadeusMessageData)
        .open_or_create()
    )
    print("✅ Service created successfully")

    # Test publisher creation
    publisher = service.publisher_builder().create()
    print("✅ Publisher created successfully")

    # Test subscriber creation
    subscriber = service.subscriber_builder().create()
    print("✅ Subscriber created successfully")

    print("\n🎉 All tests passed! iceoryx2 Python binding is working correctly.")
    print("\n💡 Next steps:")
    print("1. Start the Rust Amadeus application: cargo run --example messaging")
    print("2. Run publisher in one terminal: python3 publisher.py")
    print("3. Run subscriber in another terminal: python3 subscriber.py")
    print("4. Or run integration test: python3 test_integration.py")

if __name__ == "__main__":
    main()
