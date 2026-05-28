# 官方插件仓库（plugins）

`plugins` 存放 FlowCloudAI 官方插件示例源码，覆盖 DeepSeek LLM、Qwen LLM、图像与 TTS 能力。  
仓库用于快速复现 `.fcplug` 构建与发布约定，并验证不同模型能力在桌面端的接入一致性。

## 快速开始

### 构建插件

```bash
cd plugins
cd fcplug-deepseek-llm && cargo fcplug build
cd ../fcplug-qwen_llm && cargo fcplug build
cd ../fcplug-qwen-image && cargo fcplug build
cd ../fcplug-qwen-tts && cargo fcplug build
```

### 最小示例

1. 按顺序构建 4 个子插件。  
2. 检查每个子目录是否生成 `manifest.json`、`plugin.wasm` 与 `icon` 资源。  
3. 在 `app_main` 中加载对应插件，执行一次端到端 AI 能力调用验证。

## 主要功能 / 使用方式

- 多模型能力示例：LLM、图像、TTS。  
- 统一目录约定与构建方式（`cargo fcplug build`）。  
- 可作为自定义插件开发的起点模板。  

## 技术栈

- Rust、WASM、`tool_fcplug`、`.fcplug` 插件协议。  

## 目录结构（仅顶层）

```text
plugins/
├── fcplug-deepseek-llm
├── fcplug-qwen_llm
├── fcplug-qwen-image
└── fcplug-qwen-tts
```

## 许可证与贡献方式

以仓库许可为准。  
提交前补充构建输出、测试场景与兼容风险说明。
