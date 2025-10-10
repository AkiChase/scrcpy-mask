# 📜 脚本语法规则简介

## 🧩 变量与类型

* 动态类型系统，支持：

  * `Int`（i64）
  * `Bool`（布尔值）
  * `Str`（字符串）
* 变量需通过 `let` 声明并赋值后使用

  ```js
  let x = 100;
  ```
* 支持重新赋值（需先声明）

  ```js
  x = 200;
  ```

---

## ➕ 运算符

### 算术运算

* `+` 加法
* `-` 减法
* `*` 乘法
* `/` 除法
* `%` 取模

### 比较运算

* `<` 小于
* `<=` 小于等于
* `>` 大于
* `>=` 大于等于
* `==` 等于
* `!=` 不等于

### 逻辑运算

* `&&` 与（and）
* `||` 或（or）
* `!` 非（not）

### 字符串拼接

* `+` 可用于连接字符串

```js
"Hello" + "World" // => "HelloWorld"
```
---

## 🧠 控制结构

### 条件分支

```js
if x > 10 { ... } else { ... };
```

### 循环

```js
while x > 0 { x = x - 1; };
```

> ⚠️ 注意：代码块 `{ ... }` 末尾也需以 `;` 结尾。

---

## 🔢 内置常量

| 常量名          | 说明             |
| ------------ | -------------- |
| `ORIGINAL_W` | 配置区域的原始宽度      |
| `ORIGINAL_H` | 配置区域的原始高度      |
| `CURSOR_X`   | 鼠标指针在蒙版内的 X 坐标 |
| `CURSOR_Y`   | 鼠标指针在蒙版内的 Y 坐标 |

> 每次脚本重新执行时常量更新，执行期间为固定值。

---

## ⚙️ 内置函数

### `print(...)`

输出日志（参数自动转换为字符串）

```js
print("Value:", x); // 输出 "Value: 100"
```

### `wait(ms)`

暂停执行指定毫秒数

```js
wait(1000); // 等待 1 秒
```

### `tap(pointer_id, x, y, action?)`

模拟触摸事件

* `pointer_id`: 触控点 ID（非负整数）
* `x, y`: 相对坐标（相对于 `ORIGINAL_W` / `ORIGINAL_H`）
* `action`: `"down"`, `"up"`, `"move"`, `"default"`（默认触发 down 后 30ms up）

### `swipe(pointer_id, interval, x1, y1, x2, y2, ...)`

模拟滑动操作

* `interval`: 相邻坐标间滑动时间（毫秒）
* 至少两组坐标点 (`x1, y1, x2, y2`)

### `send_key(key_name, action?, metastate?)`

发送按键事件

* `key_name`: 按键名（如 `"KEYCODE_HOME"`）
* `action`: `"down"`, `"up"`, `"default"`（默认按下并释放）
* `metastate`: 修饰键（如 `"META_SHIFT_ON"`）

### `paste_text(text)`

粘贴指定文本到设备

```js
paste_text("Hello from script!");
```

---

## ⚠️ 错误处理

* 脚本会提示语法或运行时错误的具体位置和上下文：

  ```text
  error: Division by zero
   --> line 5, column 10 to line 5, column 15
    |
  5 |     10 / 0
    |      ^^^^^
  ```

---

## 🚫 限制与注意事项

* 不支持用户自定义函数
* 所有变量为 **全局作用域**（块内声明外部可访问）
* `send_key` 的 `key_name` 和 `metastate` 需符合
  [src/scrcpy/constant.rs](src/scrcpy/constant.rs) 中定义的枚举规范

---

## 💡 示例脚本

```rs
// 声明并初始化变量
let x = ORIGINAL_W / 2;
let y = ORIGINAL_H / 2;
let counter = 0;

// 使用内置常量进行计算
print("Original size:", ORIGINAL_W, "x", ORIGINAL_H);
print("Cursor position:", CURSOR_X, CURSOR_Y);

// 条件语句示例
if CURSOR_X > ORIGINAL_W / 2 {
    print("Cursor is on the right side");
} else {
    print("Cursor is on the left side or middle");
};

// 循环示例
while counter < 3 {
    tap(counter, x, y);     // 在当前位置点击
    x = x + 100;            // 更新变量
    counter = counter + 1;
    wait(500);              // 等待一段时间
};

// 字符串操作示例
let message = "Hello" + " " + "World";
print(message);

// 滑动示例：从中心点向右上角滑动
swipe(0, 500, ORIGINAL_W/2, ORIGINAL_H/2, ORIGINAL_W/2 + 200, ORIGINAL_H/2 - 200);

// 粘贴文本示例
paste_text("Hello from script!");

// 按键示例
send_key("VolumeUp"); // 按下并释放音量键

// 使用修饰键的示例
send_key("A", "default", "CTRL_ON");

// 控制按键时长的示例
send_key("Home", "down");
wait(100);
send_key("Home", "up");

// 使用逻辑运算符
let flag = true;
if flag && counter > 0 {
    print("Flag is true and counter is positive");
};

if !flag || counter == 3 {
    print("Either flag is false or counter equals 3");
};

// 数值比较
if x > ORIGINAL_W / 2 && y < ORIGINAL_H / 2 {
    print("Position is in the upper right quadrant");
};
```