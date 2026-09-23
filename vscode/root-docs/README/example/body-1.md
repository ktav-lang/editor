>>>>> lang=en
## Example

```ktav
## Ktav — no quotes, no commas, no indentation traps.
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

## Comma-free arrays and inline objects.
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
## Ktav — no quotes, no commas, no indentation traps.
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

## Comma-free arrays and inline objects.
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

>>>>> lang=zh
## 示例

```ktav
## Ktav — no quotes, no commas, no indentation traps.
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

## Comma-free arrays and inline objects.
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

