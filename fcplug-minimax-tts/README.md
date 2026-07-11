# MiniMax TTS 插件

FlowCloudAI WASM 插件，用于接入 MiniMax 同步语音合成 API。

当前 FlowCloudAI TTS 请求与响应协议以 MiniMax T2A v2 为基线，因此插件只需执行直通映射。清单内提供常用系统音色；自定义及复刻音色仍可直接填写对应的 `voice_id`。

## 支持模型

- `speech-2.8-hd` / `speech-2.8-turbo`
- `speech-2.6-hd` / `speech-2.6-turbo`
- `speech-02-hd` / `speech-02-turbo`
- `speech-01-hd` / `speech-01-turbo`

## 构建

```bash
cargo fcplug build
```

## 许可证

MIT
