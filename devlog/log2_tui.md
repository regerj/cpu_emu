# Dev Log 2: TUI

Recall that the entire point of this project is to better understand the workings of the CPU.

While programming this of course has taught me much, the gold standard would be an interactive, visual experience. For that reason, we will be creating a TUI for this emulator.

This will serve two purposes: visualizing the state of the CPU to improve understanding and acting as a sort of debugger during development :).

## Design

### TUI Framework

We start with the [`ratatui`](https://ratatui.rs) crate. I am not going to go over how to use this crate, thats well outside the scope of this devlog. You can read plenty about it on their website.

### Visual Content

We want to display a live feed of the instructions similar to a normal debugger like `gdb`. We want to display the current content of the CPU registers, we want to display the current contents of the cache, and we want to display the current contents of the memory.

Some of this may not scale as we (spoilers) expand the capabilities of the CPU including 16-bit memory space, disk, and more, but we will cross that bridge when we arrive.

### Interactivity

We also want some level of interactivity. For now, this will be restricted to stepping instruction by instruction through the program we are running. This will allow us to incrementaly examine the state of the machine.

In the future we may add more including editing components, adding breakpoints and run/continue commands, restarting, and more.

## Implementation Details

The main thread runs the CPU itself and executes the instructions as they arrive. For now, it will also be responsible for rendering the TUI. That means that for the components owned by this thread, we can implement `ratatui`'s `Widget` or `StatefulWidget` trait directly.

However, this reveals a more annoying problem, we cannot do the same thing for DRAM because our DRAM runs in a child thread since it is considered a distinct execution block. Of course I know everything in a real CPU is more or less a distinct execution block but damn it this is a pet project. I am not reinventing a bit-perfect x86 CPU.

Anyway, there are a few approaches we could take to fix this. We could use:

1. A multi-producer, multi-consumer strategy to the memory bus where any component that can issue a command to the DRAM (here only the CPU, but in the future it may be a DMA controller) will replicate that message to any and all endpoints. Those endpoints could be the DRAM itself as well as our widget version of the DRAM in the main thread.

This is not possible with regular Rust channels as they are designed for work distribution. The first receiver to `.recv()` the message will consume it and all other receivers will not see it.

2. An array of channel transmitters in the CPU. The CPU, when issuing an access to the DRAM, would iterate a vector of transmitters and transmit on all of them. This is highly unintuitive especially considering it is expecting a response from only one of them.

3. A DRAM mirror. This option tools in an optional channel connection between the DRAM and a DRAM mirror struct where the DRAM can replicate a subset of its received operations (here just writes) to the mirror and the mirror can store its own copy of the data and stay in sync.

I am going to use a DRAM mirror. Yes we double the required memory to represent the machine memory, but we are in 8 bit and maybe soon 16 bit address space and I do not care. This makes the most sense to me, and its not even in the critical path of the CPU itself, its just for TUI.

If/when I add a memory bus arbitration block, this may change.

[Next DevLog](./log3_tui.md)
