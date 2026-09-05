# Clipboard Manager

A lightweight, modular clipboard manager I'm building in Rust. I designed the architecture to be fast and memory-efficient. 

# Architecture & Internal Libraries

Instead of a monolithic design, I split the workspace into several decoupled crates (libraries) to keep the logic isolated.  
- ClipboardListener: Hooks into the OS to monitor and capture new clipboard events.
- ClipboardInstance: Defines the core data structures, using traits (instance_trait.rs) and implementations (instance_impl.rs) to handle individual copied items.
- CircularBuffer: My custom circular buffer implementation (circular_impl.rs) to manage the clipboard history. It automatically overwrites the oldest entries once the capacity is reached, preventing memory bloat.
- NotificationManager: Handles system alerts and feedback via pager traits (pager_trait.rs, pager_impl.rs).
- Prototype: The main executable that wires all these internal libraries together (circular, instance, and pager) into a functional application.
