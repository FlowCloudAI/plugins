# FlowCloudAI 插件源码

本仓库保存 FlowCloudAI 官方示例与内置插件源码。插件通过 `cargo-fcplug` 构建为 `.fcplug` 包，供 FlowCloudAI 桌面端加载。

## 插件列表

| 目录 | 说明 |
|------|------|
| `fcplug-deepseek-llm/` | DeepSeek LLM 插件 |
| `fcplug-qwen_llm/` | 通义千问 LLM 插件 |
| `fcplug-qwen-image/` | 通义千问图像生成插件 |
| `fcplug-qwen-tts/` | 通义千问 TTS 插件 |

## 构建

进入任一插件目录后执行：

```bash
cargo fcplug build
```

构建产物位于各插件的 `dist/` 目录，该目录不会提交到仓库。

## 注意事项

- 插件 API Key 由宿主应用通过安全存储管理，源码和 manifest 中不得写入真实密钥。
- 插件目标平台为 `wasm32-wasip2`。
- 修改 WIT 或 manifest 协议时，需要同步检查宿主侧 `core_ai_client` 的协议常量与加载逻辑。
