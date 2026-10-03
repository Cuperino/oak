# oak-node C ABI 导出计划（app 迁移 + 扩展插件）

## 目标

两条消费线共用一套导出：

1. **oak-app 迁移**：oak-app 目前用到的 oak-node API 全部经 `liboak_engine` C ABI 可达，app 从"直接链接 rlib"改为"链接引擎动态库"
2. **扩展插件**：能扩展 Oak 功能的插件（导入导出工具、自动化、批处理）获得完整的文档模型操作面

## 架构前提（已就绪）

- 句柄体系：`export_handle!`（Arc 背书的不透明句柄）、`NativeMirror`（repr(C) 镜像双向转换）、vec ABI（`OakVecI32`/`OakVecString`）、snprintf 式字符串约定、`-1`/false/null 错误约定
- **NodeId 即句柄**：`NodeId { index: u32, generation: u32 }` 有现成的 `identity()`/`from_identity()` u64 打包——文档 API 的全部入口都是 `oak_node_xxx(u64 id, ...)` 形状，节点生命周期由项目本身管理，不需要 Arc 句柄
- **进程内共享**：插件 dlopen 进 app 后与 app 解析到同一份引擎 dylib，全局状态（活动项目、配置、色彩管线）天然共享，无需 IPC

## 前置设计决策（开工前定）

1. **活动项目槽**：引擎加全局"活动项目"槽（`oak_project_set_active`/`oak_project_active`），app 打开项目时设置，插件查询。与音频 manager 单例模式一致。备选（注册时宿主传句柄）更显式但破坏"插件只依赖 C ABI"的形状。推荐前者
2. **undo 语义**：所有文档变更必须可撤销。约定：引擎的每个变更函数内部自动包一个 undo 命令；插件可用 `oak_undo_group_begin/end` 把一批调用并成一个 undo 步。oak-undo 的 global API（group/push/undo/redo/can_undo/can_redo）面小，直接导出
3. **NodeCore 参数访问**：读写统一走 `standard_value`/`set_standard_value`/`value_at_time`（id + element + OakNodeValue），不为每个输入写专门函数。`OakNodeValue` 镜像从 oak-node 挪到 oak-engine（镜像归属原则）

## 导出清单与批次

### 批次 1：文档只读面 + 基础设施（最先，风险最低）

引擎 `oak-engine/src/node.rs`（新模块）+ SDK `node.rs`：

- `OakNodeValue` 镜像迁入引擎 + `oak_node_value_to_*`/`from_*` 转换（INT/FLOAT/BOOL/RATIONAL/COLOR/VEC2/3/4；STRING 类型走单独的字符串通道）
- `oak_node_id_identity`/`oak_node_id_from_identity`（u64 打包，纯函数）
- 活动项目槽：`oak_project_active()` → 项目句柄（export_handle!），`oak_project_set_active(...)`
- 项目只读：`oak_project_name`/`filename`/`pretty_filename`（snprintf）、色彩设置 getter（复用 corestate 的字符串约定）
- 结构遍历（全部 NodeId + 计数/按索引取）：
  - `oak_project_root_folder()` → NodeId
  - `oak_folder_child_count(id)` / `oak_folder_child_at(id, i)` → NodeId
  - `oak_node_kind(id)` → 类型判别（folder/footage/sequence/track/block…）
  - `oak_footage_*`：`filename`、流计数、`video_params(index, out)`/`audio_params(index, out)`（复用 `OakVideoParams`/`OakAudioStreamParams` 镜像）、`duration`/`video_length`（OakRational）、proxy 状态查询（接现有 `oak_codec_proxy_*`）
  - `oak_sequence_*`：视频/音频轨道数、`track_at(i)` → NodeId、时长（`verify_length`）
  - `oak_track_*`：`block_count`/`block_at(i)` → NodeId、`block_containing_time(id, OakRational)`、`length`
  - `oak_block_*`：`in_`/`out`/`length`/`media_out`（OakRational）、类型判别（clip/gap/transition/adjustment）
  - 参数只读：`oak_node_standard_value(id, input_id, element, out: *mut OakNodeValue)`、`value_at_time(...)`（带 OakRational 时间）
- undo 只读：`oak_undo_can_undo/can_redo/count/index`
- 测试：只读面遍历一个手工构造的项目（引擎测试用 oak-node 直接建图），SDK 同步覆盖

### 批次 2：文档编辑面（undo 集成是核心）

- undo 命令面：`oak_undo_group_begin(name)`/`group_end`/`group_abort`（栈式，错误即 false）——插件成批操作一个 undo 步
- 素材导入：`oak_footage_import(filename)` → NodeId（内部 probe + 入媒体库 + undo 命令）
- 结构编辑（每个内部自动包 undo 命令）：
  - `oak_folder_create(name, parent)` → NodeId、`oak_folder_remove_child`
  - `oak_sequence_create(name)` → NodeId、`oak_sequence_set_default_parameters`
  - `oak_track_create(sequence, kind)` → NodeId（kind 0=video/1=audio，`TrackType`）
  - `oak_block_create_clip/gap/transition/adjustment(...)` → NodeId（clip 带 footage NodeId + 流索引 + in/out）
  - `oak_track_append_block`/`insert_block_at_index`/`remove_block`/`ripple_remove_block`/`replace_block`
  - 块修剪：`oak_block_set_in`/`set_out`/`set_length_and_media_out`/`set_length_and_media_in`/`set_length_keeping_out`
  - 参数写：`oak_node_set_standard_value(id, input_id, element, OakNodeValue)`
- 测试：引擎测试建序列 → 导入素材 → 建 clip → 修剪 → undo 回滚全链验证；SDK 端到端同一流程

### 批次 3：项目存取与设置

- `oak_project_save(filename)`（serializer::save）、`oak_project_set_filename`
- 项目设置写：工作色彩空间/输出 spec（写设置 + 同步 corestate 管线）
- 深度复制：`oak_project_deep_copy`（脚本沙箱场景）
- 权限语义：插件可写项目文件吗？（保存路径白名单/确认回调）——本批次开工前定

### 批次 4（插件协议本体，独立设计轮）

`oak_plugin_register(descriptor, callbacks)`：描述符（id/名称/版本/菜单路径）+ 回调表。这是 OFX `mainEntry` 的对应物，涉及 UI 集成点（菜单/面板/快捷键宿主），单独设计。

## 明确不做

- 渲染图内部件（connect/disconnect、traverser、NodeCaches、graphendpoints）：宿主渲染管线内部，不进 ABI
- `Factory` 元注册、`multicamnode` 内部：app 专属
- gizmo 编辑器内部：查看器交互是 UI 层的事

## 验收标准

- 每批次：引擎测试全绿（含真实媒体端到端）+ SDK 镜像全绿 + oak-app/oak-codec 无回归
- 批次 2 完成后：用一个纯 C ABI 的冒烟插件（或 SDK 小程序）完成"打开项目 → 导入素材 → 建序列 → 摆 clip → 撤销"全流程
- oak-app 迁移验收（跨批次持续）：app 侧 oak_node 直接调用点逐批替换成引擎 C ABI 调用，直到 `oak-node` 可从 oak-app 的 Cargo.toml 依赖里移除
