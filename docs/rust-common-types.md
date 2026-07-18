# Rust 常见类型速查

本文档汇总 Rust 中常用的基础类型、复合类型、集合类型、智能指针与特殊类型，供丹青项目参考。

---

## 一、标量类型（Scalar Types）

| 类型 | 说明 | 示例 |
|---|---|---|
| `i8` / `i16` / `i32` / `i64` / `i128` | 有符号整数 | `let x: i32 = -42;` |
| `u8` / `u16` / `u32` / `u64` / `u128` | 无符号整数 | `let x: u32 = 42;` |
| `isize` / `usize` | 指针大小的有符号/无符号整数，常用于索引、长度 | `let i: usize = arr.len();` |
| `f32` / `f64` | 单精度 / 双精度浮点数 | `let x: f32 = 3.14;` |
| `bool` | 布尔值 | `let ok: bool = true;` |
| `char` | Unicode 标量值，4 字节 | `let c: char = '丹';` |
| `()` | 单元类型（unit），表示“没有返回值” | `fn foo() {}` 实际返回 `()` |

### 整数范围

```rust
u8   // 0 ~ 255
u16  // 0 ~ 65535
u32  // 0 ~ 4_294_967_295
u64  // 0 ~ 18_446_744_073_709_551_615

i8   // -128 ~ 127
i32  // -2_147_483_648 ~ 2_147_483_647
i64  // -9_223_372_036_854_775_808 ~ 9_223_372_036_854_775_807
```

---

## 二、复合类型（Compound Types）

| 类型 | 说明 | 示例 |
|---|---|---|
| `(T, U, V)` | 元组，固定长度，可异构 | `let t = (1, "a", true);` |
| `[T; N]` | 数组，固定长度，同类型 | `let arr = [1, 2, 3];` |
| `&[T]` | 切片，对数组/Vec 的引用 | `let s: &[i32] = &arr[1..3];` |
| `struct` | 结构体 | `struct Point { x: f32, y: f32 }` |
| `enum` | 枚举，可带数据 | `enum Msg { Click, Move(i32, i32) }` |

---

## 三、字符串类型

| 类型 | 说明 | 示例 |
|---|---|---|
| `&str` | 字符串切片，不可变，通常指常量或 `String` 的借用 | `let s: &str = "丹青";` |
| `String` | 堆分配的可变字符串，拥有所有权 | `let s = String::from("丹青");` |

---

## 四、集合类型（标准库）

| 类型 | 说明 |
|---|---|
| `Vec<T>` | 动态数组 |
| `HashMap<K, V>` | 哈希表 |
| `HashSet<T>` | 哈希集合 |
| `BTreeMap<K, V>` | 有序映射 |
| `BTreeSet<T>` | 有序集合 |
| `VecDeque<T>` | 双端队列 |
| `LinkedList<T>` | 链表 |
| `BinaryHeap<T>` | 优先队列 |

---

## 五、引用与智能指针

| 类型 | 说明 |
|---|---|
| `&T` | 不可变引用 |
| `&mut T` | 可变引用 |
| `*const T` / `*mut T` | 原始指针（unsafe 使用） |
| `Box<T>` | 堆分配，唯一所有权 |
| `Rc<T>` | 引用计数，单线程共享所有权 |
| `Arc<T>` | 原子引用计数，多线程共享所有权 |
| `Weak<T>` | 弱引用，避免循环引用 |
| `Cell<T>` / `RefCell<T>` | 运行时内部可变性（单线程） |
| `Mutex<T>` / `RwLock<T>` | 线程安全内部可变性 |
| `Cow<'a, B>` | 写时克隆（Clone on Write） |

---

## 六、特殊类型

| 类型 | 说明 | 示例 |
|---|---|---|
| `Option<T>` | 可能为空的值 | `Some(x)` / `None` |
| `Result<T, E>` | 可能失败的值 | `Ok(x)` / `Err(e)` |
| `PhantomData<T>` | 零大小类型，用于标记生命周期/类型关系 |
| `!` | Never type，表示函数永远不会返回 |

### Option 与 Result 示例

```rust
fn maybe_divide(a: f32, b: f32) -> Option<f32> {
    if b == 0.0 {
        None
    } else {
        Some(a / b)
    }
}

fn read_file(path: &str) -> Result<String, std::io::Error> {
    std::fs::read_to_string(path)
}
```

---

## 七、函数与闭包类型

| 类型 | 说明 |
|---|---|
| `fn(T) -> U` | 函数指针 |
| `impl Fn(T) -> U` | 不可变借用捕获的闭包 |
| `impl FnMut(T) -> U` | 可变借用捕获的闭包 |
| `impl FnOnce(T) -> U` | 获取所有权的闭包 |

---

## 八、泛型与生命周期

| 写法 | 说明 |
|---|---|
| `T`, `U`, `K`, `V` | 类型参数 |
| `<T: Trait>` | 带约束的泛型 |
| `'a`, `'static` | 生命周期参数 |
| `dyn Trait` | 动态分发 trait 对象 |
| `impl Trait` | 静态分发，返回/参数中隐藏具体类型 |

---

## 九、类型别名

```rust
type Point = (f32, f32);
type MyResult<T> = Result<T, MyError>;
```

---

## 十、与 Java 类型快速对照

| Rust | Java |
|---|---|
| `i32` | `int` |
| `i64` | `long` |
| `f64` | `double` |
| `bool` | `boolean` |
| `String` | `String` |
| `&str` | 字符串常量 |
| `Vec<T>` | `ArrayList<T>` |
| `HashMap<K, V>` | `HashMap<K, V>` |
| `Option<T>` | `Optional<T>` |
| `Result<T, E>` | 异常或自定义结果类型 |
| `enum` | 更强大的枚举（可带数据） |
| `struct` | `class`（无继承） |

---

## 参考

- [The Rust Programming Language - Data Types](https://doc.rust-lang.org/book/ch03-02-data-types.html)
- [Rust By Example - Primitives](https://doc.rust-lang.org/rust-by-example/primitives.html)