## Prush

Prush is Rust based interpreter for Push programs.

## What is Push?

Push is a stack-based Turing-complete programming language that enables autoconstructive evolution in its programs.
More information can be found [here](http://faculty.hampshire.edu/lspector/push.html).

## Supported Stack Types

This implementation supports all Push3 instructions for the types desribed in the [Push 3.0 Programming Language Description](http://faculty.hampshire.edu/lspector/push3-description.html#Type):

* BOOLEAN
* CODE
* EXECUTION
* FLOAT
* INTEGER
* NAME

Additionally, it provides the vector types for boolean, float and integer:

* BOOLVECTOR
* FLOATVECTOR
* INTVECTOR

The default instructions for vector types are dup, equal, flush, shove, stackdepth, swap, yank and yankdup. 


## Usage

The following example shows how to intepret Push program with Prush.

```rust
// Define Push program
let input = "( CODE.QUOTE ( CODE.DUP INTEGER.DUP 1 INTEGER.- CODE.DO INTEGER.* )
               CODE.QUOTE ( INTEGER.POP 1 )
               INTEGER.DUP 2 INTEGER.< CODE.IF )";

// Define State and Instruction Set
let mut push_state = PushState::new();
let mut instruction_set = InstructionSet::new();

// Load default instructions
instruction_set.load();

// Add program to execution stack
PushParser::parse_program(&mut push_state, &instruction_set, &input);

// Put initial values
push_state.int_stack.push(4);

// Run the program
PushInterpreter::run(&mut push_state, &mut instruction_set);
```

For existing types the instruction set can be extended by calling the ``add`` function.


```rust
pub fn my_instruction(_push_state: &mut PushState, _instruction_set: &InstructionCache) {
    // Does nothing
}

...

let mut instruction_set = InstructionSet::new();
instruction_set.add(String::from("MyInstruction"), Instruction::new(my_instruction));

```





