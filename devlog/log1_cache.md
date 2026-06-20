# Dev Log 1: Cache

A CPU has an optional component called the cache. This is a (often) multi-level piece of memory that sits right below the registers and ahead of the DRAM in the memory hierarchy. It usually sits on the chip with the CPU, and is relatively speaking, very fast. It is limited in its capacity though.

We want to add this same functionality to our emulator. Obviously for this case, we do not care about performance, we add this cache instead to better understand how the cache works and to more accurately emulate a modern CPU.

## Where do we start?

As mentioned before, caches are usually multi-level. Often, this is a 3 level cache. Each level is generally referred to as L1, L2, and L3 cache with L1 being the smallest and fastest and L3 being the largest and slowest.

There is a **LOT** of complexity when it comes to caches, more than I will go over here. To get started, lets simplify things.

We will only use a single level cache. It will be a two-way cache with 2B cache lines. It will have a total capacity of 16 cache lines, or 32 bytes. It will be a write-through policy cache with a random eviction algorithm. We will explore these properties more later.

What we want, in effect, is an auxilliary place we can check for data prior to checking DRAM. If it is present, we have a "cache hit" and can skip going to DRAM and the real world performance hit that would incur. If it is missing we must first retrieve the data from DRAM then populate the cache with the cache line. That way, if we then access that same data again immediately after, we can reap the benefits of our cache.

## The Cache

### Instructions

Our cache will be automatically handled by the "hardware". We won't need any unique instructions to modify or leverage the cache. Any memory access in any instruction will automatically use the cache.

### Data Model

```rust
pub type CacheLine = u16;
const WAYS: usize = 2;
const SETS: usize = 8;

pub struct CacheEntry {
    tag: u8,
    val: CacheLine,
}

pub struct Cache {
    inner: [[Option<CacheEntry>; WAYS]; SETS],
}
```

This will be the data model for our initial cache implementation. We have a 2-way, 8-set cache with a two byte cache line. That works out to `8 * 2 * 2 = 32` byte cache capacity.

Let's learn more about what ways mean in caches.

Ways are so called because it is the number of "ways" a single address could be stored in a cache. To understand what that means, lets see how an address translates to a cache entry.

| Bits   | Field  |
| ------ | ------ |
| [7, 4] | Tag    |
| [3, 1] | Index  |
| [0, 0] | Offset |

```rust
#[bitfield(u8)]
pub struct CacheAddr {
    #[bits(1)]
    pub offset: usize,
    #[bits(3)]
    pub index: usize,
    #[bits(4)]
    pub tag: u8,
}
```
Recall that we had a cache line size of two bytes. This dicates how large our offset field is. Since we only have two bytes, we only require one bit to represent which of those two bytes we care about.

Next we have the index. This size is determined by how many sets we have. We had eight, so we need 3 bits to represent that.

Finally we have the tag. Here we have what is called a PIPT cache, physically indexed, physically tagged. That means that we derive both the index and the tag from the physical address. This is owed to the fact that we don't yet support virtual addresses. For this reason, we just use the final 4 bits as our tag. That way, our 8 bit physical address is encoded completely in one way or another in each byte in our cache.

There is an issue though. Two different physical addresses could share the same index!

Say for example:

```
0bXXXX_010_X
0bYYYY_010_Y
```

These two addresses share the same index! So where do we put them? That's where ways come in. Ways are the number of different "ways" that a cache line can be stored in the cache. Since we have a wayness of 2, we can actually store both of these pieces of data in the cache. When we want to look up, say, the Y address in the cache later we will index to `0b010`, then iterate over the ways until we find a matching tag.

What if we have a third address `0bZZZZ_010_Z`? We already populated our 2 ways for that index?

### Eviction Strategy

We must evict one of the cache lines to make space for this new one. I will keep this short and simple. For now, we will just perform a random eviction. We will just choose, at random, a number between 0 and 1, and evict the cache line at that way and insert our new cache line.

[Next DevLog](./log2_tui.md)
