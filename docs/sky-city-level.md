# Windheart Sky City — LDtk Level

> 制作基线：2026-07，LDtk 1.5.3 + Bevy 0.19 + `bevy_ecs_ldtk` 0.15。

## 交付内容

- `assets/levels/sky_city_of_winds.ldtk`：可直接由 LDtk 1.5.3 打开的关卡工程。
- `assets/images/levels/sky_city_panorama.png`：原创可循环天空都市远景。
- `assets/images/levels/sky_city_tiles.png`：LDtk TerrainTiles 图层使用的原创 8 格图集。
- `src/plugins/sky_level.rs`：LDtk 注册和关卡生命周期。
- `src/components/level.rs`：编辑器实体、IntGrid 和运行时状态。
- `src/systems/sky_level.rs`：碰撞合并、地图敌人、检查点、战斗门、风场、地标和终点。

## 地图规格

- 尺寸：`12288 × 1536 px`。
- 网格：`32 × 32 px`，即 `384 × 48` 格。
- 结构：12 段连续浮岛主线，31 组高空平台与一条风轮机井纵向支路。
- 战斗：32 个手工敌人点位，3 个封闭战斗场，最终强化英灵。
- 流程：起点、5 个检查点（含起点）、10 组电弧危险区、6 组风升流、终点核心。
- 地标：28 个 LDtk 背景实体，包括浮岛、塔楼、风车、水渠、瀑布、巨树、神殿和水晶。

## LDtk 图层

### Gameplay

- `PlayerStart`
- `Checkpoint(id)`
- `EnemySpawn(kind, arena, healthMultiplier, patrolRange)`
- `CombatGate(arena)`
- `Goal`

### Decor

- `Backdrop(kind, scale, depth)`

### TerrainTiles

- `SkyCityTiles`（8 × 1，32 px 网格）
- 与非零 IntGrid 单元一一对应，由 `bevy_ecs_ldtk`/`bevy_ecs_tilemap` 批量渲染

### Collision

- `1 StoneSolid`
- `2 CloudSolid`
- `3 ArcHazard`
- `4 WindLift`

Stone/Cloud IntGrid 单元在关卡加载后按连续行列合并成矩形碰撞体；致密地形由 TerrainTiles 批量渲染，运行时只为表面草边和稀疏植被增加少量实体。

## 再生成

关卡和远景均由确定性脚本生成。调整脚本中的布局数据后运行：

```bash
python3 scripts/generate_sky_city_ldtk.py
python3 scripts/generate_sky_city_panorama.py
python3 scripts/generate_sky_city_tileset.py
```

纯文本关卡生成物可在 CI/审查中检查是否过期：

```bash
python3 scripts/generate_sky_city_ldtk.py --check
```

生成完成后可用本机 LDtk 打开：

```bash
'/home/zyg/下载/ubuntu-distribution/LDtk 1.5.3 installer.AppImage' \
  --no-sandbox assets/levels/sky_city_of_winds.ldtk
```

运行时不读取生成脚本，只读取已提交的 `.ldtk` 与 PNG 成品。

## 设计约束

- 主线无需二段跳、墙跳或攻击位移。
- 必经平台的典型水平缺口不超过 96 px，高差不超过 96 px。
- 高风险跳跃、风升流和高空平台均属于奖励支路，不阻断主线。
- LDtk 模式停用旧的随机刷怪、程序化地面装饰和全局灰色地面。
- 坠落和死亡从最近已激活检查点恢复。
- 第三个战斗场完成后终点核心才可触发胜利状态。
