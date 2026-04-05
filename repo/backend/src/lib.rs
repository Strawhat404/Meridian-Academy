/// Backend library — exposes production modules for testing.
pub mod models;

// Re-export the middleware module only when needed; it depends on Rocket types
// which aren't useful for unit testing. The models module is what unit tests need.
