pub mod blake3;
pub mod routes;
pub mod security;

#[macro_export]
macro_rules! mutex_lock {
    ($e:expr) => {
        $e.lock().unwrap()
    };
}

#[macro_export]
macro_rules! print_flush {
    ($e:literal) => {
        print!($e);
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
    };
}
