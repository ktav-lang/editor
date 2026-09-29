>>>>> lang=en
## Possible Solutions

### 1. Manual Registration with Good UX ✓ (Recommended at the Time)

**Advantages**:
- Works reliably
- Does not depend on internal APIs
- Is compatible with all IDE versions

**Implementation**:
- Extract the bundle automatically into a temporary directory
- Have the plugin offer the user a way to copy its path
- Alternatively, provide a "Register Ktav TextMate Bundle" menu action that opens Settings → TextMate Bundles
- Or provide detailed, step-by-step documentation

### 2. Write to textmate.xml Before the IDE Loads

**Difficulty**: The IDE initializes this setting very early, before the plugin loads.

>>>>> lang=ru
## Возможные решения

### 1. Ручная регистрация и понятный интерфейс ✓ (РЕКОМЕНДОВАЛОСЬ)

**Преимущества**:
- Работает надёжно
- Не зависит от внутреннего API
- Совместимо со всеми версиями IDE

**Реализация**:
- Бандл автоматически извлекается во временную директорию
- Плагин предлагает пользователю скопировать путь
- Или: действие меню «Register Ktav TextMate Bundle» открывает Settings → TextMate Bundles
- Либо: подробная пошаговая документация

### 2. Записывать textmate.xml до загрузки IDE

**Сложность**: IDE инициализирует эту настройку очень рано, до загрузки плагина.

>>>>> lang=zh
## 可选解决方案

### 1. 手动注册并提供良好体验 ✓ (当时推荐)

**优点**：
- 可靠工作
- 不依赖内部 API
- 兼容所有 IDE 版本

**实现**：
- 自动把包解压到临时目录
- 插件提示用户复制路径
- 或者：菜单操作“Register Ktav TextMate Bundle”打开 Settings → TextMate Bundles
- 也可以提供详细的分步说明

### 2. 在 IDE 加载前写入 textmate.xml

**难点**：IDE 在插件加载前很早就初始化该设置。

