use dowe_icons::{GenerateIconOptions, IconRounded, IconTarget, generate_project_icons};
use icns::{IconFamily, IconType, PixelFormat};
use ico::IconDir;
use std::fs;
use std::io::BufReader;
use tempfile::TempDir;

include!("support/icon_test_helpers.rs");
include!("support/icon_generation_tests.rs");
include!("support/icon_geometry_tests.rs");
