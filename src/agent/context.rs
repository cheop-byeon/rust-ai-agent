use std::collections::HashMap;

use serde_json::Value;
use uuid::Uuid;

use super::event::Event;

#[derive(Debug)]
/// Agent 执行期间所有状态的中央存储
/// 所有方法（think / act / build_messages ...）都只接收这一个结构体，
/// 不用再拼一堆零散参数。
pub struct ExecutionContext {
    pub execution_id: String,
    pub events: Vec<Event>,
    pub current_step: u32,
    pub state: HashMap<String, Value>,
    pub final_result: Option<String>,
}

impl ExecutionContext {
    pub fn new() -> Self {
        Self {
            execution_id: Uuid::new_v4().to_string(),
            events: Vec::new(),
            current_step: 0,
            state: HashMap::new(),
            final_result: None,
        }
    }

    /// 把一条 Event 追加进执行历史。
    pub fn add_event(&mut self, event: Event) {
        self.events.push(event);
    }

    /// 往下推进一步，对应完成一轮 think-act 循环。
    pub fn increment_step(&mut self) {
        self.current_step += 1;
    }
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self::new()
    }
}