# 官方插件仓库（plugins）

## 项目简介

`plugins` 存放 FlowCloudAI 官方 `.fcplug` 示例源码，覆盖 DeepSeek、千问、火山方舟、智谱、MiniMax 与 Moonshot。
仓库用于统一校验 `tool_fcplug` 构建链路与客户端兼容行为。

## 快速开始

### 构建全部插件

```powershell
Get-ChildItem -Directory -Filter "fcplug-*" | ForEach-Object {
    Push-Location $_.FullName
    cargo fcplug build
    Pop-Location
}
```

### 最小示例

1. 依次构建所需插件。
2. 检查 `manifest.json` 与 `plugin.wasm` 是否一一对应。  
3. 在示例客户端执行一次端到端调用验证。  

## 主要功能 / 使用方式

- 官方 LLM、图片、TTS 示例能力。  
- `.fcplug` 构建与兼容性校验。  
- 自定义插件开发对齐参考基线。  

## 技术栈

- Rust + WASM + `.fcplug` + `tool_fcplug`

## 目录结构（仅顶层）

| 厂商 | LLM | 图片 | TTS |
| --- | --- | --- | --- |
| DeepSeek | `fcplug-deepseek-llm` | — | — |
| 千问 | `fcplug-qwen_llm` | `fcplug-qwen-image` | `fcplug-qwen-tts` |
| 火山方舟 | `fcplug-volcengine-llm` | `fcplug-volcengine-image` | — |
| 智谱 | `fcplug-zhipu-llm` | `fcplug-zhipu-image` | `fcplug-zhipu-tts` |
| MiniMax | `fcplug-minimax-llm` | `fcplug-minimax-image` | `fcplug-minimax-tts` |
| Moonshot | `fcplug-moonshot-llm` | — | — |

火山方舟 TTS 需要额外的应用凭证与非 Bearer 鉴权头，当前插件协议和统一 HTTP 客户端无法仅靠 mapper 安全接入，因此暂未提供。

## 许可证与贡献方式

- 许可证：MIT，详见根目录 `LICENSE`。
- PR 建议补充 `cargo fcplug build` 结果与复现步骤。  
- 兼容性改动需说明 manifest 变更与加载策略。  

文档同步时间：2026-07-11 +08:00
