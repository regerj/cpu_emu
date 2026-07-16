# Dev Log 5: Jmp

Ok, so the last devlog was in service of this goal, the ability to jump around in code. This builds the groundwork for loops and other more advanced constructs.

As a reminder, needed to implement SRAM so that our instructions had actual addresses that could be "jumped" to. We allocated the [0xF000-0xFFFF] address space to SRAM, and that's where our instructions will live.

## Design

Adding support for a `jmp` instruction (and other related instructions later) simply requires that we modify the `$IP` register that we created last time! Remember, that is where the CPU looks for the next instruction, so modification of it in one instruction can change where the next instruction will be read from, i.e. jump!

`jmp` should take only one argument, the address to which to jump. This will be the first instruction we support with a number of arguments other than two. But remember from last devlog, we set it up so that we can support any instruciton accepting [0-4] arguments, so this should be fine.

However, it would be awfully obnoxious to require users of our assembly language to hand calculate and maintain absolute addresses for their `jmp` instructions...

Instead we will implement support for a concept called "labels" which you will probably be familiar with if you've written industry C code or other assembly languages.

These labels look a bit like this simple spinlock example:

```asm
SPINLOCK:
jmp SPINLOCK
```

The label is a kind of preprocessor marker for a location in the binary. It can be used in place of an operand and the assembler will replace it with the actual address it corresponds to in the generated binary. This makes it auto-updating, and much easier to maintain.

For example, say that snippet constituted a simple program we were going to assemble. What address would `SPINLOCK:` correspond to?

Would it be `0x0000`? Maybe `0x0001`?

Neither actually, it would correspond to address `0xF000`. Why? Because remember, our SRAM is mapped physically to begin at address `0xF000` and this label is the first thing in the program. Thus, it will be processed as the address `0xF000`, which is also the address at which the only instruction: `jmp SPINLOCK` exists. This means we will just continuously jump back to the same instruciton: i.e. spin!

So what happens when we want to add some other instruction to our program?

```asm
add $r0,1
SPINLOCK:
jmp SPINLOCK
```

Now what does the `SPINLOCK:` label resolve to?

It resolves to `0xF006` because our `add $r0,1` instruction compiles into 6 bytes. This is all automatically calculated and processed for you by the assembler, meaning you can freely make changes and reordering to your assembly and not need to worry about recalculating and maintaining jump target addresses!

So how do we do this?

## Improving our Assembler

Our initial assembler was functional, but far too simple as our use case and feature set grows closer to real full assembly languages.

The industry standard for language processing is to first perform "tokenization", followed by a two-pass parsing and assembly process.

Don't worry, fellow AI luddites, we are not feeding our assembly into an LLM. They stole the concept from us! Tokenization is the breaking up of our input text stream into distinct typed tokens. For example, the following spinlock would break into:

```asm
SPINLOCK:
jmp SPINLOCK
```

```txt
Token::LabelDecl("SPINLOCK"),
Token::Mneumonic("jmp"),
Token::LabelInvoc("SPINLOCK"),
```

This process is called tokenization, or lexing, which just means to break into lexical tokens.

It is important to note that this lexing is distinct from the act of parsing, which we will talk about later. It does not validate the syntactical correctness of our assembly, it just breaks it up for easier processing by the parser which is responsible for that.

Next, we need to go over our parser. This is the block that is responsible for processing the tokens in our input, calculating labels, validating syntactical correctness, and producing our AST (abstract syntax tree). Though since we are in assembly, this is more or less just an array of instructions.

This parsing process will be two pass.

The first pass will be responsible for evaluating our labels and building a "label table" that will contain the mappings of named labels to locations in the final binary. To do this, we will keep a running tally of the byte location (called a location counter) as we iterate through the instructions. This stage will still not validate syntactical correctness, just build our map.

The second pass will be responsible for validating syntactical correctness, constructing the proper representation of the instruction, and replacing label uses with their mapped location in the binary. We will do this with essentially a state machine. We will track the last encountered token to know what kind of token to expect next. For example, after a `Token::Comma`, we expect some kind of operand token like a `Token::Deref`, `Token::Register`, `Token::LabelInvoc`, or `Token::Immediate`. As we parse through the tokens, when we encounter a mneumonic token, we will begin pushing these tokens onto a stack until we encounter a token that indicates that the previous instruction should be complete, i.e. another mneumonic or a label or a comment.

We can then evaluate the stack we have built so far for mneumonic specific correctness (correct number and kind of arguments, etc.). This will produce our `Operation` representation of a complete instruction or error. We can add this `Operation` to our array of operations and when we finish parsing all tokens, we can return this array.

Then finally, the assembler can take our parsed AST and compile it using the macro generated code we worked on in the previous post!

Et voila, we have a compiled binary that supports jumping and labels!
