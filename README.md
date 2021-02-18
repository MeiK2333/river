# river

## 环境要求

- linux

## TODO

- 解决偶发性的 `wait4 failure` 问题
- 解决内存占用已经超出限制时，未能结束进程，从而结果表现为 TLE 的问题
- 修复 Rust 无法编译的问题
- 更新内存测量机制
- 修复某些情况下时间占用会偏高的 bug（使用 `cin` 输入的代码会测量出偏高的时间，`scanf` 却没有这个问题）
- 添加 debug 界面以定位并解决问题
- 修改 `cgroup v1` 为 `cgroup v2`
- 研究下是否需要复用沙盒，之前因为沙盒创建与启动的速度很快（1-2ms），就没有考虑复用
