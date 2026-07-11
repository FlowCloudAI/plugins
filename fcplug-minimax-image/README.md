# MiniMax Image 插件

FlowCloudAI WASM 插件，用于适配 MiniMax 图片生成 API。

## 支持模型

- `image-01`：文生图、人物主体参考图生图
- `image-01-live`：文生图、人物主体参考图生图，并由厂商提供画风增强

当前 FlowCloudAI 图片协议没有画风字段，因此 `image-01-live` 的 `style` 参数暂不开放。MiniMax 仅接受一张人物主体参考图，传入多张图片时插件使用第一张。

## 构建

```bash
cargo fcplug build
```

## 许可证

MIT
