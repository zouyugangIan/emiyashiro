# 2D 角色动画导航

这份文档是精灵动画的新手入口。当前有两条正式渲染链：

- HF 卫宫士郎：`TextureAtlas` 主链，由 RON 决定帧序列和播放速度。
- 间桐樱：原画不是规则网格，使用逐张图片链。

## HF 士郎一帧是怎样画出来的

1. `plugins/core.rs` 加载纹理，创建图集布局，组成一个 `SpriteAnimationSheets`。
2. `systems/game.rs` 生成玩家，附加 `SpriteAnimation`、`SpriteAnimationSheets` 和 `AttackAnimationState`。
3. `systems/sprite_animation.rs` 每帧依次执行：
   - 推进攻击剩余时间；
   - 根据落地、蹲下、移动和攻击状态选择动画；
   - 选择对应图集并推进帧索引；
   - 按垂直速度锁定起跳、滞空、下落姿势，并在抓边/上翻时复用专用姿势；
   - 将攻击图集完整帧数压入实际动作锁定时长，避免长图集被提前截断；
   - 对起跑、落地和空中姿势施加轻量平滑伸缩，锚点会反向补偿以保持脚底稳定。
4. `assets/animations/hf_shirou.ron` 是基础动画帧序列的唯一配置源。

`PresentationPlugin` 对上述三个运行系统使用 `.chain()`，因此它们的先后顺序是确定的。

基础移动强调快速取消：落地收势可立即被跑动打断；待机进入跑步会从收紧的过渡帧开始，
避免高挑待机姿势直接跳到最大跨步。跳跃帧不再按固定计时盲播，而是由真实 `Velocity.y`
驱动，因此短跳、长跳和风场抬升都能保持姿势与轨迹一致。

## 重要数据结构

- `AnimationType`：待机、跑步、攻击、跳跃、蹲下和落地。
- `AnimationClipData`：一个动画的帧列表、帧时长、播放模式和速度联动参数。
- `SpriteAnimation`：单个角色当前的动画、逻辑帧位置和计时器。
- `SpriteAnimationSheets`：动画所需的纹理、图集布局和可用帧数。
- `AttackAnimationState`：攻击动画的总时长、剩余时间、重触发序号和攻击样式。

RON 中的 `current_frame` 不是图片序号。运行时的 `current_frame` 先指向帧列表中的位置，再由
`frames[current_frame]` 得到真正的图集索引。

## 修改基础动画

修改 `assets/animations/hf_shirou.ron` 后重新编译。配置在编译时内嵌，因此可以从任意工作目录启动游戏，
不会再因相对路径找不到 RON。启动时会检查：

- 六个必需动画都存在；
- 帧列表不为空；
- 帧时长、最小帧时长和速度参考值是正的有限数。

加入新精灵图集时，还要在 `asset_paths.rs` 声明列数/行数，并在 `plugins/core.rs` 创建对应
`TextureAtlasLayout`。帧索引必须小于该图集的总帧数。

## 樱的逐帧图片链

`systems/image_sequence_animation.rs` 只处理樱，不会接管 HF 士郎。基础动作和 7 组攻击
都只切换独立图片；攻击源图会预切为 224 张 `256x256` PNG，运行时不会给樱附加
`TextureAtlas`。该链使用确定的状态切换、首帧立即应用和卡顿补帧语义。
