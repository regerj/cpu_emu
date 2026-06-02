# Machine Architecture Emulator

This is a crude emulator of a standard machine architecture. It is a toy machine with a toy CPU as a means to explore computer architecture. 

## Architecture

```mmd
flowchart LR
    subgraph Group 1 [CPU]
        Core <--> mc[Memory Controller]
    end
    mc <-->|Memory Bus| DRAM
```

## Current Features

- CPU
- Rudimentary instruction set
  - add
  - sub
  - mov
- DRAM

## Planned Features

- CPU Cache
- Virtual Memory / MMU
- Telemetry
