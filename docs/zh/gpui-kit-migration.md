# oak-app 的 gpui-kit 迁移

oak-app 从 oak-gpui fork（`gpui/` 子仓库的 `gpui` / `gpui_widgets` /
`gpui_elements` crate）迁移到 gpui-kit 0.7.3（registry 的
`gpui-pre` + `gpui-base` + `gpui-component`）。引擎侧保持直链进程内
oak-\* 模块 crate 不变。

## 结构决策

- **fork 的自定义 UI 模块提升为 app 侧模块**，构建在 gpui-kit 公共 API 上：
  - `gpui` crate 的 `effect_stack` / `node_graph` / `timeline` /
    `i18n`（字符串表挂钩）
  - `gpui_widgets` 的 `theme`（Olive 主题 → `GlobalColors`，
    `cx.default_colors()` 语义不变）、`menu`、`dialog`、`slider`（+`value`、
    `keyable`）、`tooltip`、`audio_meter`、`curve_editor`、`scopes`、
    `project_explorer`、`viewer`、`icons`
  - 迁移期间暂存于 `crates/oak-widgets`（独立可测），完成后折回
    `oak-app/src/widgets/`。
- **dock 不移植**：换成 gpui-kit 内置的 `component::dock`
  （`DockSkin` + `DockArea` + `BasePanel`/`Panel`）。
- **`gpui_elements::editable_text` 不移植**（4.9k 行编辑引擎）：换成
  gpui-kit 内置 `component::input::Input` / `InputState`（slider 直输编辑、
  文件对话框路径字段已切换）。

## 迁移中的 API 漂移（fork → gpui-pre 0.3.9）

- `Pixels` 的 `.0` 字段私有化 → `.as_f32()`（113 处，编译器诊断驱动批量修复）。
- `App::active_drag` 私有化 → 各拖拽 payload 自己记录抓点偏移
  （`on_drag` 构造器的第二参数正是该偏移）；timeline 的 clip 移动 ghost
  改由视图自身字段跟踪。
- `TextRun.letter_spacing`（fork 私增字段）→ 删除。
- `Window::draw(cx).clear()` → `.clear(cx)`；`cx.new(f)` 必须传闭包。
- `path_builder::PathBuilder` → 根导出 `PathBuilder`；const 上下文
  `Pixels(x)` → `px(x)`。
- gpui-kit 组件（如 `Input`）需要 `gpui_kit::init(cx)` 安装组件主题
  （`cx.theme()`），与 `init_colors`（`cx.default_colors()`）并存——
  两个全局都要初始化。

## 已知功能回退

### viewer 的 10-bit GPU 纹理路径（已移除）

fork 的 `SurfaceSource::Texture` 变体让 viewer 把引擎上传的 RGBA16F
wgpu 纹理直接采样进 10-bit swapchain（`oak-app/src/oakui/gpu.rs` 生产，
`gpui_widgets::viewer` 消费，`register_gpu_frame` 注册表衔接）。
gpui-pre 的 `SurfaceSource` 只有 macOS CoreVideo 变体，没有外部纹理入口。

**现状**：viewer 全平台走 CPU `img()`（BGRA8 → sprite atlas），画面正确
但只有 8-bit。GPU 注册表与对应测试已移除。

**恢复条件**：需要渲染器层支持 —— gpui-kit 平台层（gpui-pre-platform /
gpui-fast）提供外部 wgpu 纹理绘制入口，或 Oak 自建渲染叠加层。届时
`oakui/gpu.rs` 整条链路一并重做。

## 进度

- [x] oak-widgets：fork 模块提升（dock 除外），155 单测 + 5 doctest 全过
- [ ] oak-app 本体：依赖切换、oakui/ 引擎缝（frames/gpu/real 耦合 fork
      渲染器的部分重写）、panels/、app.rs 外壳（dock 换内置）
- [ ] oak-widgets 折回 `oak-app/src/widgets/`
- [ ] 删除 gpui/ 子仓库引用
