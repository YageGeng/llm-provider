# ZeroClaw Agent Loop 实现分析

## 项目概述

ZeroClaw 是一个极简、高性能的 Rust AI Agent 运行时，核心理念是 **零开销、零妥协**。它能在 $10 硬件上运行，仅需 <5MB RAM，启动时间 <10ms。

## 核心架构

### Agent 结构

```rust
pub struct Agent {
    provider: Box<dyn Provider>,        // LLM 提供商
    tools: Vec<Box<dyn Tool>>,          // 工具列表
    tool_specs: Vec<ToolSpec>,          // 工具规格
    memory: Arc<dyn Memory>,            // 记忆系统
    observer: Arc<dyn Observer>,        // 可观测性
    prompt_builder: SystemPromptBuilder, // 系统提示构建器
    tool_dispatcher: Box<dyn ToolDispatcher>, // 工具分发器
    memory_loader: Box<dyn MemoryLoader>,     // 记忆加载器
    config: AgentConfig,                // 配置
    model_name: String,                 // 模型名称
    temperature: f64,                   // 温度参数
    workspace_dir: PathBuf,             // 工作区目录
    identity_config: IdentityConfig,    // 身份配置
    skills: Vec<Skill>,                 // 技能列表
    history: Vec<ConversationMessage>,  // 对话历史
}
```

### Trait 驱动设计

ZeroClaw 使用 **Trait 驱动架构**，所有核心组件均为 trait 实现：

| Trait | 位置 | 职责 |
|-------|------|------|
| Provider | src/providers/ | LLM 提供商抽象（OpenAI, Anthropic 等） |
| Tool | src/tools/ | 工具执行抽象 |
| Memory | src/memory/ | 持久化记忆（SQLite FTS5） |
| Observer | src/observability/ | 可观测性 |
| ToolDispatcher | src/agent/dispatcher.rs | 工具调用分发 |
| Channel | src/channels/ | 通道抽象 |

## Agent Loop 实现

### 核心流程

ZeroClaw 的 Agent Loop 位于 src/agent/loop_.rs，采用经典的 LLM -> Tool -> Repeat 模式：

1. 接收用户消息
2. 构建上下文（加载相关记忆）
3. 构建系统提示
4. 过滤工具集
5. 调用 LLM
6. 解析响应
7. 如果有工具调用：
   - 解析工具调用（支持多种格式）
   - 执行工具
   - 清理凭证
   - 返回步骤 5
8. 提取文本响应
9. 保存历史
10. 自动保存记忆
11. 返回响应

### 关键实现细节

#### 1. 工具调用解析（多格式支持）

ZeroClaw 支持多种 LLM 输出格式的工具调用解析，这是其最大特色：

- **OpenAI 原生格式**：标准 tool_calls JSON 字段
- **XML 格式**：<tool_call>、<toolcall>、<tool-call> 等
- **MiniMax XML 格式**：invoke/parameter 标签
- **Perl/Hash-Ref 格式**：TOOL_CALL {...} /TOOL_CALL
- **FunctionCall 格式**：<FunctionCall> 块
- **GLM 格式**：简化的 tool_name/param>value 格式

#### 2. 凭证清理

自动从工具输出中清理敏感信息：

```rust
fn scrub_credentials(input: &str) -> String {
    // 使用正则表达式匹配敏感模式
    // token, api_key, password, secret, bearer, credential
    // 保留前 4 个字符，其余替换为 [REDACTED]
}
```

#### 3. 上下文压缩

当对话历史过长时自动压缩：

- 触发条件：消息数量超过 max_history 或 token 数超过 max_context_tokens
- 保留最近的消息和系统提示
- 使用 LLM 生成摘要替代旧消息
- 保持用户轮次边界的完整性

#### 4. 记忆管理

- 加载相关记忆构建上下文
- 自动保存重要消息到记忆
- 支持 FTS5 全文搜索
- 过滤低相关性记忆

#### 5. MCP 工具过滤

支持按组过滤 MCP 工具：

- always 组：始终包含
- dynamic 组：根据用户消息关键词动态包含
- 减少不必要的工具上下文

## 总结

ZeroClaw 的 Agent Loop 设计哲学是极简高效：

| 特点 | 实现方式 |
|------|----------|
| 极简 | 单一循环，无后台任务 |
| 高效 | Trait 驱动，零开销抽象 |
| 兼容 | 多格式工具调用解析 |
| 安全 | 凭证自动清理 |
| 持久 | SQLite FTS5 记忆系统 |

其核心优势在于多 LLM 格式兼容性和极致的资源效率，适合边缘计算和资源受限场景。
