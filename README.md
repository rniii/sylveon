# sylveon

A very simple way to parse CLI arguments.

```rs
sylveon::parse! {
    /// Display hidden files
    all,
    /// Increase output
    verbose+,
    /// List of files to ignore
    ignore?,

    /// List files
    ..path => { /* ... */ }
}
```
