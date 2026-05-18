# 退回 "stake 365 day" 更新指南

## 概述

提交 `327257b` 新增了 365 天质押档位（period_type = 4，日利率 1.2%）。本文档说明如何将代码退回到该提交之前的状态。

## 涉及文件

| 文件 | 改动内容 |
|------|----------|
| `programs/vela-anchor/src/constants.rs` | 新增 `STAKE_PERIOD_365_DAYS`、`PERIOD_365_DAYS`、`DAILY_RATE_365_DAYS` 三个常量 |
| `programs/vela-anchor/src/stake_token.rs` | `get_current_rates` 返回值从 3 元组改为 4 元组；`handler_create_stake` match 分支新增 365 天；`handler_query_current_rates` 输出新增 rate_365d |
| `programs/vela-anchor/src/structs.rs` | `CurrentRatesResult` 新增 `rate_365d` 字段 |
| `programs/vela-anchor/src/lib.rs` | 注释从 "three tiers" 改为 "four tiers" |

## 方法一：Git Revert（推荐）

直接生成一个反向提交，保留完整历史：

```bash
git revert 327257b
```

如果有冲突需手动解决后：

```bash
git add .
git revert --continue
```

## 方法二：手动退回

如果 revert 冲突较多或只想部分退回，按以下步骤手动操作。

### 1. constants.rs

删除以下三行：

```rust
pub const STAKE_PERIOD_365_DAYS: u8 = 4;
pub const PERIOD_365_DAYS: u64 = 365 * SECONDS_PER_DAY; // 365 days in seconds
pub const DAILY_RATE_365_DAYS: u64 = 12_000; // 1.2%
```

### 2. stake_token.rs

**a) `get_current_rates` 函数**

将返回类型从 `(u64, u64, u64, u64)` 改回 `(u64, u64, u64)`，删除 `rate_365d` 的计算和返回：

```rust
pub fn get_current_rates(reduction_count: u16) -> (u64, u64, u64) {
    let rate_7d  = calc_current_daily_rate(DAILY_RATE_7_DAYS, reduction_count);
    let rate_30d = calc_current_daily_rate(DAILY_RATE_30_DAYS, reduction_count);
    let rate_90d = calc_current_daily_rate(DAILY_RATE_90_DAYS, reduction_count);
    (rate_7d, rate_30d, rate_90d)
}
```

**b) `handler_create_stake` 中的 match 分支**

删除 365 天的分支：

```rust
// 删除这一行：
STAKE_PERIOD_365_DAYS => (PERIOD_365_DAYS, calc_current_daily_rate(DAILY_RATE_365_DAYS, reduction_count)),
```

**c) `handler_query_current_rates` 函数**

将解构从 4 元组改回 3 元组，msg! 和返回值中移除 `rate_365d`：

```rust
let (rate_7d, rate_30d, rate_90d) = get_current_rates(reduction_count);

msg!("QueryCurrentRates: total_output={}, reduction_count={}, rate_7d={}, rate_30d={}, rate_90d={}",
    global_state.total_output, reduction_count, rate_7d, rate_30d, rate_90d);

Ok(CurrentRatesResult {
    total_output: global_state.total_output,
    reduction_count,
    rate_7d,
    rate_30d,
    rate_90d,
})
```

### 3. structs.rs

从 `CurrentRatesResult` 中删除：

```rust
/// 365-day lock daily interest rate (RATE_BASIS_POINTS precision)
pub rate_365d: u64,
```

### 4. lib.rs

将注释改回：

```rust
/// Query current lock-up interest rates for all three tiers (read-only, affected by halving mechanism)
```

## 退回后验证

```bash
cargo build-sbf
anchor test
```

确认编译通过且测试全部通过即可。

## 注意事项

- 如果链上已有用户创建了 365 天质押（period_type = 4），退回代码后这些质押将无法正常 unstake/claim，需要额外处理迁移逻辑。
- 建议退回前先检查链上是否存在 period_type = 4 的 StakeEntry 账户。
