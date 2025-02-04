fn main() {
    #[cfg(all(feature = "ubx_series8", feature = "ubx_series9"))]
    compile_error!(
        r#"The "ubx_series8" and "ubx_series9" features are mutually exclusive and cannot be activated at the same time. Please disable one or the other."#
    );

    #[cfg(all(not(feature = "ubx_series8"), not(feature = "ubx_series9")))]
    compile_error!(
        r#"At least one feature "ubx_series8" or "ubx_series9" has to be selected. Please select only one."#
    );
}
