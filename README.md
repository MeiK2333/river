# river

## 环境要求

- linux

## Example

```json
{
  "language": 0,
  "judge_type": 0,
  "compile_data": {
    "code": "I2luY2x1ZGUgPHN0ZGlvLmg+CmludCBtYWluKCkgewogIHByaW50ZigiSGVsbG8gV29ybGQhXG4iKTsKICByZXR1cm4gMDsKfQ=="
  },
  "judge_data": {
    "in_data": "SGVsbG8gV29ybGQh",
    "out_data": "SGVsbG8gV29ybGQh",
    "time_limit": 1000,
    "memory_limit": 65535
  }
}
```

## TODOs

已经完成基本功能，后续需要优化

- 基于 ptrace 的精准内存测量
- 基于 cgroups 的资源控制
- 用户、组限制
- 示例代码
- 安全测试
- 优化 args 生成代码，减少测量出的用户代码执行时间
- special judge
