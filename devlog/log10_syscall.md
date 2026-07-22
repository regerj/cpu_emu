# Dev Log 10: Interrupts and Syscalls

We also want to add some early tooling for interrupts. This is not a completely accurate implementation, as we do not yet have the idea of contexts, memory maps, or priviledge levels.

## Design

So interrupts work via a specific table of handlers. This is called the "interrupt descriptor table" or IDT for short. This table is located in memory and a pointer to that table is generally stored in a special CPU register. There is a special instruction, sometimes called `int` that is passed an interrupt number to represent what kind of interrupt is being triggered.

This results in the CPU indexing its IDT by that number and executing the function pointer that exists in that entry. This allows any code to execute interrupts, which often involve a context switch to the kernel. This is how syscalls are implemented.

We have no construct of priviledge levels, context, or virtual memory mapping yet, so this is largely just a slightly abstracted function call. However, we will tool it up now, and when we add support for those things, they will be added to this functionality.

The implementation will be fairly simple. We will add a special register that can hold the address of the IDT, then we can add an instruction to trigger an interrupt, and finally we can write an example of a syscall being registered and called.

```asm
MAIN:
// Build our IDT
mov *0,SYSCALL_0
mov *2,SYSCALL_1
mov $it,0

int 0
int 1
end

SYSCALL_0:
mov $r0,0
ret

SYSCALL_1:
mov $r0,1
ret
```

As you can see, our register will be called `$it` and our instruction will be called `int`. In reality, the `$it` register should not be accessible directly through `mov`, instead it is accessed through a specialized instruction `lidt` which is gated behind priviledge level. Since we are just building things simply, it is fine to not have an explicit instruction for modifying the IDT.

[Next DevLog](TODO)
