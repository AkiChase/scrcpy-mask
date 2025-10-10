# 📜 Script Syntax Overview

## 🧩 Variables and Types

* Dynamic typing system supporting:

  * `Int` (i64)
  * `Bool` (boolean)
  * `Str` (string)
* Variables must be declared with `let` before use:

  ```js
  let x = 100;
  ```
* Reassignment is supported (variable must be declared first):

  ```js
  x = 200;
  ```

---

## ➕ Operators

### Arithmetic Operators

* `+` Addition
* `-` Subtraction
* `*` Multiplication
* `/` Division
* `%` Modulo

### Comparison Operators

* `<` Less than
* `<=` Less than or equal to
* `>` Greater than
* `>=` Greater than or equal to
* `==` Equal to
* `!=` Not equal to

### Logical Operators

* `&&` And
* `||` Or
* `!` Not

### String Concatenation

* `+` can be used to join strings

```js
"Hello" + "World" // => "HelloWorld"
```

---

## 🧠 Control Structures

### Conditional Branches

```js
if x > 10 { ... } else { ... };
```

### Loops

```js
while x > 0 { x = x - 1; };
```

> ⚠️ Note: Each code block `{ ... }` must also end with a semicolon `;`.

---

## 🔢 Built-in Constants

| Constant Name | Description                                |
| ------------- | ------------------------------------------ |
| `ORIGINAL_W`  | Original width of the configuration area   |
| `ORIGINAL_H`  | Original height of the configuration area  |
| `CURSOR_X`    | X-coordinate of the cursor inside the mask |
| `CURSOR_Y`    | Y-coordinate of the cursor inside the mask |

> Constants are updated each time the script runs and remain fixed during execution.

---

## ⚙️ Built-in Functions

### `print(...)`

Outputs log messages (arguments are automatically converted to strings):

```js
print("Value:", x); // Output: "Value: 100"
```

### `wait(ms)`

Pauses execution for the specified number of milliseconds:

```js
wait(1000); // Wait for 1 second
```

### `tap(pointer_id, x, y, action?)`

Simulates a touch event:

* `pointer_id`: Touch point ID (non-negative integer)
* `x, y`: Relative coordinates (based on `ORIGINAL_W` / `ORIGINAL_H`)
* `action`: `"down"`, `"up"`, `"move"`, `"default"` (default triggers a down then up after 30ms)

### `swipe(pointer_id, interval, x1, y1, x2, y2, ...)`

Simulates a swipe gesture:

* `interval`: Time (ms) between consecutive points
* Requires at least two coordinate pairs (`x1, y1, x2, y2`)

### `send_key(key_name, action?, metastate?)`

Sends a key event:

* `key_name`: Key name (e.g., `"KEYCODE_HOME"`)
* `action`: `"down"`, `"up"`, `"default"` (default presses and releases the key)
* `metastate`: Modifier key (e.g., `"META_SHIFT_ON"`)

### `paste_text(text)`

Pastes the given text into the device:

```js
paste_text("Hello from script!");
```

---

## ⚠️ Error Handling

* The script reports syntax and runtime errors with detailed context:

  ```text
  error: Division by zero
   --> line 5, column 10 to line 5, column 15
    |
  5 |     10 / 0
    |      ^^^^^
  ```

---

## 🚫 Limitations & Notes

* User-defined functions are **not supported**
* All variables are **global** (accessible outside their declaring block)
* `send_key`’s `key_name` and `metastate` must conform to the enums defined in
  [src/scrcpy/constant.rs](src/scrcpy/constant.rs)

---

## 💡 Example Script

```rs
// Declare and initialize variables
let x = ORIGINAL_W / 2;
let y = ORIGINAL_H / 2;
let counter = 0;

// Use built-in constants
print("Original size:", ORIGINAL_W, "x", ORIGINAL_H);
print("Cursor position:", CURSOR_X, CURSOR_Y);

// Conditional example
if CURSOR_X > ORIGINAL_W / 2 {
    print("Cursor is on the right side");
} else {
    print("Cursor is on the left side or middle");
};

// Loop example
while counter < 3 {
    tap(counter, x, y);     // Tap at current position
    x = x + 100;            // Update variable
    counter = counter + 1;
    wait(500);              // Wait for a while
};

// String example
let message = "Hello" + " " + "World";
print(message);

// Swipe example: from center to upper-right
swipe(0, 500, ORIGINAL_W/2, ORIGINAL_H/2, ORIGINAL_W/2 + 200, ORIGINAL_H/2 - 200);

// Paste text example
paste_text("Hello from script!");

// Key event example
send_key("VolumeUp"); // Press and release the volume key

// Example using modifier key
send_key("A", "default", "CTRL_ON");

// Example controlling key press duration
send_key("Home", "down");
wait(100);
send_key("Home", "up");

// Logical operations
let flag = true;
if flag && counter > 0 {
    print("Flag is true and counter is positive");
};

if !flag || counter == 3 {
    print("Either flag is false or counter equals 3");
};

// Numeric comparison
if x > ORIGINAL_W / 2 && y < ORIGINAL_H / 2 {
    print("Position is in the upper right quadrant");
};
```