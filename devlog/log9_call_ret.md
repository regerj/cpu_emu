# Dev Log 9: Call and Return

Given that we have completed support for the stack, we now can support function calling and returning. We will discuss what role the stack plays soon, but this will bring in again the idea of stack frames.

Calling and returning is one of the most fundemental ideas in computer programming. They are essentially a slightly more featureful set of `jmp` instructions. They enable you to jump to predefined routines, which can then return to the point of invocation when complete, optionally returning some data via some calling convention, whether that be on the stack or in some register or registers.

## Design

First, let's decide on a set of instructions which we will support. For that, its pretty simple.
- `cal`: Call a procedure located at the given address
- `ret`: Return from the current procedure to the last `cal` location

To see how we want to use this, lets write some of our `asm` for a kind of function example. First, lets write a function or procedure snippet.

```asm
// -- snip --

// Increments the value in $r0
INC_R0:
add $r0,1
ret
```

This snippet defines a procedure identified by its label: `INC_R0`, which does exactly what it sounds like: it increments `$r0` register by one, then returns. Pretty basic, but it will work for our use case. 

Let's look at what the C alternative might look like:

```c
int inc_r0(int a) {
    return a + 1;
}
```

Next, let's see how we might `cal` this procedure.

Say we wanted to do something like this:

```c
int inc_r0(int a) {
    return a + 1;
}

void main() {
    int count = 0;
    count = inc_r0(count);
    count = inc_r0(count);
}
```

What would that look like?

```asm
mov $r0,0
cal INC_R0
cal INC_R0
end

// Increments the value in $r0
INC_R0:
add $r0,1
ret
```

Alright so the first `cal` instruction jumps to `INC_R0:`, which `ret`s to the second `cal` which jumps back to `INC_R0:` which then `ret`s to the `end` instruction.

## Implementation

Alright, so how do we actually implement that? First we can just add the two instructions to our ISA. Then, in the CPU instruction executor, we can add two more branches to our match to handle them.

`cal` and `ret` are, again, just slightly more advanced `jmp`s. We just need to know where to `jmp`. For `cal`, we will accept an argument that will represent the address we need to jump to, so that one is solved. But `ret` doesn't accept an argument. So how do we know where to jump back to?

That's where the stack comes in. In the `cal` instruction handler, we will first push the address of the next instruction onto the stack before jumping to the procedure location. That means that, so long as the procedure maintains it's stack properly, in our implementation of `ret` we can `pop` the return address off of the stack and place it into the `IP` register / jump to it.

To see how this works, let's map it out.

Take this simple program. We just push two values onto the stack then cal a function, which returns then the program ends.

```asm
psh 13   <-          // Stack  
psh 37               //  -------------
cal FOO              //  |    ...    |
end                  //  -------------

FOO:
ret
```

```asm
psh 13               // Stack  
psh 37   <-          //  -------------
cal FOO              //  |    13     |
end                  //  -------------

FOO:
ret
```

```asm
psh 13               // Stack  
psh 37               //  -------------
cal FOO  <-          //  |    13     |
end                  //  -------------
                     //  |    37     |
FOO:                 //  -------------
ret
```

```asm
psh 13                   Stack  
psh 37                   -------------
cal FOO                  |    13     |
end  <========||         -------------
              ||         |    37     |
FOO:          ||         -------------
ret      <-   ========== |    &end   |
                         -------------
```

```asm
psh 13                   Stack  
psh 37                   -------------
cal FOO                  |    13     |
end      <-              -------------
                         |    37     |
FOO:                     -------------
ret
```

As you can see, the `cal` pushes an address onto the stack and the `ret` pops that address off of the stack in order to know where to jump back to.

So to cap off, `jmp` and `cal`/`ret` can absolutely coexist, they do and they should. They each have their own use cases.

`jmp` is useful for creating things like loops, conditionals, and `goto` statements.

`cal`/`ret` is useful for mimicking function calls in C.

[Next DevLog](TODO)
