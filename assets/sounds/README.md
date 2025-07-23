# 音频资源说明

## 📁 文件夹结构
```
assets/sounds/
├── music/          # 背景音乐
│   ├── menu.ogg    # 菜单音乐
│   └── game.ogg    # 游戏音乐
├── sfx/            # 音效
│   ├── jump.wav    # 跳跃音效
│   ├── land.wav    # 着地音效
│   ├── footstep.wav # 脚步声
│   ├── collect.wav # 收集音效
│   └── button.wav  # 按钮点击音效
└── README.md       # 本文件
```

## 🎵 推荐的免费音频资源网站

### 高质量免费音效
1. **Freesound.org** (需注册)
   - 网址: https://freesound.org/
   - 许可: Creative Commons
   - 特点: 专业级音效库，质量极高
   - 推荐搜索: "jump", "footstep", "coin collect", "button click"

2. **Zapsplat** (需免费注册)
   - 网址: https://www.zapsplat.com/
   - 许可: 免费商用
   - 特点: 广播级音效，分类详细
   - 推荐: 游戏音效分类

3. **Myinstants**
   - 网址: https://www.myinstants.com/en/instant/heavy-footsteps-7421/
   - 许可: 免费使用
   - 特点: 游戏音效全面，方便简单
   - 推荐: 按钮提示音效

### 免费背景音乐
1. **Kevin MacLeod (incompetech.com)**
   - 网址: https://incompetech.com/music/royalty-free/
   - 许可: Creative Commons
   - 特点: 高质量配乐，风格多样
   - 推荐: "Sneaky Snitch", "Cipher", "Carefree"

2. **DOVA-SYNDROME** (日本)
   - 网址: https://dova-s.jp/
   - 许可: 免费商用
   - 特点: 日式风格音乐，适合动漫游戏
   - 推荐: 搜索 "ゲーム" (游戏)

3. **Purple Planet Music**
   - 网址: https://www.purple-planet.com/
   - 许可: 免费商用
   - 特点: 电子音乐，适合科幻游戏

## 🎯 针对 Shirou Runner 的音频建议

### 背景音乐风格
- **菜单音乐**: 神秘、史诗感，符合 Fate 世界观
- **游戏音乐**: 节奏感强，适合跑酷动作
- **格式**: OGG Vorbis (Bevy 推荐格式)
- **时长**: 2-3分钟循环

### 音效设计
- **跳跃音效**: 清脆的"whoosh"声
- **着地音效**: 沉闷的撞击声
- **脚步声**: 轻快的跑步声
- **收集音效**: 明亮的"ding"声
- **按钮音效**: 简洁的点击声

## 🔧 技术规格

### 音频格式支持 (Bevy 0.16)
- **推荐格式**: OGG Vorbis (.ogg)
- **备选格式**: WAV (.wav), MP3 (.mp3)
- **采样率**: 44.1kHz 或 48kHz
- **位深度**: 16-bit 或 24-bit
- **声道**: 单声道或立体声

### 文件大小建议
- **音效**: < 100KB 每个
- **背景音乐**: < 5MB 每个
- **总音频资源**: < 20MB

## 📥 快速获取资源的步骤

### 1. 获取跳跃音效
```bash
# 搜索关键词: "jump sound effect game"
# 推荐网站: freesound.org
# 搜索: "jump" + "game" + "8bit" 或 "retro"
```

### 2. 获取背景音乐
```bash
# 搜索关键词: "epic fantasy music loop"
# 推荐网站: incompetech.com
# 分类: "Cinematic" 或 "Electronic"
```

### 3. 批量下载工具
- **youtube-dl**: 从 YouTube 下载 CC 音乐
- **wget**: 批量下载 freesound 资源
- **Audacity**: 音频编辑和格式转换

## 🎨 自制音效 (如果需要)

### 使用 Audacity (免费)
1. **跳跃音效**: 录制"呼"声 + 高通滤波
2. **着地音效**: 录制拍手声 + 低通滤波
3. **脚步声**: 录制真实脚步 + 节奏调整
4. **按钮音效**: 合成简单的正弦波

### 在线音效生成器
- **sfxr**: 8-bit 风格音效生成器
- **Bfxr**: sfxr 的增强版本
- **ChipTone**: 复古游戏音效生成

## 🚀 快速开始

如果你想立即测试音频功能，可以：

1. **下载测试音效**:
   - 从 freesound.org 下载任意跳跃音效
   - 重命名为 `jump.wav` 放入 `assets/sounds/sfx/`

2. **下载测试音乐**:
   - 从 incompetech.com 下载任意循环音乐
   - 重命名为 `game.ogg` 放入 `assets/sounds/music/`

3. **测试代码**:
   ```rust
   // 在游戏中播放音效
   commands.spawn((
       AudioPlayer::new(asset_server.load("sounds/sfx/jump.wav")),
       PlaybackSettings::DESPAWN,
   ));
   ```

这样你就可以快速为游戏添加音频效果了！