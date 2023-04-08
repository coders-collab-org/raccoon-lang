# Language Specification

## Types

### Primitive Types

#### Integer

- `int` - 64 bit signed integer

```rust
let variable = 2;
```

#### Floating Point

- `float` - 64 bit floating point

```rust
let variable: float = 2;

// or
let variable = 2.0;
```

#### Boolean

- `bool` - boolean type

```rust
let variable: bool = true;

variable = false;
```

#### string

- `str` - string type

```rust
let variable: str = "hello world";
```

## Variables

### Declaration

- `let` - declare a variable

```rust

let variable = 2;

```

### Assignment

- `=` - assign a value to a variable

```rust

let variable = 2;

variable = 3;

```

### Constant

- `const` - declare a constant

```rust

const CONSTANT = 2;

```

## Functions

### Declaration

- `fn` - declare a function

```rust

fn main() {
    println("hello world");
}

```

### Parameters

- `()` - function parameters

```rust

fn add(a: int, b: int) {
    println(a + b);
}

```

### Return

- `->` - function return type

```rust

fn add(a: int, b: int) -> int {
    return a + b;
}

```

## Control Flow

### if

- `if` - if statement

```rust

let variable = 2;

if variable == 2 {
    println("variable is 2");
} else {
    println("variable is not 2");
}

```

### match

- `match` - match statement

```rust

let variable = 2;

match variable {
    1 => println("variable is 1"),
    2 => println("variable is 2"),
    _ => println("variable is not 1 or 2"),
}

```

### loop

- `loop` - loop statement

```rust

let variable = 0;

loop {
    println(variable);
    variable += 1;
    if variable == 10 {
        break;
    }
}

```

### while

- `while` - while statement

```rust

let variable = 0;

while variable < 10 {
    println(variable);
    variable += 1;
}

```

### for

- `for` - for statement

```rust

for variable in 0..10 {
    println(variable);
}

```

## Comments

### Single Line

- `//` - single line comment

```rust

// this is a single line comment

```

### Multi Line

- `/* */` - multi line comment

```rust

/*
this is a multi line comment
*/

```

## Operators

### Arithmetic

- `+` - addition
- `-` - subtraction
- `*` - multiplication
- `/` - division
- `%` - modulus

### Comparison

- `==` - equal
- `!=` - not equal
- `>` - greater than
- `<` - less than
- `>=` - greater than or equal
- `<=` - less than or equal

### Logical

- `&&` - and
- `||` - or
- `!` - not

### Bitwise

- `&` - bitwise and
- `|` - bitwise or
- `^` - bitwise xor
- `~` - bitwise not
- `<<` - bitwise left shift
- `>>` - bitwise right shift

### Assignment

- `=` - assign a value to a variable
- `+=` - add and assign
- `-=` - subtract and assign
- `*=` - multiply and assign
- `/=` - divide and assign
- `%=` - modulus and assign
- `&=` - bitwise and and assign
- `|=` - bitwise or and assign
- `^=` - bitwise xor and assign
- `<<=` - bitwise left shift and assign
- `>>=` - bitwise right shift and assign

## Array

### Declaration

- `[]` - declare an array

```rust

let array: int[] = [1, 2, 3];

```

### Access

- `[]` - access an array

```rust

let array = [1, 2, 3];

println(array[0]); // 1

```

### Slice

- `[]` - slice an array

```rust

let array = [1, 2, 3];

println(array[0..2]); // [1, 2]

```

## Struct

### Declaration

- `{}` - declare a struct

```rust

struct Person {
    name: str,
    age: int,
}

let person: Person = {
    name: "John",
    age: 20,
};

```

- `()` - declare a struct with a constructor

```rust

struct Person(str, int);

let person = Person("John", 20);

```

### Access

- `.` - access a struct

```rust

let person: Person = {
    name: "John",
    age: 20,
};

println(person.name); // John

```

## Enum

### Declaration

- `enum` - declare an enum

```rust

enum Color {
    Red,
    Green,
    Blue,
}

```

### Access

- `::` - access an enum

```rust

let color = Color::Red;

println(color); // Red

```

## Modules

### Declaration

- `mod` - declare a module

```rust

mod module_name {
    // module code
}

```

### Import

- `use` - import a module

```rust

use file_name::module_name;

```

## Standard Library

### println

- `println` - print to the console

```rust

println("hello world");

```

### print

- `print` - print to the console without a newline

```rust

print("hello world");

```
