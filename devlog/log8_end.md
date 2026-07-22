# Dev Log 8: End

This will be a short one. There was a "bug" previously because after finishing the final instruction in the sequence would cause a crash as we would not be able to properly interpret the next instruction as it wasn't present in the SRAM.

To prevent this, we will just add an `end` instruction that indicates to the CPU to end execution. This will allow for a clean shutdown and cleanup of the CPU and hardware.

[Next DevLog](./log9_call_ret.md)
