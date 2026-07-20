# Dev Log 7: The Stack

So one of the most primitive memory constructs of modern CPUs is the "stack". This is a growable and shrinkable portion of contiguous memory that programs can use for local variables. It incurs minimal to no overhead for allocation and freeing, and is highly performant. Right now, we can only use random chunks of DRAM or the available registers to store and perform calculations. Let's change that by adding support for a stack.

## Design

So a stack looks something like this:

```c
void foo() {
    uint32_t a = 13;
    uint32_t b = 37;
}
```

```txt
-------------
|   a: 13   |
-------------
|   b: 37   |
-------------
|    ...    |
-------------
```

With the stack growing downwards (I know, counterintuitive, but that is the norm). How do we track *where* the stack is? Let's update our drawing.

```txt
------------- <- $bp
|   a: 13   |
-------------
|   b: 37   |
------------- <- $sp
|    ...    |
-------------
```

We use two special CPU registers, often called the stack base pointer and stack pointer. In x86 they are called `$rbp` and `$rsp`. The base pointer points to the bottom of the current stack frame, while the stack pointer points to the top of the current stack frame. Yes, I know that these are inverted from the visual / in-memory representation, blame the people who decided this ought to be the direction and terminology.

Stack frames, which I referenced earlier, represent a new "block" of data pushed onto the stack. This is how we scope local variables. The current stack frame encompasses the memory between the base pointer and the stack pointer.

Alright, so we need to add these two pointer registers that can hold the current stack frame bounds. What else?

We want a convenient way to add and remove values from the stack. This is, by convention, called pushing and popping.

So, we will also add two new instructions, `psh` and `pop` that will push and pop values onto and off of the stack.

## Infra Improvements

Finally, we will also throw together some basic procedural macros to lower the number of manual modification locations necessary to add new registers / instructions. You can read the diff for these, they are fairly basic.

[Next DevLog](./log8_end.md)
