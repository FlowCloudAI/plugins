# 智谱 TTS 插件

FlowCloudAI WASM 插件，用于适配智谱 `glm-tts` 语音合成 API。

插件固定使用官方流式 PCM + hex 输出，再合并成 FlowCloudAI 的统一 TTS 响应，因此无需修改核心的二进制响应处理。

## 构建

```bash
cargo fcplug build
```

## 许可证

MIT
