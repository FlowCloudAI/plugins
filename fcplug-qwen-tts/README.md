# Qwen TTS Plugin

FlowCloudAI WASM 插件，用于适配阿里云百炼（DashScope）Qwen 语音合成（TTS）API。

---

## 支持模型

- `qwen3-tts-flash`
- `qwen3-tts-instruct-flash`
- `qwen3-tts-instruct-flash-2026-01-26`
- `qwen3-tts-flash-2025-11-27`

---

## 构建

```bash
cargo build --target wasm32-wasip2 --release
cargo fcplug build
```

---

## 接口

实现 WIT `mapper` 接口：

- `map-request`：将统一 TTS 请求映射为 DashScope 多模态生成格式
- `map-response`：将 DashScope 响应映射回统一格式（提取音频 URL / Base64）
- `map-stream-line`：处理流式响应行（如适用）

---

## 许可证

MIT
