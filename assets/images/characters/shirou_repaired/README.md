# 士郎动作修复素材

本目录是 1P 的替换素材，由内置 `image_gen` 根据仓库现有士郎素材生成。
完整提示词保存在 `generation-prompts.md`。本次未取得另一段 ChatGPT 对话的原图或下载链接。

- `locomotion.png`：4 个待机姿势、8 个跑步姿势、起跳/滞空/下落、蹲下，共 16 帧。
- `ground_attacks.png`：8 帧完整突刺、8 帧低扫，替换地面轻攻击第 3、4 行。
- 同名 JSON：每帧实际像素边界、身体横向原点与脚底基线。生成图不是精确等分网格，运行时必须使用这些边界。

原图保持生成时的 RGBA 数据。运行时按 JSON 创建 `TextureAtlasLayout`，以统一比例显示完整帧，
并把脚底对齐碰撞体底部；面朝左时同步镜像身体原点。旧图留在原位置供追溯。

冲刺第 5 帧和滑步第 3 帧的断肢画面通过播放相邻完整帧替换，动作时长保持一致。
其他攻击模块继续使用原有图集；透明孔洞自动检查只是提示，不能替代逐帧目视检查。

验证命令：

```bash
python3 scripts/audit_shirou_repaired.py
cargo test --lib --all-features
```
