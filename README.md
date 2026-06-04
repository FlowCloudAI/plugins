# 官方插件仓库（plugins）

`plugins` 存放 FlowCloudAI 官方 `.fcplug` 示例源码，覆盖 DeepSeek、Qwen LLM、Qwen 图像与 Qwen TTS。  
仓库用于统一验证 `tool_fcplug` 构建链、manifest 与运行时加载一致性。

## 项目简介

各插件共享同一构建入口（`cargo fcplug build`）与一致的可复现约定。  
在新增示例或改造能力时建议先在本仓库完成构建验证，再回归 `app_main`。

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

1. 依次构建 4 个插件。  
2. 检查每个目录下 `manifest.json` 与 `plugin.wasm`。  
3. 在示例客户端进行一次端到端调用确认加载成功。  

## 主要功能 / 使用方式

- 官方 LLM、图像、TTS 示例能力。  
- `.fcplug` 构建与更新一致性验证。  
- 自定义插件开发的参考实现。  

## 技术栈

- Rust、WASM、`.fcplug`、`tool_fcplug`  

## 目录结构（仅顶层）

```text
plugins/
├── fcplug-deepseek-llm
├── fcplug-qwen_llm
├── fcplug-qwen-image
└── fcplug-qwen-tts
```

## 许可证与贡献方式

- 许可证：仓库未发现独立 `LICENSE` 文件（TODO：确认示例插件许可）。  
- 贡献前请补充构建结果、复现步骤与兼容风险。  
- 提交信息默认中文，变更范围建议按插件分组。  

文档同步时间：2026-06-04 17:03:10 +08:00
