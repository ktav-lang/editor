>>>>> lang=en
Attempt 1: JSON as a simple key-value map
```json
{
  "ktav": "C:\\path\\to\\Ktav.tmbundle"
}
```
**Result**: `XmlSerializationException: Cannot deserialize TextMateUserBundleServiceState`

Attempt 2: JSON as an array of objects
```json
[{
  "name": "ktav",
  "enabled": true,
  "path": "C:\\path\\to\\Ktav.tmbundle"
}]
```
**Result**: Deserialization failed, with no errors logged

### 3. textmate.xml Is Unavailable During Registration

- The plugin initializes at **appFrameCreated** (very early)
- The IDE creates `textmate.xml` later
- The attempt to locate the file fails
- Even if the file is rewritten, the IDE might not reread it

### 4. The TextMate API Is Very Limited

>>>>> lang=ru
Попытка 1: JSON как простая карта ключей и значений:
```json
{
  "ktav": "C:\\path\\to\\Ktav.tmbundle"
}
```
**Результат:** `XmlSerializationException: Cannot deserialize TextMateUserBundleServiceState`.

Попытка 2: JSON как массив объектов:
```json
[{
  "name": "ktav",
  "enabled": true,
  "path": "C:\\path\\to\\Ktav.tmbundle"
}]
```
**Результат:** десериализация не сработала, ошибки не записаны в журнал.

### 3. textmate.xml недоступен во время регистрации

- Плагин инициализируется при **appFrameCreated**, слишком рано.
- IDE создаёт `textmate.xml` позже.
- Найти файл не удаётся.
- Даже переписанный файл IDE может не перечитать.

### 4. TextMate API очень ограничен

>>>>> lang=zh
尝试 1：用简单键值映射表示 JSON：
```json
{
  "ktav": "C:\\path\\to\\Ktav.tmbundle"
}
```
**结果：**`XmlSerializationException: Cannot deserialize TextMateUserBundleServiceState`。

尝试 2：用对象数组表示 JSON：
```json
[{
  "name": "ktav",
  "enabled": true,
  "path": "C:\\path\\to\\Ktav.tmbundle"
}]
```
**结果：**反序列化未成功，也没有记录错误。

### 3. 注册时 textmate.xml 尚不可用

- 插件在 **appFrameCreated** 时初始化，时间很早。
- IDE 稍后才创建 `textmate.xml`。
- 因此查找该文件失败。
- 即使重写文件，IDE 也可能不会重新读取。

### 4. TextMate API 极其有限

