# Moonshot LLM 插件

FlowCloudAI WASM 插件，用于适配 Moonshot Kimi Chat Completions API。

## 支持模型

- `kimi-k2.7-code` / `kimi-k2.7-code-highspeed`
- `kimi-k2.6`
- `kimi-k2.5`
- `moonshot-v1-8k` / `moonshot-v1-32k` / `moonshot-v1-128k`

K2.7 Code、K2.6 和 K2.5 的图片、视频输入暂不支持，因为当前 FlowCloudAI LLM 消息协议只接受文本内容。K2.7 Code 始终开启思考；K2.6 与 K2.5 可切换思考模式。

## 构建

```bash
cargo fcplug build
```

## 许可证

MIT
