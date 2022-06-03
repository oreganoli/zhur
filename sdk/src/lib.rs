/// This module contains crates the Zhur SDK uses, but you should not directly have to import.
pub mod __internals {
    pub use bincode;
    pub use wapc_guest;
}
/// Date and time handling methods.
pub mod datetime;
/// Key-value database methods.
pub mod db;
/// Error types.
pub mod error;
/// HTTP-related types.
pub mod http;
/// Metadata methods.
pub mod meta;
/// Basic "web framework"
pub mod web;
/// This macro sets up all the Zhur SDK boilerplate for a text function that takes a UTF-8 `String` and returns another.
#[macro_export]
macro_rules! text_function {
    ($function_name:ident) => {
        fn func_wrapper(msg: &[u8]) -> zhur_sdk::__internals::wapc_guest::CallResult {
            let msg_string = zhur_sdk::__internals::bincode::deserialize::<String>(msg)
                .expect("a text function expects a deserializable utf-8 string");
            let output = $function_name(msg_string);
            let response = zhur_sdk::__internals::bincode::serialize(&output)
                .expect("should be able to bincode-serialize a string");
            Ok(response)
        }
        #[no_mangle]
        pub fn wapc_init() {
            std::panic::set_hook(Box::new(|p| {
                zhur_sdk::__internals::wapc_guest::host_call(
                    "zhur",
                    "internals",
                    "panic",
                    p.to_string().as_bytes(),
                )
                .unwrap();
            }));
            zhur_sdk::__internals::wapc_guest::register_function("text", func_wrapper)
        }
    };
}

/// This macro takes any function that takes an `&HttpReq` and a `&mut HttpRes`, then generates the WAPC boilerplate for running it.
/// This lets you just focus on writing your app.
/// HTTP request deserialization, response serialization and all such are all taken care of here.
#[macro_export]
macro_rules! http_function {
    ($http_handler:ident) => {
        fn outer_handler(msg: &[u8]) -> zhur_sdk::__internals::wapc_guest::CallResult {
            let output = inner_handler(msg);
            let bytes = zhur_sdk::__internals::bincode::serialize(&output).unwrap(); // WE DO NOT EXPECT TO FAIL HERE
            Ok(bytes)
        }
        fn inner_handler(msg: &[u8]) -> zhur_sdk::http::HttpRes {
            let req: zhur_sdk::http::HttpReq =
                zhur_sdk::__internals::bincode::deserialize(msg).unwrap(); // WE DO NOT EXPECT TO FAIL HERE
            let mut res = zhur_sdk::http::HttpRes::default();
            $http_handler(&req, &mut res);
            res
        }
        #[no_mangle]
        pub extern "C" fn wapc_init() {
            std::panic::set_hook(Box::new(|p| {
                zhur_sdk::__internals::wapc_guest::host_call(
                    "zhur",
                    "internals",
                    "panic",
                    p.to_string().as_bytes(),
                )
                .unwrap();
            }));
            zhur_sdk::__internals::wapc_guest::register_function("http", outer_handler);
        }
    };
}