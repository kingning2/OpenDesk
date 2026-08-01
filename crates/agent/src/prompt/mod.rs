//! 模型提示词请求结构。

/// 一次模型调用所需的提示词与采样参数。
#[derive(Debug, Clone, Copy)]
pub struct Prompt<'a> {
    /// 系统提示词。
    pub system: &'a str,
    /// 用户提示词。
    pub user: &'a str,
    /// 采样温度。
    pub temperature: f32,
}

impl<'a> Prompt<'a> {
    /// 创建使用默认温度 `0.2` 的提示词。
    pub const fn new(system: &'a str, user: &'a str) -> Self {
        Self {
            system,
            user,
            temperature: 0.2,
        }
    }

    /// 设置采样温度。
    pub const fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }
}
