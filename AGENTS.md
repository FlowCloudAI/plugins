# plugins — AGENTS.md

## 项目概览

`plugins` 是官方示例插件仓库，聚合 LLM、图像与语音能力的 `.fcplug` 工程。  
每个子目录是一个独立可构建的插件项目，并共享统一产物与协议约束。

## 构建 / 运行 / 测试 / lint

```bash
cd plugins
cd fcplug-deepseek-llm && cargo fcplug build
cd ../fcplug-qwen_llm && cargo fcplug build
cd ../fcplug-qwen-image && cargo fcplug build
cd ../fcplug-qwen-tts && cargo fcplug build
```

`plugins` 不包含统一 lint 命令；每个插件以自身构建成功与产物一致性作为最小校验。

## 代码风格与命名约定

- 与 `tool_fcplug` 协议对齐，入口与能力命名保持稳定。  
- 文件名、目录名严格按仓库约定（包含大小写）处理。  
- WIT/WASM 接口变更需同步更新映射与版本说明。  

## 目录结构与职责

```text
plugins/
├── fcplug-deepseek-llm
├── fcplug-qwen_llm
├── fcplug-qwen-image
└── fcplug-qwen-tts
```

## 安全 / 禁止事项

- 不在插件代码中提交真实 API Key、签名密钥或私钥。  
- 产物 `manifest.json` 与 `plugin.wasm` 字段必须与源码一致。  
- 任何命名大小写变更要先在 Linux/macOS 上验证。

## 贡献方式与 PR 规范

- 更新插件时同步记录能力模型、配置参数与兼容影响。  
- PR 说明需含构建输出和接口行为变化。  
- 提交信息默认中文。

## 项目特有坑点

- Linux/macOS 对路径大小写严格敏感，`fcplug-qwen_llm` 与 `fcplug-qwen-tts` 不可改名。  
- 产物验证需逐个核对 manifest 与 runtime 入口。

## 文档同步依据（本次核对）

- 同步时间：2026-05-28 18:02:58 +08:00  
- 依据文件：`plugins/fcplug-deepseek-llm`、`plugins/fcplug-qwen_llm`、`plugins/fcplug-qwen-image`、`plugins/fcplug-qwen-tts`
