# MiniMax LLM 插件

FlowCloudAI WASM 插件，用于适配 MiniMax OpenAI Chat Completions API。

## 支持模型

- `MiniMax-M3`
- `MiniMax-M2.7` / `MiniMax-M2.7-highspeed`
- `MiniMax-M2.5` / `MiniMax-M2.5-highspeed`
- `MiniMax-M2.1` / `MiniMax-M2.1-highspeed`
- `MiniMax-M2`

`MiniMax-M3` 的图片和视频输入暂不支持，因为当前 FlowCloudAI LLM 消息协议只接受文本内容。

## 构建

```bash
cargo fcplug build
```

## 许可证

MIT
