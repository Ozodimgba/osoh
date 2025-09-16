#[macro_export]
macro_rules! define_methods {
    ($(($name:ident, $id:expr)),*) => {
        // Generate all the constants
        $(pub const $name: u16 = $id;)*

        // Generate the validation function - THIS IS THE IMPORTANT PART
        pub fn is_valid(method_id: u16) -> bool {
            matches!(method_id, $($id)|*)
        }

        // Optional: Generate a list for debugging
        pub fn all_valid_methods() -> &'static [u16] {
            &[$($id),*]
        }
    };
}
