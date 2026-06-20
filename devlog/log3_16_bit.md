# Dev Log 3: 16 Bit

8-bit is a fairly restrictive address space to use. In the beginning it made things easy, but as we scale up we will find this word size to be too small. For that reason I am going to move us to 16 bit.

## Design

First of all, we are going to take a little detour. One of the stretch goals here is to have a somewhat configurable CPU. This will enable use cases like turning knobs in order to plot tradeoffs in performance.

### Configuration

There are two kinds of configuration. There is boot-time configuration and there is compile time configuration. We will semi-arbitrarily (mostly on constness need) sort our possible configurations into either category.

For now, our sorting will be simple. Latencies will be boot-time configurations. Hardware specs will be compile-time configurations.

Configuration will be globally accessible that way they can be pulled from different modules and will be centralized. Maybe this is good, maybe it is bad. Until it is painful it will be true. This will be in the `crate::cfg` module. The compile-time configurations will be present in a const `CConfig` struct. The boot-time configuration will be available via a static `Config` struct, and read in from a `machine.toml` file in the current directory. Maybe I will make this more flexible in the future, but not right now...

Earlier, I lied. There is technically a third kind of configuration which will be type-relevant configurations. These will of course be compile-time. These will be literal `pub type X...` lines in our `crate::cfg` module.

The reason we go on this detour is that we will be defining the CPU word size as one of these configurations. This will allow great flexibility in changing it in the future, though I doubt we will ever move to 32-bit. 16-bit should be sufficient to satisfy the purposes of this project.

### The Rest of It

The rest should kind of fall into place. Once we have this centralized configurations, just changing it here should reflect all over the codebase. I implemented some logic around cache line sizes (another compile-time type style configuration) intelligently, so they should adapt to changing cache line sizes.

The only other weak point is actually in the TUI / visualization implementations. Currently I am padding the registers and such with the appropriate number of zeroes for the word size. This is hard coded though because it is part of the format specifier. I need to look into whether or not this can be done differently. To be determined...

[Next DevLog](TODO)
