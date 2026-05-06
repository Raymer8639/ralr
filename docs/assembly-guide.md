# 汇编语言指南

## 源文件格式

`.ralr` 文件是 ralr 的汇编源文件，使用纯文本格式。每条指令以分号 `;` 结尾，支持在一行内编写多条指令。

### 基本结构

```
指令 操作数1 操作数2 $目标寄存器;
```

- **指令**：`add`、`sub`、`mul`、`div` 之一
- **操作数1**：立即数或寄存器
- **操作数2**：立即数或寄存器
- **目标寄存器**：必须是一个寄存器（`$a1` 到 `$a5`），运算结果写入此寄存器

### 示例：基本运算

```
add 1 2 $a1;
```

这条指令将 `1` 和 `2` 相加，结果 `3` 存入寄存器 `a1`。

### 示例：多条指令

```
add 1 2 $a1;
sub $a1 1 $a1;
mul $a1 2 $a1;
div $a1 2 $a1;
```

上述代码的计算过程：`((1 + 2) - 1) × 2 ÷ 2 = 2`，最终结果存入 `$a1`。

### 示例：多行写法

以下写法与上面等价：

```
add 1 2 $a1
sub $a1 1 $a1
mul $a1 2 $a1
div $a1 2 $a1
```

（注意：虽然代码示例中将指令写在多行，但每条指令仍然需要以分号结尾。）

## 编译为二进制

使用 `ralr-asm` 将 `.ralr` 源文件编译为 `.abin` 二进制文件：

```bash
# 编译单个文件，输出默认名 output.abin
ralr-asm input.ralr

# 指定输出文件名
ralr-asm input.ralr -o program.abin

# 编译多个文件（合并到一个输出）
ralr-asm file1.ralr file2.ralr -o combined.abin
```

### 示例：打印输出

```
_println "hello";
_print "world";
_print "\n";
```

`_println` 打印后换行，`_print` 打印后不换行。字符串参数必须用双引号 `"` 包裹。

字符串内可以使用转义字符，如 `\n`（换行）、`\t`（制表）、`\\`（反斜杠）、`\"`（双引号）等：

```
_println "hello\nworld";
_print "tab\there";
```

输出：
```
hello
world
tab	here
```

## 运行二进制

使用 `ralr` 运行 `.abin` 二进制文件：

```bash
ralr program.abin
```

## 完整工作流

```bash
# 1. 编写汇编源文件
echo 'add 10 20 $a1;' > test.ralr

# 2. 编译
ralr-asm test.ralr -o test.abin

# 3. 运行
ralr test.abin
```
