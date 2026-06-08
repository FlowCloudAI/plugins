# 官方插件仓库（plugins）

## 项目简介

`plugins` 存放 FlowCloudAI 官方 `.fcplug` 示例源码，覆盖 DeepSeek、Qwen LLM、Qwen 图片与 Qwen TTS。  
仓库用于统一校验 `tool_fcplug` 构建链路与客户端兼容行为。

## 快速开始

### 构建全部插件

```bash
cd plugins
cd fcplug-deepseek-llm && cargo fcplug build
cd ../fcplug-qwen_llm && cargo fcplug build
cd ../fcplug-qwen-image && cargo fcplug build
cd ../fcplug-qwen-tts && cargo fcplug build
```

### 最小示例

1. 依次构建四个插件。  
2. 检查 `manifest.json` 与 `plugin.wasm` 是否一一对应。  
3. 在示例客户端执行一次端到端调用验证。  

## 主要功能 / 使用方式

- 官方 LLM、图片、TTS 示例能力。  
- `.fcplug` 构建与兼容性校验。  
- 自定义插件开发对齐参考基线。  

## 技术栈

- Rust + WASM + `.fcplug` + `tool_fcplug`

## 目录结构（仅顶层）

```text
plugins/
├── fcplug-deepseek-llm
├── fcplug-qwen_llm
├── fcplug-qwen-image
└── fcplug-qwen-tts
```

## 许可证与贡献方式

- 许可证：本仓库未发现独立 `LICENSE`，按仓库当前授权策略执行。  
- PR 建议补充 `cargo fcplug build` 结果与复现步骤。  
- 兼容性改动需说明 manifest 变更与加载策略。  

文档同步时间：2026-06-08 13:20:10 +08:00
