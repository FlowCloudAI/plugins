# plugins — AGENTS.md

## 项目概览

`plugins` 是 FlowCloudAI 官方 `.fcplug` 示例仓库，聚合 DeepSeek 与 Qwen 的 LLM、图片、TTS 示例实现。  
仓库用于统一验证 manifest、WASM 构建与桌面端加载链路一致性。

## 构建 / 运行 / 测试 / lint

```bash
cd plugins
cd fcplug-deepseek-llm && cargo fcplug build
cd ../fcplug-qwen_llm && cargo fcplug build
cd ../fcplug-qwen-image && cargo fcplug build
cd ../fcplug-qwen-tts && cargo fcplug build
```

该仓库无统一 lint/test 命令，构建成功与 `manifest.json` / `plugin.wasm` 配对是最小验收点。  

## 代码风格与命名约定

- Rust 使用 2024 Edition，接口与 `tool_fcplug`、`core_ai_client` 语义对齐。  
- 插件 crate 名称与目录大小写稳定一致，避免跨平台解析问题。  
- manifest 与运行时入口需随实现变更同步更新。  

## 目录结构与职责

```text
plugins/
├── fcplug-deepseek-llm    # DeepSeek 示例
├── fcplug-qwen_llm        # Qwen LLM 示例
├── fcplug-qwen-image      # Qwen 图片示例
└── fcplug-qwen-tts        # Qwen TTS 示例
```

## 安全 / 禁止事项

- 不提交真实 API Key、签名私钥或生产端点。  
- 不在示例中写入测试口令和真实用户数据。  
- 发布前逐一核对 `manifest.json` 与 `plugin.wasm` 一一对应。  

## 提交与 PR 规范

- 提交信息默认中文，变更按单仓库或单能力分组。  
- PR 说明需写清每个插件的构建结果、兼容性与回归结论。  
- 目录名或大小写变更需补充跨平台影响说明。  

## 项目特有坑点

- `fcplug-qwen_llm` 与 `fcplug-qwen-tts` 对大小写敏感，跨平台需同步验证。  
- Linux/macOS 严格大小写环境下路径差异更容易触发加载失败。  
- manifest 与 runtime 入口不一致会导致客户端初始化错误。  

文档同步时间：2026-06-08 13:20:10 +08:00
