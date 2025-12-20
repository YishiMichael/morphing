mod scene;

extern crate self as morphing;

pub use scene::Lifecycle;

pub use morphing_macros::{chapter, fp, scene, FieldIndex};

// For macro invocation internal usage
#[doc(hidden)]
pub use morphing_macros as __macros;
