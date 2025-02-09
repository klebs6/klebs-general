// ---------------- [ File: mock_crate/src/lib.rs ]

        /// A function that adds two numbers.
        pub fn add(a: i32, b: i32) -> i32 {
            a + b
        }

        /// A public struct.
        pub struct Point {
            x: i32,
            y: i32,
        }
             }

        // Private struct should not be included
        struct Hidden;
            };
        }
        