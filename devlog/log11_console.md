# Dev Log 11: Console

It would be nice to be able to provide a tangible output mechanism for our machine. For that purpose, we will add a basic console to our machine!

Consoles are text-only (kind of) displays that can provide visual information to the user. We will create an extremely simple and limited console to the same end.

## Design

The way we are going to do this will likely change in the future, but for now, a specific region of physical memory will be mapped as a console buffer that can be modified and in the `ratatui` interface, we can display that buffer as the console, interpretting the bytes inside as ASCII characters.

At some point it might be nice to support more than just ASCII, but we will keep things simple for now. It will also be nice in the future to more accurately model DMA for this purpose, but we kind of have a very shallow model of DMA with this approach.

This does not require any new instructions or registers, we just need to modify our physical memory map to allocate a region for the console buffer.

The implementation of this also allows us to write our very first Hello World!

```asm
MAIN:
// Setup idt
mov *0,SYS_CALL_PUTC
mov $it,0

// He
mov $r0,25928
mov $r1,0
sys 0

// ll
mov $r0,27756
mov $r1,2
sys 0

// 'o '
mov $r0,8303
mov $r1,4
sys 0

// Wo
mov $r0,28503
mov $r1,6
sys 0

// rl
mov $r0,27762
mov $r1,8
sys 0

// d!
mov $r0,8548
mov $r1,10
sys 0
end

// $r0: char
// $r1: buf_idx
SYS_CALL_PUTC:
// Calculate address in display buffer
mov $r2,57344
add $r2,$r1
// Write
mov *$r2,$r0
ret
```

This leverages our IDT / syscall work from the previous iteration to define a basic `PUTC` function, and then use that function to print to our screen. One oddity is that it requires that characters be provided in pairs, since our word size is 16-bit. I need to do some more research into whether or not this is accurate to the real world, or if I need to design a way to allow it to accept byte by byte.

For now, I am happy to say that we have a working Hello World!

[Next DevLog](TODO)
