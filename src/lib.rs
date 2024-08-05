// TODO: expose less?

// Code shared between different Tabry components (language, app, engine, etc.)
// Mainly having to do with types used across all of tabry
pub mod core;

// Code which takes a tabry config and arguments to be parsed, parses command line arguments
// (identifying flags/arguments/subcommands) and generates options. The "machine" is the core of
// this.
pub mod engine;

// Higher-level code used in the `tabry` options-finding application.
pub mod app;

// The tabry language parser (compiler)
pub mod lang;
