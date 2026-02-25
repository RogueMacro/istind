# Language Specification

> **Status:** Draft — based on the tests in `tests/simple.rs`.

---

## 1. Lexical Structure

### 1.1 Whitespace

Spaces, tabs, and newlines are insignificant and used only to separate tokens.

### 1.2 Keywords

| Keyword  | Purpose                      |
|----------|------------------------------|
| `fn`     | Introduce a function item    |
| `let`    | Declare a local variable     |
| `return` | Return a value from a function |

### 1.3 Identifiers

An identifier begins with a Unicode letter or underscore (`_`) and is followed by zero or more letters, digits, or underscores.

```
identifier ::= [a-zA-Z_][a-zA-Z0-9_]*
```

### 1.4 Integer Literals

A sequence of one or more decimal digits representing a 64-bit signed integer.

```
integer_literal ::= [0-9]+
```

### 1.5 Operators and Punctuation

| Token | Description               |
|-------|---------------------------|
| `(`   | Left parenthesis          |
| `)`   | Right parenthesis         |
| `{`   | Left curly bracket        |
| `}`   | Right curly bracket       |
| `=`   | Assignment / equality     |
| `+`   | Addition operator         |
| `;`   | Statement terminator      |

---

## 2. Program Structure

A program is a sequence of top-level *items*. Currently the only supported item is a function definition.

```
program ::= item*
item    ::= function
```

### 2.1 Entry Point

A program must contain a function named `main`. Execution begins there.

---

## 3. Functions

```
function ::= "fn" identifier "(" ")" block
block    ::= "{" statement* "}"
```

A function has a name, an empty parameter list, and a block body. The block body contains zero or more statements.

**Examples**

```
fn main() {}
```

```
fn main() {
    return 0;
}
```

### 3.1 Implicit Return

A function with no `return` statement returns implicitly (the return value is implementation-defined).

---

## 4. Statements

```
statement ::= return_stmt
            | declaration_stmt
```

Every statement ends with a semicolon (`;`).

### 4.1 Return Statement

```
return_stmt ::= "return" expression ";"
```

Evaluates `expression` and returns the result to the caller.

**Example**

```
return 0;
return a;
return a + b;
```

### 4.2 Variable Declaration

```
declaration_stmt ::= "let" identifier "=" expression ";"
```

Binds a new local variable with the given name to the value of `expression`. A variable must be declared before it is used.

**Example**

```
let a = 2;
let b = 3;
```

---

## 5. Expressions

```
expression ::= integer_literal
             | identifier
             | expression "+" expression
```

Expressions are evaluated left-to-right. No explicit operator-precedence grouping syntax (parentheses around expressions) is currently defined.

### 5.1 Integer Literal

A constant 64-bit signed integer value.

**Example**

```
42
0
1
```

### 5.2 Variable Reference

An identifier that names a previously declared local variable.

**Example**

```
a
result
```

### 5.3 Addition

```
addition_expr ::= expression "+" expression
```

Adds two integer values and produces an integer result.

**Example**

```
a + b
2 + 3
```

---

## 6. Complete Example

The program below declares two local variables, adds them, and returns the result as the process exit code.

```
fn main() {
    let a = 2;
    let b = 3;
    return a + b;
}
```
