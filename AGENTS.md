# plugins — AGENTS.md

## 项目概览

`plugins` 是官方示例插件仓库，聚合 LLM、图像与 TTS 的 `.fcplug` 示例实现。  
仓库用于统一验证 manifest、WASM 构建和客户端加载链路的兼容性。

## 构建 / 运行 / 测试 / lint

```bash
cd plugins
cd fcplug-deepseek-llm && cargo fcplug build
cd ../fcplug-qwen_llm && cargo fcplug build
cd ../fcplug-qwen-image && cargo fcplug build
cd ../fcplug-qwen-tts && cargo fcplug build
```

`plugins` 本身不提供统一 lint 命令，以上构建产物为最小一致性验收。  

## 代码风格与命名约定

- Rust 2024，接口与 `tool_fcplug`、`core_ai_client` 保持语义一致。  
- 插件目录与 crate 名称大小写稳定，避免跨平台解析问题。  
- manifest 与运行时入口需与仓库文档同步定义。  

## 目录结构与职责

```text
plugins/
├── fcplug-deepseek-llm    # DeepSeek 示例插件
├── fcplug-qwen_llm        # Qwen LLM 示例插件
├── fcplug-qwen-image      # Qwen 图像示例插件
└── fcplug-qwen-tts        # Qwen TTS 示例插件
```

## 安全 / 禁止事项

- 不提交真实 API Key、签名私钥或生产端点。  
- 不在示例中写入测试口令和真实用户数据。  
- 发布前比对 `manifest.json` 与 `plugin.wasm` 一一对应。  

## 提交与 PR 规范

- 提交信息默认中文，单次提交聚焦单个插件或能力组。  
- PR 说明应写明每个插件的构建结果、兼容性和失败排查。  
- 修改目录名或大小写需补充跨平台影响说明。  

## 项目特有坑点

- `fcplug-qwen_llm` 与 `fcplug-qwen-tts` 对目录名大小写敏感。  
- Linux/macOS 严格大小写环境下的路径差异会导致加载失败。  
- manifest 与 runtime 入口不一致会导致客户端初始化错误。  

## 文档同步依据（本次核对）

- 同步时间：2026-06-03 21:04:46 +08:00
- 依据文件：`plugins/fcplug-deepseek-llm/Cargo.toml`、`plugins/fcplug-qwen_llm/Cargo.toml`、`plugins/fcplug-qwen-image/Cargo.toml`、`plugins/fcplug-qwen-tts/Cargo.toml`、`tool_fcplug/Cargo.toml`
