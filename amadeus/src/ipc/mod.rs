// IPC 模块 - 进程间通信接口定义
// 此模块定义了与 iceoryx2 交互的数据结构和接口，保持与核心系统的解耦

// 确保 proc-macro 可用（proc-macro 需要在 crate root 或通过 extern crate 声明）
#[allow(unused_imports)]
use iceoryx2::prelude::*;

pub mod iceoryx2_types;
pub mod iceoryx2_receiver;

pub use iceoryx2_types::*;
pub use iceoryx2_receiver::*;

