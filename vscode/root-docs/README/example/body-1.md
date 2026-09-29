>>>>> lang=en
## Example

```ktav
## Ktav — optional quotes and no commas between block array items.
service: socks5-rotator
## bare scalars are auto-typed: int / float / bool / null
port: 20082
debug: true

## Dotted keys are a flat alternative to nesting.
node.host: a.example
node.port: 1080

## '::' forces a literal string — keeps "8080" a string, not a number.
node.token:: 8080

## Need a literal '.' or ':' inside a key? Escape it (new in 0.6.0).
metric.http\.requests: 42

## Block array items need no commas; inline object fields do.
upstreams: [
    { host: a.example, port: 1080, weight: 0.7 }
    { host: b.example, port: 1080, weight: 0.3 }
]

## Multi-line strings.
motd: (
    Welcome to the node.
    Please behave.
)
```

>>>>> lang=ru
## Пример

```ktav
## Ktav — кавычки необязательны, а элементы блочного массива не требуют запятых.
service: socks5-rotator
## Значения без кавычек распознаются автоматически: int / float / bool / null
port: 20082
debug: true

## Ключи с точками — альтернатива вложенным объектам.
node.host: a.example
node.port: 1080

## '::' задаёт строку явно — 8080 остаётся строкой, а не числом.
node.token:: 8080

## Нужна буквальная '.' или ':' в ключе? Экранируйте её (с версии 0.6.0).
metric.http\.requests: 42

## В блочном массиве запятые не нужны; поля объектов в строке разделяются запятыми.
upstreams: [
    { host: a.example, port: 1080, weight: 0.7 }
    { host: b.example, port: 1080, weight: 0.3 }
]

## Многострочные строки.
motd: (
    Welcome to the node.
    Please behave.
)
```

>>>>> lang=zh
## 示例

```ktav
## Ktav：引号可选，块数组的元素之间不需要逗号。
service: socks5-rotator
## 无引号标量会自动识别类型：int / float / bool / null
port: 20082
debug: true

## 点分隔键可替代嵌套对象。
node.host: a.example
node.port: 1080

## '::' 强制保留字符串：8080 不会被解析为数字。
node.token:: 8080

## 键中需要字面量 '.' 或 ':'？用反斜杠转义（0.6.0 起支持）。
metric.http\.requests: 42

## 块数组的元素无需逗号；行内对象的字段需要逗号分隔。
upstreams: [
    { host: a.example, port: 1080, weight: 0.7 }
    { host: b.example, port: 1080, weight: 0.3 }
]

## 多行字符串。
motd: (
    Welcome to the node.
    Please behave.
)
```

