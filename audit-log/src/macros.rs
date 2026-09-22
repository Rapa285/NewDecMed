#[macro_export]
macro_rules! current_fn {
    () => {{
        fn f() {}
        let current = std::any::type_name_of_val(&f);
        current.strip_suffix("::f").unwrap_or(current)
    }};
}
