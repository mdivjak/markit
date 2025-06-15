1. I prefer fail fast approach when developing apps. Use as many assert as possible to quickly find bugs.
2. Add unit tests for everything.
3. When writing Rust do not use mod.rs files - instead have a .rs file of the same name as the module and a folder of the same name with all module files in that folder.
4. Use instrumentation and tracing extensively throughout the application. For instrumentation use the tokio tracing crate.
5. When implementing tasks split them into smallest possible parts. Ideally you should only edit one file at a time and add unit tests. Prioritize micro changes that I can review easily.