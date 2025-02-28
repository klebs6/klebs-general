# README

## Overview

The **hydro2-network-performance** crate provides performance tracking tools for monitoring and measuring the execution of networks in the `hydro2` ecosystem. It allows for tracking key statistics such as execution time, number of operators executed, and peak memory usage during the execution of a network.

### Key Features

1. **Execution Time Tracking**  
   The `PerformanceStats` struct tracks the time it takes for a network to run, capturing both the start and end time. It also provides the ability to calculate the total duration of network execution.

2. **Operator Execution Count**  
   The crate keeps track of the number of operators executed during the network's execution.

3. **Peak Memory Usage**  
   The `PerformanceStats` struct stores the peak memory usage in bytes, helping monitor the memory footprint during the execution.

4. **Start and End Timing**  
   The crate provides the ability to mark the start and end of the execution, with methods like `start()` and `end()` for tracking.

### `PerformanceStats` Struct

The **`PerformanceStats`** struct is the main entry point for tracking execution statistics. It is designed to start when the network execution begins, and it ends when you call the `end()` method.

### Methods

- **`start()`**  
  Initializes the `PerformanceStats` and records the current time as the start time of the execution.

- **`end()`**  
  Marks the end of the execution and records the current time as the end time.

- **`total_duration()`**  
  Returns the total execution duration as an `Option<Duration>`. If the execution hasn't ended, `None` is returned.

### Usage Example

Below is an example of how to use the `PerformanceStats` struct to measure network performance:

```rust
use hydro2_network_performance::PerformanceStats;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Start performance measurement
    let mut stats = PerformanceStats::start();
    
    // Simulate network execution (e.g., running a network of operators)
    // Here you can replace this with your network execution logic
    std::thread::sleep(Duration::from_secs(2)); // Simulate delay
    
    // Mark the end of the performance measurement
    stats.end();

    // Print the total execution time
    if let Some(duration) = stats.total_duration() {
        println!("Execution time: {:?}", duration);
    }

    Ok(())
}
```

In this example, the `PerformanceStats::start()` method begins tracking, and `PerformanceStats::end()` ends the tracking once the network execution is complete. The total duration is then printed.

---

## License

Distributed under the OGPv1 License (see `ogp-license-text` crate for more details).

## Repository

Hosted on GitHub:  
[https://github.com/klebs6/klebs-general](https://github.com/klebs6/klebs-general)
