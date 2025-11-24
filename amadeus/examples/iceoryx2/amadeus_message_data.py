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

"""Amadeus message data type for iceoryx2 Python binding."""

import ctypes
import json
import time


class AmadeusMessageData(ctypes.Structure):
    """The strongly typed payload type for Amadeus messages.

    This structure matches the Rust AmadeusMessageData struct used in the
    Amadeus system for zero-copy IPC communication.
    """

    _fields_ = [
        # 消息类型（最大长度 64 字符）
        ("message_type", ctypes.c_char * 64),
        # 消息类型实际长度
        ("message_type_len", ctypes.c_uint8),
        # 消息 JSON 数据（最大 4096 字节）
        ("json_data", ctypes.c_char * 4096),
        # JSON 数据实际长度
        ("json_data_len", ctypes.c_uint16),
        # 优先级（0=Low, 1=Normal, 2=High, 3=Critical）
        ("priority", ctypes.c_uint8),
        # 时间戳（Unix 时间戳，毫秒）
        ("timestamp", ctypes.c_uint64),
    ]

    def __init__(
        self,
        message_type: str = "",
        json_payload: str = "{}",
        priority: int = 1,
        timestamp: int = None
    ):
        """Initialize AmadeusMessageData with validation and padding."""
        super().__init__()

        # Auto-generate timestamp if not provided
        if timestamp is None:
            timestamp = int(time.time() * 1000)

        # Encode and validate message type (max 64 bytes)
        msg_type_bytes = message_type.encode('utf-8')
        if len(msg_type_bytes) > 64:
            raise ValueError("Message type too long (max 64 bytes)")
        self.message_type = msg_type_bytes.ljust(64, b'\x00')  # Pad with null bytes
        self.message_type_len = len(msg_type_bytes)

        # Encode and validate JSON payload (max 4096 bytes)
        json_bytes = json_payload.encode('utf-8')
        if len(json_bytes) > 4096:
            raise ValueError("JSON payload too long (max 4096 bytes)")
        self.json_data = json_bytes.ljust(4096, b'\x00')  # Pad with null bytes
        self.json_data_len = len(json_bytes)

        self.priority = priority
        self.timestamp = timestamp

    def get_message_type(self) -> str:
        """Extract message type string from padded buffer."""
        return self.message_type[:self.message_type_len].decode('utf-8')

    def get_json_payload(self) -> str:
        """Extract JSON payload string from padded buffer."""
        return self.json_data[:self.json_data_len].decode('utf-8')

    def get_priority_name(self) -> str:
        """Convert priority number to human-readable name."""
        priorities = ["Low", "Normal", "High", "Critical"]
        return priorities[self.priority] if 0 <= self.priority < len(priorities) else f"Unknown({self.priority})"

    def __str__(self) -> str:
        """Returns human-readable string of the contents."""
        try:
            message_type = self.get_message_type()
            json_payload = self.get_json_payload()
            priority_name = self.get_priority_name()

            return (f"AmadeusMessageData {{ "
                    f"timestamp: {self.timestamp}, "
                    f"message_type='{message_type}', "
                    f"priority={priority_name}, "
                    f"json_payload='{json_payload}'}}")
        except Exception as e:
            return f"AmadeusMessageData {{ error: {e} }}"

    @staticmethod
    def type_name() -> str:
        """Returns the system-wide unique type name required for communication."""
        return "AmadeusMessage"

    @classmethod
    def from_dict(cls, message_type: str, data: dict, priority: int = 1) -> 'AmadeusMessageData':
        """Create AmadeusMessageData from a dictionary.

        Args:
            message_type: The message type
            data: Dictionary to convert to JSON
            priority: Priority level

        Returns:
            AmadeusMessageData instance
        """
        json_payload = json.dumps(data, ensure_ascii=False)
        return cls(message_type=message_type, json_payload=json_payload, priority=priority)

    def to_dict(self) -> dict:
        """Convert the JSON payload back to a dictionary.

        Returns:
            Dictionary representation of the JSON payload
        """
        try:
            return json.loads(self.get_json_payload())
        except json.JSONDecodeError:
            return {"error": "Invalid JSON payload"}
